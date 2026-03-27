use hoshi_core::{
    schedule::{
        repository::{AiringEntryEnriched, ScheduleWindow},
        service::ScheduleService,
    },
    state::AppState,
    error::CoreError,
};
use std::sync::Arc;
use tauri::State;
use crate::{TauriSession, require_auth};

#[tauri::command]
pub async fn get_schedule(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    window: ScheduleWindow,
) -> Result<Vec<AiringEntryEnriched>, CoreError> {
    let user_id = require_auth(&session_state).await?;

    ScheduleService::get_schedule(state.inner().clone(), user_id, window).await
}