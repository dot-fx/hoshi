use crate::{require_auth, TauriSession};
use hoshi_core::tracker::repository::AddIntegrationRequest;
use hoshi_core::{
    state::AppState,
    tracker::service::{IntegrationService, SuccessResponse, TrackerInfoResponse},
    error::CoreError,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_trackers(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Vec<TrackerInfoResponse>, CoreError> {
    let user_id = require_auth(&session_state).await?;
    IntegrationService::list_trackers(&state, user_id)
}

#[tauri::command]
pub async fn add_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: AddIntegrationRequest,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    IntegrationService::add_integration(Arc::clone(&state), user_id, body).await
}

#[tauri::command]
pub async fn remove_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    IntegrationService::remove_integration(&state, user_id, &tracker_name)
}

#[tauri::command]
pub async fn set_sync_enabled(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
    enabled: bool,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    IntegrationService::set_sync_enabled(&state, user_id, &tracker_name, enabled)
}