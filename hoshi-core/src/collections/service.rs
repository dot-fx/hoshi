use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::booru::repository::{BooruRepo, SavedImage};
use crate::collections::repository::{Collection, CollectionRepo};
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddImageToCollectionRequest {
    pub id: String,
    pub provider: String,
    pub title: String,
    pub artist: String,
    pub tags: Option<String>,
    pub original_link: String,
    pub image_url: String,
    pub headers: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderCollectionRequest {
    pub ordered_ids: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionListResponse {
    pub collections: Vec<Collection>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionResponse {
    pub collection: Collection,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionImagesResponse {
    pub images: Vec<SavedImage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionResponse {
    pub success: bool,
    pub id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse {
    pub success: bool,
}

pub struct CollectionService;

impl CollectionService {
    pub fn get_user_collections(
        state: &AppState,
        user_id: i32,
    ) -> CoreResult<CollectionListResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let collections = CollectionRepo::get_by_user(&conn_lock, user_id)?;
        Ok(CollectionListResponse { collections })
    }

    pub fn create_collection(
        state: &AppState,
        user_id: i32,
        payload: CreateCollectionRequest,
    ) -> CoreResult<CreateCollectionResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let id = Uuid::new_v4().to_string();
        let collection = Collection {
            id: id.clone(),
            user_id,
            name: payload.name,
            description: payload.description.unwrap_or_default(),
            is_private: payload.is_private.unwrap_or(false),
            cover_id: None,
            created_at: Utc::now().timestamp(),
        };

        CollectionRepo::create(&conn_lock, &collection)?;
        Ok(CreateCollectionResponse { success: true, id })
    }

    pub fn get_collection(
        state: &AppState,
        id: &str,
        user_id: i32,
    ) -> CoreResult<CollectionResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let collection = CollectionRepo::get_by_user(&conn_lock, user_id)?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| CoreError::NotFound("Collection not found".into()))?;

        Ok(CollectionResponse { collection })
    }

    pub fn update_collection(
        state: &AppState,
        id: &str,
        user_id: i32,
        payload: CreateCollectionRequest,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let mut collection = CollectionRepo::get_by_user(&conn_lock, user_id)?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| CoreError::NotFound("Collection not found".into()))?;

        collection.name = payload.name;
        collection.description = payload.description.unwrap_or_default();
        collection.is_private = payload.is_private.unwrap_or(false);

        CollectionRepo::update(&conn_lock, &collection)?;
        Ok(SuccessResponse { success: true })
    }

    pub fn delete_collection(
        state: &AppState,
        id: &str,
        user_id: i32,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        CollectionRepo::delete(&conn_lock, id, user_id)?;
        Ok(SuccessResponse { success: true })
    }

    pub fn get_collection_images(
        state: &AppState,
        collection_id: &str,
    ) -> CoreResult<CollectionImagesResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        let images = BooruRepo::get_collection_images(&conn_lock, collection_id)?;
        Ok(CollectionImagesResponse { images })
    }

    pub async fn add_image_to_collection(
        state: &AppState,
        collection_id: &str,
        payload: AddImageToCollectionRequest,
    ) -> CoreResult<SuccessResponse> {
        let existing_image = {
            let conn = state.db.connection();
            let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;
            BooruRepo::get_image(&conn_lock, &payload.id)?
        };

        let existing_local_path = existing_image.as_ref().and_then(|img| img.local_path.clone());
        let image_exists = existing_image.is_some();

        let local_path = if !image_exists {
            Some(Self::download_and_save_image(state, &payload).await?)
        } else {
            existing_local_path
        };

        let conn = state.db.connection();
        let mut conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        if !image_exists {
            let new_image = SavedImage {
                id: payload.id.clone(),
                provider: payload.provider.clone(),
                title: payload.title.clone(),
                artist: payload.artist.clone(),
                tags: payload.tags.clone().unwrap_or_default(),
                original_link: payload.original_link.clone(),
                local_path: local_path.clone(),
                created_at: Utc::now().timestamp(),
            };
            BooruRepo::create_image(&conn_lock, &new_image)?;
        }

        CollectionRepo::add_image(
            &mut conn_lock,
            collection_id,
            &payload.id,
            Utc::now().timestamp(),
        )?;

        Ok(SuccessResponse { success: true })
    }

    pub fn remove_image_from_collection(
        state: &AppState,
        collection_id: &str,
        image_id: &str,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        CollectionRepo::remove_image(&conn_lock, collection_id, image_id)?;
        Ok(SuccessResponse { success: true })
    }

    pub fn reorder_collection(
        state: &AppState,
        collection_id: &str,
        payload: ReorderCollectionRequest,
    ) -> CoreResult<SuccessResponse> {
        let conn = state.db.connection();
        let mut conn_lock = conn.lock().map_err(|_| CoreError::Internal("DB Lock error".into()))?;

        CollectionRepo::reorder(&mut conn_lock, collection_id, payload.ordered_ids)?;
        Ok(SuccessResponse { success: true })
    }

    async fn download_and_save_image(
        state: &AppState,
        req: &AddImageToCollectionRequest,
    ) -> CoreResult<String> {
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        let images_base = &state.paths.images_path;
        let provider_dir = images_base.join(&req.provider);

        if !provider_dir.exists() {
            fs::create_dir_all(&provider_dir).await.map_err(|e| {
                CoreError::Internal(format!("Failed to create provider dir: {}", e))
            })?;
        }

        let extension = Self::extract_extension(&req.image_url).unwrap_or("jpg");
        let filename = format!("{}.{}", req.id, extension);
        let file_path = provider_dir.join(&filename);

        if file_path.exists() {
            return Ok(format!("{}/{}", req.provider, filename));
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| CoreError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let mut request = client.get(&req.image_url);

        if let Some(headers_json) = &req.headers {
            if let Some(headers_obj) = headers_json.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        request = request.header(key, value_str);
                    }
                }
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to download image: {}", e)))?;

        if !response.status().is_success() {
            return Err(CoreError::Internal(format!(
                "Failed to download image: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to read image bytes: {}", e)))?;

        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to create file: {}", e)))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to write file: {}", e)))?;

        tracing::info!("Downloaded image: {} -> {:?}", req.image_url, file_path);

        Ok(format!("{}/{}", req.provider, filename))
    }

    fn extract_extension(url: &str) -> Option<&str> {
        url.split('?')
            .next()?
            .split('.')
            .last()
            .and_then(|ext| {
                let ext_lower = ext.to_lowercase();
                match ext_lower.as_str() {
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" => Some(ext),
                    _ => None,
                }
            })
    }
}