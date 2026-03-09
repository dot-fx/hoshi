use crate::{require_auth, TauriSession};
use hoshi_core::tracker::repository::AddIntegrationRequest;
use hoshi_core::{
    state::AppState,
    tracker::service::{IntegrationService, SuccessResponse, TrackerInfoResponse},
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_trackers(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Vec<TrackerInfoResponse>, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    IntegrationService::list_trackers(&state, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: AddIntegrationRequest,
) -> Result<SuccessResponse, String> {
    let user_id = require_auth(&session_state)
        .await?
        .parse::<i32>()
        .map_err(|_| "Invalid user ID".to_string())?;

    let res = IntegrationService::add_integration(Arc::clone(&state), user_id, body)
        .await
        .map_err(|e| e.to_string())?;

    Ok(res)
}

#[tauri::command]
pub async fn remove_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
) -> Result<SuccessResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    IntegrationService::remove_integration(&state, user_id, &tracker_name).map_err(|e| e.to_string())
}