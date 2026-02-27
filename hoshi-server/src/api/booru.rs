use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    booru::service::{AutocompleteQuery, BooruService, ImageInfo, InfoQuery, SearchQuery, SearchResponse},
    error::CoreError,
    state::AppState,
};

pub fn booru_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/booru/search", get(search))
        .route("/booru/info/:id", get(get_info))
        .route("/booru/:provider/autocomplete", get(get_autocomplete))
        .route("/booru/local/:provider/:filename", get(serve_local_image))
}

async fn search(
    Query(params): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SearchResponse>> {
    let result = BooruService::search_in_extension(&state.extension_manager, params).await?;
    Ok(Json(result))
}

async fn get_info(
    Path(id): Path<String>,
    Query(params): Query<InfoQuery>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<ImageInfo>> {
    let result = BooruService::get_info(&state.extension_manager, id, params.provider).await?;
    Ok(Json(result))
}

async fn get_autocomplete(
    Path(provider): Path<String>,
    Query(params): Query<AutocompleteQuery>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let result = BooruService::get_autocomplete(&state.extension_manager, provider, params.q).await?;
    Ok(Json(result))
}

async fn serve_local_image(
    Path((provider, filename)): Path<(String, String)>,
    State(_state): State<Arc<AppState>>,
) -> AppResult<Response> {
    let (content_type, bytes) = BooruService::serve_local_image(&provider, &filename).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(bytes))
        .map_err(|e| CoreError::Internal(format!("Failed to build response: {}", e)))?)
}