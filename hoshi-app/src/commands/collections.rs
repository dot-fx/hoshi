use hoshi_core::{
    collections::service::{
        AddImageToCollectionRequest, CollectionService, CollectionImagesResponse,
        CollectionListResponse, CollectionResponse, CreateCollectionRequest,
        CreateCollectionResponse, ReorderCollectionRequest,
    },
    state::AppState,
};
use std::sync::Arc;
use tauri::State;
use crate::{TauriSession, require_auth};

#[tauri::command]
pub async fn get_collections(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<CollectionListResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    CollectionService::get_user_collections(&state, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_collection(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    payload: CreateCollectionRequest,
) -> Result<CreateCollectionResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    CollectionService::create_collection(&state, user_id, payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_collection(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    id: String,
) -> Result<CollectionResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    CollectionService::get_collection(&state, &id, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_collection(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    id: String,
    payload: CreateCollectionRequest,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    CollectionService::update_collection(&state, &id, user_id, payload)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_collection(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    id: String,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    CollectionService::delete_collection(&state, &id, user_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_collection_images(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<CollectionImagesResponse, String> {
    CollectionService::get_collection_images(&state, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_image_to_collection(
    state: State<'_, Arc<AppState>>,
    id: String,
    payload: AddImageToCollectionRequest,
) -> Result<(), String> {
    CollectionService::add_image_to_collection(&state, &id, payload)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_image_from_collection(
    state: State<'_, Arc<AppState>>,
    id: String,
    image_id: String,
) -> Result<(), String> {
    CollectionService::remove_image_from_collection(&state, &id, &image_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reorder_collection(
    state: State<'_, Arc<AppState>>,
    id: String,
    payload: ReorderCollectionRequest,
) -> Result<(), String> {
    CollectionService::reorder_collection(&state, &id, payload)
        .map(|_| ())
        .map_err(|e| e.to_string())
}