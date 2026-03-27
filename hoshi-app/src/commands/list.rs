use hoshi_core::{
    list::service::{
        FilterQuery, ListResponse, ListService, SingleEntryResponse,
        UpsertEntryBody, UpsertEntryResponse, UserStats,
    },
    state::AppState,
    error::CoreError,
};
use std::sync::Arc;
use tauri::State;
use crate::{TauriSession, require_auth};

#[tauri::command]
pub async fn get_list(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    query: FilterQuery,
) -> Result<ListResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_list(&state, user_id, query).await
}

#[tauri::command]
pub async fn get_stats(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<UserStats, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_user_stats(&state, user_id).await
}

#[tauri::command]
pub async fn get_single_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<SingleEntryResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::get_single_entry(&state, user_id, cid).await
}

#[tauri::command]
pub async fn upsert_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpsertEntryBody,
) -> Result<UpsertEntryResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::upsert_entry(state.inner().clone(), user_id, body).await
}

#[tauri::command]
pub async fn delete_entry(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;
    ListService::delete_entry(state.inner().clone(), user_id, cid).await?;
    Ok(())
}