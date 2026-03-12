use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    progress::service::{
        ContinueWatchingResponse, ContentProgressResponse, ProgressResponse, ProgressService,
        UpdateAnimeProgressBody, UpdateChapterProgressBody,
    },
    state::AppState,
};

pub fn progress_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/progress/anime", post(update_anime_progress))
        .route("/progress/chapter", post(update_chapter_progress))
        .route("/progress/continue", get(get_continue_watching))
        .route("/progress/:cid", get(get_content_progress))
}

#[derive(Deserialize)]
struct ContinueQuery {
    limit: Option<i64>,
}

async fn update_anime_progress(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Json(body): Json<UpdateAnimeProgressBody>,
) -> AppResult<Json<ProgressResponse>> {
    let result = ProgressService::update_anime_progress(&state, user_id, body).await?;
    Ok(Json(result))
}

async fn update_chapter_progress(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Json(body): Json<UpdateChapterProgressBody>,
) -> AppResult<Json<ProgressResponse>> {
    let result = ProgressService::update_chapter_progress(&state, user_id, body).await?;
    Ok(Json(result))
}

async fn get_continue_watching(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Query(query): Query<ContinueQuery>,
) -> AppResult<Json<ContinueWatchingResponse>> {
    let result = ProgressService::get_continue_watching(&state, user_id, query.limit).await?;
    Ok(Json(result))
}

async fn get_content_progress(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(cid): Path<String>,
) -> AppResult<Json<ContentProgressResponse>> {
    let result = ProgressService::get_content_progress(&state, user_id, cid).await?;
    Ok(Json(result))
}