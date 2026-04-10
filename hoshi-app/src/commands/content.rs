use hoshi_core::{
    error::CoreError,
    state::AppState,
};
use crate::{require_auth, TauriSession};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;
use hoshi_core::content::models::{ExtensionSource, FullContent, Metadata};
use hoshi_core::content::services::content::ContentService;
use hoshi_core::content::services::extensions::ExtensionService;
use hoshi_core::content::services::home::HomeService;
use hoshi_core::content::services::mapping::MappingService;
use hoshi_core::content::services::search::SearchService;
use hoshi_core::content::types::{ContentListResponse, ExtensionSearchResponse, PlayResponse, SearchParams, UpdateExtensionMappingRequest, UpdateTrackerMappingRequest};
use hoshi_core::tracker::provider::TrackerMedia;
use hoshi_core::tracker::types::TrackerMapping;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_home_content(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Value, CoreError> {
    let user_id = require_auth(&session_state).await?;
    HomeService::get_home_view(state.inner(), user_id)
        .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_content(
    state: State<'_, Arc<AppState>>,
    source: String,
    source_id: String,
) -> Result<FullContent, CoreError> {
    ContentService::get_content(state.inner(), &source, &source_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_content(
    state: State<'_, Arc<AppState>>,
    cid: String,
    meta: Metadata,
) -> Result<FullContent, CoreError> {
    ContentService::update_content(state.inner(), &cid, meta).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: SearchParams,
) -> Result<ContentListResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    let limit  = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let res_value = SearchService::search(state.inner(), query, user_id).await?;
    let total = res_value.as_array().map(|a| a.len()).unwrap_or(0);
    let data: Vec<TrackerMedia> = serde_json::from_value(res_value)
        .map_err(|e| CoreError::Internal(format!("Error deserializing search results: {}", e)))?;

    Ok(ContentListResponse {
        data,
        total,
        limit,
        offset,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_content_items(
    state: State<'_, Arc<AppState>>,
    cid: String,
    ext_name: String,
) -> Result<Value, CoreError> {
    ExtensionService::get_content_items(state.inner(), &cid, &ext_name).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn play_content_by_number(
    state: State<'_, Arc<AppState>>,
    cid: String,
    ext_name: String,
    number: f64,
    server: Option<String>,
    category: Option<String>,
) -> Result<PlayResponse, CoreError> {
    let res = ExtensionService::play_content(state.inner(), &cid, &ext_name, number, server, category).await?;

    Ok(PlayResponse {
        play_type: res["type"].clone(),
        data:      res["data"].clone(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    mut mapping: TrackerMapping,
) -> Result<(), CoreError> {
    mapping.cid = cid;
    MappingService::add_tracker_mapping(&state.pool, mapping).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_extension_source(
    state: State<'_, Arc<AppState>>,
    cid: String,
    mut source: ExtensionSource,
) -> Result<i64, CoreError> {
    source.cid = cid;
    MappingService::add_extension_mapping(state.inner(), source).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_extension_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: UpdateExtensionMappingRequest,
) -> Result<FullContent, CoreError> {
    MappingService::update_extension_mapping(state.inner(), &cid, &req.extension_name, &req.extension_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: UpdateTrackerMappingRequest,
) -> Result<(), CoreError> {
    MappingService::update_tracker_mapping(state.inner(), &cid, &req.tracker_name, &req.tracker_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    tracker_name: String,
) -> Result<(), CoreError> {
    MappingService::delete_tracker_mapping(state.inner(), &cid, &tracker_name).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_extension(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    params: SearchParams,
) -> Result<ExtensionSearchResponse, CoreError> {
    SearchService::search_extension(state.inner(), &ext_name, params.query, params.extension_filters).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_trending(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    media_type: String,
) -> Result<Value, CoreError> {
    if !matches!(media_type.as_str(), "anime" | "manga" | "novel") {
        return Err(CoreError::BadRequest("error.content.invalid_media_type".into()));
    }
    let user_id = require_auth(&session_state).await?;
    HomeService::get_trending(
        &state,
        &media_type,
        user_id,
    ).await
}