use std::sync::Arc;
use serde_json::{json, Value};
use tracing::{debug, error, instrument};
use crate::content::types::{parse_content_type, ExtensionSearchResponse, SearchParams};
use crate::content::utils::show_adult;
use crate::error::{CoreError, CoreResult};
use crate::state::AppState;

pub struct SearchService;

impl SearchService {
    #[instrument(skip(state, params))]
    pub async fn search(
        state: &Arc<AppState>,
        params: SearchParams,
        user_id: i32,
    ) -> CoreResult<Value> {
        let show_adult = show_adult(state, user_id).await;
        let query_str  = params.query.clone().unwrap_or_default();

        debug!(query = %query_str, "Performing raw search across trackers");

        let content_type = parse_content_type(
            params.r#type.as_deref().unwrap_or("anime")
        );

        let tracker_name = match params.tracker.as_deref().unwrap_or("anilist") {
            "mal"   => "mal",
            "kitsu" => "kitsu",
            _       => "anilist",
        };

        let provider = state.tracker_registry.get(tracker_name)
            .or_else(|| state.tracker_registry.get("anilist"))
            .ok_or_else(|| {
                error!(tracker = %tracker_name, "No suitable search provider found");
                CoreError::Internal("error.tracker.no_provider_available".into())
            })?;

        let tracker = provider.name();

        let limit = params.limit.unwrap_or(20) as usize;

        let mut results = provider.search(
            params.query.as_deref(),
            content_type,
            limit,
            params.sort.as_deref(),
            params.genre.as_deref(),
            params.format.as_deref(),
            params.nsfw,
        ).await?;

        if !show_adult {
            results.retain(|media| !media.nsfw);
        }

        debug!(results = results.len(), tracker = %tracker, "Raw tracker search completed successfully");

        Ok(json!({ "total": results.len(), "data": results }))
    }

    #[instrument(skip(state, query, filters_json))]
    pub async fn search_extension(
        state: &Arc<AppState>,
        ext_id: &str,
        query: Option<String>,
        filters_json: Option<String>,
    ) -> CoreResult<ExtensionSearchResponse> {
        let filters: Value = filters_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(json!({}));

        let args = json!({ "query": query.unwrap_or_default(), "filers": filters });
        let is_nsfw = state.extension_manager.read().await.is_nsfw(ext_id);

        debug!(ext = %ext_id, "Calling extension search function");
        let raw = state.extension_manager.read().await
            .call_extension_function(ext_id, "search", vec![args])
            .await
            .map_err(|e| {
                error!(error = ?e, ext = %ext_id, "Failed to execute search on extension");
                CoreError::Internal("error.content.extension_search_failed".into())
            })?;

        let results = match raw {
            Value::Array(mut arr) => {
                for item in &mut arr {
                    if let Some(obj) = item.as_object_mut() {
                        let item_nsfw = obj.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);
                        obj.insert("nsfw".to_string(), json!(is_nsfw || item_nsfw));
                    }
                }
                Value::Array(arr)
            }
            other => other,
        };

        Ok(ExtensionSearchResponse { results })
    }
}