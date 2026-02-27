use axum::{
    extract::{Json, Path, State},
    routing::{delete, get, put},
    Extension, Router,
};
use std::sync::Arc;

use crate::error::AppResult;
use hoshi_core::{
    collections::service::{
        AddImageToCollectionRequest, CollectionService, CollectionImagesResponse,
        CollectionListResponse, CollectionResponse, CreateCollectionRequest,
        CreateCollectionResponse, ReorderCollectionRequest, SuccessResponse,
    },
    state::AppState,
};

pub fn collection_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/collections", get(get_collections).post(create_collection))
        .route("/collections/:id", get(get_collection).put(update_collection).delete(delete_collection))
        .route("/collections/:id/images", get(get_collection_images).post(add_image_to_collection))
        .route("/collections/:id/images/:image_id", delete(remove_image_from_collection))
        .route("/collections/:id/reorder", put(reorder_collection))
}

async fn get_collections(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<CollectionListResponse>> {
    let result = CollectionService::get_user_collections(&state, user_id)?;
    Ok(Json(result))
}

async fn create_collection(
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCollectionRequest>,
) -> AppResult<Json<CreateCollectionResponse>> {
    let result = CollectionService::create_collection(&state, user_id, payload)?;
    Ok(Json(result))
}

async fn get_collection(
    Path(id): Path<String>,
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<CollectionResponse>> {
    let result = CollectionService::get_collection(&state, &id, user_id)?;
    Ok(Json(result))
}

async fn update_collection(
    Path(id): Path<String>,
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCollectionRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let result = CollectionService::update_collection(&state, &id, user_id, payload)?;
    Ok(Json(result))
}

async fn delete_collection(
    Path(id): Path<String>,
    Extension(user_id): Extension<i32>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SuccessResponse>> {
    let result = CollectionService::delete_collection(&state, &id, user_id)?;
    Ok(Json(result))
}

async fn get_collection_images(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<CollectionImagesResponse>> {
    let result = CollectionService::get_collection_images(&state, &id)?;
    Ok(Json(result))
}

async fn add_image_to_collection(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddImageToCollectionRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let result = CollectionService::add_image_to_collection(&state, &id, payload).await?;
    Ok(Json(result))
}

async fn remove_image_from_collection(
    Path((id, image_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<SuccessResponse>> {
    let result = CollectionService::remove_image_from_collection(&state, &id, &image_id)?;
    Ok(Json(result))
}

async fn reorder_collection(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderCollectionRequest>,
) -> AppResult<Json<SuccessResponse>> {
    let result = CollectionService::reorder_collection(&state, &id, payload)?;
    Ok(Json(result))
}