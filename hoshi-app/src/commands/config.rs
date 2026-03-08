use crate::TauriSession;
use hoshi_core::{
    state::AppState,
    config::service::ConfigService,
    error::CoreError,
};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_user_config(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Value, String> {
    let user_id = resolve_user_id(&session_state).await?;
    ConfigService::get_config(&state, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn patch_user_config(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    patch: Value,
) -> Result<Value, String> {
    let user_id = resolve_user_id(&session_state).await?;
    ConfigService::patch_config(&state, user_id, patch).map_err(|e| e.to_string())
}

async fn resolve_user_id(session_state: &TauriSession) -> Result<i32, String> {
    let guard = session_state.user_id.read().await;
    guard
        .as_ref()
        .and_then(|id| id.parse::<i32>().ok())
        .ok_or_else(|| CoreError::AuthError("No active session".into()).to_string())
}