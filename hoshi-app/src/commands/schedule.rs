use hoshi_core::{
    error::CoreError,
    schedule::service::ScheduleService,
    state::AppState,
};
use std::sync::Arc;
use tauri::State;
use hoshi_core::schedule::types::{AiringEntryEnriched, ScheduleWindow};
use crate::{require_auth, TauriSession};

#[tauri::command]
pub async fn get_schedule(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    window: ScheduleWindow,
) -> Result<Vec<AiringEntryEnriched>, CoreError> {
    let user_id = require_auth(&session_state).await?;

    ScheduleService::get_schedule(state.inner().clone(), user_id, window).await
}