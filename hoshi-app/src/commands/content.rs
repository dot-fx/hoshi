use hoshi_core::{
    content::{
        ContentMetadata, ExtensionSource, ContentWithMappings,
        ContentImportService, ContentService, MappingService,
        ContentListResponse, ResolveExtensionResponse, ExtensionSearchResponse,
        PlayResponse, SearchQuery, LinkTrackerRequest,
        UpdateExtensionMappingRequest, UpdateTrackerMappingRequest,
    },
    tracker::repository::TrackerMapping,
    state::AppState,
    error::CoreError,
};
use crate::{require_auth, TauriSession};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_home_content(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Value, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ContentImportService::get_home_view(state.inner().db.clone(), state.inner().tracker_registry.clone(), user_id)
        .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_content(
    state: State<'_, Arc<AppState>>,
    cid: String,
    session_state: State<'_, TauriSession>,
) -> Result<ContentWithMappings, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ContentService::get_content(state.inner(), &cid, Option::from(user_id)).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_content(
    state: State<'_, Arc<AppState>>,
    cid: String,
    meta: ContentMetadata,
) -> Result<ContentWithMappings, CoreError> {
    ContentService::update_content(state.inner(), &cid, meta).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_content(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: SearchQuery,
) -> Result<ContentListResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;

    let limit  = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let res = ContentService::search_content(state.inner(), query.into_params(), user_id).await?;

    Ok(ContentListResponse {
        data: res.data,
        total: res.total,
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
    ContentService::get_content_items(state.inner(), &cid, &ext_name).await
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
    let res = ContentService::play_content(state.inner(), &cid, &ext_name, number, server, category).await?;

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
    MappingService::add_tracker_mapping(state.inner(), mapping)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_extension_source(
    state: State<'_, Arc<AppState>>,
    cid: String,
    mut source: ExtensionSource,
) -> Result<i64, CoreError> {
    source.cid = cid;
    MappingService::add_extension_source(state.inner(), source)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_extension_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: UpdateExtensionMappingRequest,
) -> Result<ContentWithMappings, CoreError> {
    MappingService::update_extension_mapping(state.inner(), &cid, &req.extension_name, &req.extension_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: UpdateTrackerMappingRequest,
) -> Result<(), CoreError> {
    MappingService::update_tracker_mapping(state.inner(), &cid, &req.tracker_name, &req.tracker_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    tracker_name: String,
) -> Result<(), CoreError> {
    MappingService::delete_tracker_mapping(state.inner(), &cid, &tracker_name)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_by_tracker(
    state: State<'_, Arc<AppState>>,
    tracker: String,
    id: String,
) -> Result<ContentWithMappings, CoreError> {
    MappingService::resolve_by_tracker(state.inner(), &tracker, &id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_by_extension(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    ext_id: String,
) -> Result<ContentWithMappings, CoreError> {
    MappingService::resolve_by_extension(state.inner(), &ext_name, &ext_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn link_tracker(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: LinkTrackerRequest,
) -> Result<ContentWithMappings, CoreError> {
    MappingService::link_tracker(state.inner(), &cid, &req.tracker_name, &req.tracker_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_extension_item(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    ext_id: String,
) -> Result<ResolveExtensionResponse, CoreError> {
    MappingService::resolve_extension_item(state.inner(), &ext_name, &ext_id).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_extension_direct(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    params: SearchQuery,
) -> Result<ExtensionSearchResponse, CoreError> {
    MappingService::search_extension_direct(state.inner(), &ext_name, params.query, params.extension_filters).await
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
    ContentImportService::get_trending(
        state.inner().db.clone(),
        state.inner().tracker_registry.clone(),
        &media_type,
        user_id,
    ).await
}