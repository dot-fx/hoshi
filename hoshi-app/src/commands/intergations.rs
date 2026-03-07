use hoshi_core::{
    state::AppState,
    tracker::{
        repository::TrackerIntegration,
        service::{IntegrationService, SuccessResponse, TrackerInfoResponse},
    },
};
use std::sync::Arc;
use tauri::State;
use crate::{TauriSession, require_auth};

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
    body: TrackerIntegration,
) -> Result<SuccessResponse, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    IntegrationService::add_integration(&state, user_id, body).map_err(|e| e.to_string())
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