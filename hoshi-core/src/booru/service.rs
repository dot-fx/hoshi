use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{CoreError, CoreResult};
use crate::extensions::ExtensionManager;


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoQuery {
    pub provider: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteQuery {
    pub q: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub provider: Option<String>,
    pub q: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    #[serde(flatten)]
    pub filters: HashMap<String, Value>,
}


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: Option<Value>,
    pub image: Option<Value>,
    #[serde(rename = "type")]
    pub result_type: Value,
    pub tags: Option<Value>,
    pub title: Option<Value>,
    pub headers: Option<Value>,
    pub provider: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub page: Value,
    pub has_next_page: Value,
    pub total: Value,
    pub results: Vec<SearchResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub id: Value,
    pub provider: String,
    pub image: Option<Value>,
    #[serde(rename = "type")]
    pub info_type: Option<Value>,
    pub tags: Option<Value>,
    pub title: Option<Value>,
    pub artist: Option<Value>,
    pub headers: Option<Value>,
    pub original_link: Option<Value>,
}

pub struct BooruService;

impl BooruService {
    pub async fn search_in_extension(
        manager: &ExtensionManager,
        params: SearchQuery,
    ) -> CoreResult<SearchResponse> {
        let provider = params
            .provider
            .ok_or_else(|| CoreError::BadRequest("Missing provider".into()))?;
        let query = params.q.unwrap_or_default();
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(48);

        let filters_json = serde_json::to_value(params.filters).unwrap_or(json!({}));
        let args = vec![json!(query), json!(page), json!(per_page), filters_json];

        match manager
            .call_extension_function(&provider, "search", args)
            .await
        {
            Ok(results) => {
                let empty_vec = vec![];
                let raw_results = results
                    .get("results")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&empty_vec);

                let normalized = raw_results
                    .iter()
                    .map(|r| SearchResult {
                        id: r.get("id").cloned(),
                        image: r.get("image").cloned(),
                        result_type: r
                            .get("type")
                            .cloned()
                            .unwrap_or_else(|| json!("image")),
                        tags: r.get("tags").cloned(),
                        title: r.get("title").cloned(),
                        headers: r.get("headers").cloned(),
                        provider: provider.clone(),
                    })
                    .collect();

                Ok(SearchResponse {
                    page: results.get("page").cloned().unwrap_or(json!(page)),
                    has_next_page: results
                        .get("hasNextPage")
                        .cloned()
                        .unwrap_or(json!(false)),
                    total: results.get("total").cloned().unwrap_or(json!(0)),
                    results: normalized,
                })
            }
            Err(e) => {
                tracing::error!("Gallery search error: {}", e);
                Ok(SearchResponse {
                    page: json!(1),
                    has_next_page: json!(false),
                    total: json!(0),
                    results: vec![],
                })
            }
        }
    }

    pub async fn get_info(
        manager: &ExtensionManager,
        id: String,
        provider_opt: Option<String>,
    ) -> CoreResult<ImageInfo> {
        let provider_name = provider_opt
            .ok_or_else(|| CoreError::NotFound("Gallery item not found in any extension".into()))?;

        let args = vec![json!(id)];
        let info = manager
            .call_extension_function(&provider_name, "getInfo", args)
            .await?;

        Ok(ImageInfo {
            id: info.get("id").cloned().unwrap_or(json!(id)),
            provider: provider_name,
            image: info.get("image").cloned(),
            info_type: info.get("type").cloned(),
            tags: info.get("tags").cloned(),
            title: info.get("title").cloned(),
            artist: info.get("artist").cloned(),
            headers: info.get("headers").cloned(),
            original_link: info.get("original_link").cloned(),
        })
    }

    pub async fn serve_local_image(
        provider: &str,
        filename: &str,
    ) -> CoreResult<(String, Vec<u8>)> {
        use crate::paths;

        let images_base = paths::get_images_path();
        let file_path = images_base.join(provider).join(filename);

        if !file_path.exists() {
            return Err(CoreError::NotFound("Image not found".into()));
        }

        let canonical_file = file_path
            .canonicalize()
            .map_err(|_| CoreError::NotFound("Invalid path".into()))?;
        let canonical_base = images_base
            .canonicalize()
            .map_err(|_| CoreError::Internal("Invalid base path".into()))?;

        if !canonical_file.starts_with(&canonical_base) {
            return Err(CoreError::BadRequest("Invalid path".into()));
        }

        let content_type = Self::get_content_type(filename).to_string();
        let bytes = tokio::fs::read(&file_path)
            .await
            .map_err(|e| CoreError::Internal(format!("Failed to read file: {}", e)))?;

        Ok((content_type, bytes))
    }

    fn get_content_type(filename: &str) -> &'static str {
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        }
    }

    pub async fn get_autocomplete(
        manager: &ExtensionManager,
        provider: String,
        q: Option<String>,
    ) -> CoreResult<Value> {
        let query = q.unwrap_or_default();
        let args = vec![json!(query)];

        match manager
            .call_extension_function(&provider, "autocomplete", args)
            .await
        {
            Ok(res) => Ok(res),
            Err(_) => Ok(json!([])),
        }
    }
}