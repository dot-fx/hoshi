use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    list::service::{
        FilterQuery, ListResponse, ListService, SingleEntryResponse,
        SuccessResponse, UpsertEntryBody, UpsertEntryResponse,
    },
    state::AppState
};

pub fn list_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/list", get(get_list))
        .route("/list/entry", post(upsert_entry))
        .route("/list/entry/:cid", get(get_single_entry).delete(delete_entry))
}

async fn get_list(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Query(query): Query<FilterQuery>,
) -> AppResult<Json<ListResponse>> {
    let result = ListService::get_list(&state, user_id, query).await?;
    Ok(Json(result))
}

async fn get_single_entry(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(cid): Path<String>,
) -> AppResult<Json<SingleEntryResponse>> {
    let result = ListService::get_single_entry(&state, user_id, cid).await?;
    Ok(Json(result))
}

async fn upsert_entry(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Json(body): Json<UpsertEntryBody>,
) -> AppResult<Json<UpsertEntryResponse>> {
    let result = ListService::upsert_entry(state, user_id, body).await?;
    Ok(Json(result))
}

async fn delete_entry(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<i32>,
    Path(cid): Path<String>,
) -> AppResult<Json<SuccessResponse>> {
    let result = ListService::delete_entry(state, user_id, cid).await?;
    Ok(Json(result))
}