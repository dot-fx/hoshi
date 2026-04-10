use std::sync::Arc;
use tauri::State;
use hoshi_core::{
    state::AppState,
    error::CoreError,
};
use crate::{require_auth, TauriSession};

#[cfg(feature = "discord-rpc")]
#[tauri::command]
pub async fn set_activity(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    title: String,
    details: String,
    image_url: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    is_video: bool,
    is_nsfw: bool,
) -> Result<(), CoreError> {
    let user_id = require_auth(&session_state).await?;

    state.discord_rpc.set_activity(
        &state,
        user_id,
        &title,
        &details,
        image_url.as_deref(),
        start_time,
        end_time,
        is_video,
        is_nsfw,
    ).await;

    Ok(())
}

#[cfg(feature = "discord-rpc")]
#[tauri::command]
pub async fn clear_activity(state: State<'_, Arc<AppState>>) -> Result<(), CoreError> {
    state.discord_rpc.clear_activity();
    Ok(())
}