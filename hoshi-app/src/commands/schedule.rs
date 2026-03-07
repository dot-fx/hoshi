use hoshi_core::{
    schedule::{
        repository::{AiringEntryEnriched, ScheduleWindow},
        service::ScheduleService,
    },
    state::AppState,
};
use std::sync::Arc;
use tauri::State;
use crate::{TauriSession, require_auth};

#[tauri::command]
pub async fn get_schedule(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    window: ScheduleWindow,
) -> Result<Vec<AiringEntryEnriched>, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    ScheduleService::get_schedule(state.inner().clone(), user_id, window)
        .await
        .map_err(|e| e.to_string())
}