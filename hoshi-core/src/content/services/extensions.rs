use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use serde_json::{json, Value};
use crate::content::models::ContentType;
use crate::content::repositories::cache::CacheRepository;
use crate::content::repositories::extension::ExtensionRepository;
use crate::content::repositories::content::ContentRepository;
use crate::content::services::resolver::ContentResolverService;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

pub struct ExtensionService;

impl ExtensionService {

    #[instrument(skip(state))]
    pub async fn save_extension_metadata(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        ext_id: &str,
    ) {
        debug!(ext = %ext_name, id = %ext_id, "Fetching metadata from extension");

        let ext_meta = match ContentResolverService::fetch_ext_metadata(state, ext_name, ext_id).await {
            Ok(v) => v,
            Err(e) => {
                warn!(ext = %ext_name, id = %ext_id, error = ?e, "Failed to fetch extension metadata");
                return;
            }
        };

        let now = chrono::Utc::now().timestamp();
        let meta = ContentResolverService::ext_meta_to_metadata(cid, ext_name, ext_id, &ext_meta, now);

        match ContentRepository::upsert_metadata(&state.pool, &meta).await {
            Ok(_) => info!(cid = %cid, source = %ext_name, "Extension metadata saved"),
            Err(e) => error!(cid = %cid, source = %ext_name, error = ?e, "Failed to upsert extension metadata"),
        }
    }

    #[instrument(skip(state))]
    pub async fn get_content_items(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
    ) -> CoreResult<Value> {
        let (content_type, ext_id) = ContentResolverService::ensure_extension_link(state, cid, ext_name).await?;

        let cache_key = format!("items:{}:{}", ext_name, ext_id);

        let cached = CacheRepository::get(&state.pool, &cache_key).await?;

        if let Some(data) = cached {
            debug!(cid = %cid, ext = %ext_name, "Returning cached items");
            return Ok(data);
        }

        let func = match content_type {
            ContentType::Anime => "findEpisodes",
            _ => "findChapters",
        };
        debug!(cid = %cid, ext = %ext_name, func = %func, "Fetching items from extension");

        let items = state
            .extension_manager
            .read()
            .await
            .call_extension_function(ext_name, func, vec![json!(ext_id)])
            .await
            .map_err(|e| {
                error!(ext = %ext_name, error = ?e, "Failed to fetch items");
                CoreError::Internal("error.system.external".into())
            })?;

        let _ = CacheRepository::set(&state.pool, &cache_key, ext_name, "items", &items, 1800).await;

        Ok(items)
    }

    #[instrument(skip(state, server, category))]
    pub async fn play_content(
        state: &Arc<AppState>,
        cid: &str,
        ext_name: &str,
        number: f64,
        server: Option<String>,
        category: Option<String>,
    ) -> CoreResult<Value> {
        let items_list = Self::get_content_items(state, cid, ext_name).await?;

        let (content_type, _ext_id) = {
            let (type_str, id) = ExtensionRepository::get_extension_id_and_type(&state.pool, cid, ext_name)
                .await?
                .ok_or_else(|| CoreError::Internal("error.content.link_failed".into()))?;

            let ct = serde_json::from_str::<ContentType>(&format!("\"{}\"", type_str))
                .unwrap_or(ContentType::Anime);
            (ct, id)
        };

        let real_id = items_list
            .as_array()
            .ok_or_else(|| CoreError::Internal("error.content.invalid_items_list".into()))?
            .iter()
            .find(|item| {
                item.get("number")
                    .and_then(|v| v.as_f64())
                    .map(|n| (n - number).abs() < 0.01)
                    .unwrap_or(false)
            })
            .and_then(|item| item.get("id")?.as_str().map(String::from))
            .ok_or_else(|| {
                warn!(cid = %cid, ext = %ext_name, number = %number, "Item number not found");
                CoreError::NotFound("error.content.item_number_not_found".into())
            })?;

        match content_type {
            ContentType::Anime => {
                let srv = server.unwrap_or_else(|| "default".into());
                let cat = category.unwrap_or_else(|| "sub".into());
                debug!(ext = %ext_name, id = %real_id, server = %srv, "Fetching video servers");
                let data = state
                    .extension_manager
                    .read()
                    .await
                    .call_extension_function(ext_name, "findEpisodeServer", vec![json!(real_id), json!(srv), json!(cat)])
                    .await
                    .map_err(|e| {
                        error!(ext = %ext_name, error = ?e, "Failed to get video server data");
                        CoreError::Internal("error.system.external".into())
                    })?;
                Ok(json!({ "type": "video", "data": data }))
            }
            _ => {
                debug!(ext = %ext_name, id = %real_id, "Fetching chapter pages");
                let data = state
                    .extension_manager
                    .read()
                    .await
                    .call_extension_function(ext_name, "findChapterPages", vec![json!(real_id)])
                    .await
                    .map_err(|e| {
                        error!(ext = %ext_name, error = ?e, "Failed to get chapter pages");
                        CoreError::Internal("error.system.external".into())
                    })?;
                Ok(json!({ "type": "reader", "data": data }))
            }
        }
    }
}