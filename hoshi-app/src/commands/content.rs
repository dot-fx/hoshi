use hoshi_core::{
    content::{
        ContentMetadata, ExtensionSource, ContentWithMappings,
        ContentImportService, ContentService, MappingService,
        ContentListResponse, ResolveExtensionResponse, ExtensionSearchResponse,
        PlayResponse, CreateContentRequest, SearchQuery, LinkTrackerRequest,
        UpdateExtensionMappingRequest, UpdateTrackerMappingRequest,
    },
    tracker::repository::TrackerMapping,
    state::AppState,
};
use crate::TauriSession;
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_home_content(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Value, String> {
    let user_id = session_state.user_id.read().await
        .as_ref()
        .and_then(|id| id.parse::<i32>().ok());
    ContentImportService::get_home_view(state.inner().db.clone(), state.inner().tracker_registry.clone(), user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_content(
    state: State<'_, Arc<AppState>>,
    req: CreateContentRequest,
) -> Result<ContentWithMappings, String> {
    ContentService::create_content(
        state.inner(),
        req.content_type,
        req.nsfw,
        req.metadata,
        req.tracker_mappings,
        req.extension_sources,
    )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_content(
    state: State<'_, Arc<AppState>>,
    cid: String,
    session_state: State<'_, TauriSession>,
) -> Result<ContentWithMappings, String> {
    let user_id = session_state.user_id.read().await
        .as_ref()
        .and_then(|id| id.parse::<i32>().ok());
    ContentService::get_content(state.inner(), &cid, user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_content(
    state: State<'_, Arc<AppState>>,
    cid: String,
    meta: ContentMetadata,
) -> Result<ContentWithMappings, String> {
    ContentService::update_content(state.inner(), &cid, meta)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_content(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: SearchQuery,
) -> Result<ContentListResponse, String> {
    let user_id = session_state.user_id.read().await
        .as_ref()
        .and_then(|id| id.parse::<i32>().ok());
    let limit  = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let res = ContentService::search_content(state.inner(), query.into_params(), user_id)
        .await
        .map_err(|e| e.to_string())?;

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
) -> Result<Value, String> {
    ContentService::get_content_items(state.inner(), &cid, &ext_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn play_content_by_number(
    state: State<'_, Arc<AppState>>,
    cid: String,
    ext_name: String,
    number: f64,
    server: Option<String>,
    category: Option<String>,
) -> Result<PlayResponse, String> {
    let res = ContentService::play_content(state.inner(), &cid, &ext_name, number, server, category)
        .await
        .map_err(|e| e.to_string())?;

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
) -> Result<(), String> {
    mapping.cid = cid;
    MappingService::add_tracker_mapping(state.inner(), mapping).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_extension_source(
    state: State<'_, Arc<AppState>>,
    cid: String,
    mut source: ExtensionSource,
) -> Result<i64, String> {
    source.cid = cid;
    MappingService::add_extension_source(state.inner(), source).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_extension_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: UpdateExtensionMappingRequest,
) -> Result<ContentWithMappings, String> {
    MappingService::update_extension_mapping(state.inner(), &cid, &req.extension_name, &req.extension_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: UpdateTrackerMappingRequest,
) -> Result<(), String> {
    MappingService::update_tracker_mapping(state.inner(), &cid, &req.tracker_name, &req.tracker_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_tracker_mapping(
    state: State<'_, Arc<AppState>>,
    cid: String,
    tracker_name: String,
) -> Result<(), String> {
    MappingService::delete_tracker_mapping(state.inner(), &cid, &tracker_name)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_by_tracker(
    state: State<'_, Arc<AppState>>,
    tracker: String,
    id: String,
) -> Result<ContentWithMappings, String> {
    MappingService::resolve_by_tracker(state.inner(), &tracker, &id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_by_extension(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    ext_id: String,
) -> Result<ContentWithMappings, String> {
    MappingService::resolve_by_extension(state.inner(), &ext_name, &ext_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn link_tracker(
    state: State<'_, Arc<AppState>>,
    cid: String,
    req: LinkTrackerRequest,
) -> Result<ContentWithMappings, String> {
    MappingService::link_tracker(state.inner(), &cid, &req.tracker_name, &req.tracker_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn resolve_extension_item(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    ext_id: String,
) -> Result<ResolveExtensionResponse, String> {
    MappingService::resolve_extension_item(state.inner(), &ext_name, &ext_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_extension_direct(
    state: State<'_, Arc<AppState>>,
    ext_name: String,
    params: SearchQuery,
) -> Result<ExtensionSearchResponse, String> {
    MappingService::search_extension_direct(state.inner(), &ext_name, params.query, params.extension_filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_trending(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    media_type: String,
) -> Result<Value, String> {
    if !matches!(media_type.as_str(), "anime" | "manga" | "novel") {
        return Err(format!("media_type inválido: {}", media_type));
    }
    let user_id = session_state.user_id.read().await
        .as_ref()
        .and_then(|id| id.parse::<i32>().ok());
    ContentImportService::get_trending(
        state.inner().db.clone(),
        state.inner().tracker_registry.clone(),
        &media_type,
        user_id,
    )
        .await
        .map_err(|e| e.to_string())
}
