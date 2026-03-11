use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;
use serde_json::Value;

use crate::error::AppResult;
use hoshi_core::{
    content::{
        repository::{ContentMetadata, ExtensionSource, ContentWithMappings},
        service::{
            ContentImportService, ContentListResponse, ContentService,
            CreateContentRequest, ExtensionSearchResponse, PlayResponse,
            SearchQuery, SourceQuery, UpdateExtensionMappingRequest,
            UpdateTrackerMappingRequest, ResolveExtensionResponse, LinkTrackerRequest,
        },
    },
    tracker::repository::TrackerMapping,
    state::AppState,
};

pub fn content_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/content",                                    post(create_content))
        .route("/content/home",                               get(get_home_content))
        .route("/content/search",                             get(search_content))
        .route("/content/:cid",                               get(get_content).put(update_content))
        .route("/content/:cid/trackers",                      post(add_tracker_mapping))
        .route("/content/:cid/trackers/update",               post(update_tracker_mapping))
        .route("/content/:cid/trackers/:tracker",             delete(delete_tracker_mapping))
        .route("/content/:cid/extensions",                    post(add_extension_source))
        .route("/content/:cid/extensions/update",             post(update_extension_mapping))
        .route("/content/:cid/:extension/items",              get(get_content_items))
        .route("/content/:cid/:extension/play/:number",       get(play_content_by_number))
        .route("/content/resolve/tracker/:tracker/:id",       get(resolve_by_tracker))
        .route("/content/resolve/extension/:extension/:id",   get(resolve_by_extension))
        .route("/content/resolve/extension/:extension/:id/link", post(resolve_extension_item))
        .route("/content/:cid/link-tracker",                  post(link_tracker))
        .route("/extensions/:extension/search",               get(search_extension_direct))
        .route("/content/trending/:media_type",               get(get_trending))
}

async fn get_home_content(State(state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    let data = ContentImportService::get_home_view(state.db.clone(), state.tracker_registry.clone()).await?;
    Ok(Json(data))
}

async fn create_content(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContentRequest>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::create_content(
        &state,
        req.content_type,
        req.nsfw,
        req.metadata,
        req.tracker_mappings,
        req.extension_sources,
    ).await?;
    Ok(Json(data))
}

async fn get_content(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::get_content(&state, &cid).await?;
    Ok(Json(data))
}

async fn update_content(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
    Json(meta): Json<ContentMetadata>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::update_content(&state, &cid, meta).await?;
    Ok(Json(data))
}

async fn search_content(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<ContentListResponse>> {
    let limit  = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let res = ContentService::search_content(&state, query.into_params()).await?;

    Ok(Json(ContentListResponse {
        data: res.data,
        total: res.total,
        limit,
        offset,
    }))
}

async fn get_content_items(
    State(state): State<Arc<AppState>>,
    Path((cid, ext_name)): Path<(String, String)>,
) -> AppResult<Json<Value>> {
    let data = ContentService::get_content_items(&state, &cid, &ext_name).await?;
    Ok(Json(data))
}

async fn play_content_by_number(
    State(state): State<Arc<AppState>>,
    Path((cid, ext_name, number_str)): Path<(String, String, String)>,
    Query(q): Query<SourceQuery>,
) -> AppResult<Json<PlayResponse>> {
    let number = number_str.parse::<f64>().unwrap_or(1.0);
    let res = ContentService::play_content(&state, &cid, &ext_name, number, q.server, q.category).await?;
    Ok(Json(PlayResponse {
        play_type: res["type"].clone(),
        data:      res["data"].clone(),
    }))
}

async fn add_tracker_mapping(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
    Json(mut mapping): Json<TrackerMapping>,
) -> AppResult<()> {
    mapping.cid = cid;
    ContentService::add_tracker_mapping(&state, mapping)?;
    Ok(())
}

async fn add_extension_source(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
    Json(mut source): Json<ExtensionSource>,
) -> AppResult<Json<i64>> {
    source.cid = cid;
    let id = ContentService::add_extension_source(&state, source)?;
    Ok(Json(id))
}

async fn update_extension_mapping(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
    Json(req): Json<UpdateExtensionMappingRequest>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::update_extension_mapping(&state, &cid, &req.extension_name, &req.extension_id).await?;
    Ok(Json(data))
}

async fn update_tracker_mapping(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
    Json(req): Json<UpdateTrackerMappingRequest>,
) -> AppResult<()> {
    ContentService::update_tracker_mapping(&state, &cid, &req.tracker_name, &req.tracker_id)?;
    Ok(())
}

async fn delete_tracker_mapping(
    State(state): State<Arc<AppState>>,
    Path((cid, tracker_name)): Path<(String, String)>,
) -> AppResult<()> {
    ContentService::delete_tracker_mapping(&state, &cid, &tracker_name)?;
    Ok(())
}

async fn resolve_by_tracker(
    State(state): State<Arc<AppState>>,
    Path((tracker, id)): Path<(String, String)>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::resolve_by_tracker(&state, &tracker, &id)?;
    Ok(Json(data))
}

async fn resolve_by_extension(
    State(state): State<Arc<AppState>>,
    Path((ext_name, ext_id)): Path<(String, String)>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::resolve_by_extension(&state, &ext_name, &ext_id).await?;
    Ok(Json(data))
}

async fn link_tracker(
    State(state): State<Arc<AppState>>,
    Path(cid): Path<String>,
    Json(req): Json<LinkTrackerRequest>,
) -> AppResult<Json<ContentWithMappings>> {
    let data = ContentService::link_tracker(&state, &cid, &req.tracker_name, &req.tracker_id).await?;
    Ok(Json(data))
}

async fn resolve_extension_item(
    State(state): State<Arc<AppState>>,
    Path((ext_name, ext_id)): Path<(String, String)>,
) -> AppResult<Json<ResolveExtensionResponse>> {
    let result = ContentService::resolve_extension_item(&state, &ext_name, &ext_id).await?;
    Ok(Json(result))
}

async fn search_extension_direct(
    State(state): State<Arc<AppState>>,
    Path(ext_name): Path<String>,
    Query(params): Query<SearchQuery>,
) -> AppResult<Json<ExtensionSearchResponse>> {
    let result = ContentService::search_extension_direct(&state, &ext_name, params.query, params.extension_filters).await?;
    Ok(Json(result))
}

async fn get_trending(
    State(state): State<Arc<AppState>>,
    Path(media_type): Path<String>,
) -> AppResult<Json<Value>> {
    // Validar tipo
    if !matches!(media_type.as_str(), "anime" | "manga" | "novel") {
        return Err(hoshi_core::error::CoreError::BadRequest(
            format!("media_type debe ser anime, manga o novel, recibido: {}", media_type)
        ).into());
    }
    let data = ContentImportService::get_trending(
        state.db.clone(),
        state.tracker_registry.clone(),
        &media_type,
    ).await?;
    Ok(Json(data))
}