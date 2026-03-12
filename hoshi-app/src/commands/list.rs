use hoshi_core::{
    list::service::{
        FilterQuery, ListResponse, ListService, SingleEntryResponse,
        UpsertEntryBody, UpsertEntryResponse, UserStats,
    },
    state::AppState,
};
use std::sync::Arc;
use tauri::State;
use crate::{TauriSession, require_auth};

#[tauri::command]
pub async fn get_list(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: FilterQuery,
) -> Result<ListResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    ListService::get_list(&state, user_id, query).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stats(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<UserStats, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    ListService::get_user_stats(&state, user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_single_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<SingleEntryResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    ListService::get_single_entry(&state, user_id, cid).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upsert_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpsertEntryBody,
) -> Result<UpsertEntryResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    ListService::upsert_entry(state.inner().clone(), user_id, body).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    ListService::delete_entry(state.inner().clone(), user_id, cid).await
        .map(|_| ())
        .map_err(|e| e.to_string())
}