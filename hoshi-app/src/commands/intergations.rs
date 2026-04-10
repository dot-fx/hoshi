use crate::{require_auth, TauriSession};
use hoshi_core::tracker::types::AddIntegrationRequest;
use hoshi_core::{
    error::CoreError,
    state::AppState,
    tracker::service::TrackerService,
};
use std::sync::Arc;
use tauri::State;
use hoshi_core::tracker::types::{SuccessResponse, TrackerInfoResponse};

#[tauri::command]
pub async fn list_trackers(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Vec<TrackerInfoResponse>, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::list_trackers(&state, user_id).await
}

#[tauri::command]
pub async fn add_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: AddIntegrationRequest,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::add_integration(Arc::clone(&state), user_id, body).await
}

#[tauri::command]
pub async fn remove_integration(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::remove_integration(&state, user_id, &tracker_name).await
}

#[tauri::command]
pub async fn set_sync_enabled(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    tracker_name: String,
    enabled: bool,
) -> Result<SuccessResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    TrackerService::set_sync_enabled(&state, user_id, &tracker_name, enabled).await
}