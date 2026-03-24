use std::sync::Arc;
use tauri::State;
use hoshi_core::state::AppState;

#[cfg(feature = "discord-rpc")]
#[tauri::command]
pub async fn set_activity(
    state: State<'_, Arc<AppState>>,
    title: String,
    details: String,
    image_url: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    is_video: bool,
) -> Result<(), String> {
    state.discord_rpc.set_activity(
        &title,
        &details,
        image_url.as_deref(),
        start_time,
        end_time,
        is_video,
    );
    Ok(())
}

#[cfg(feature = "discord-rpc")]
#[tauri::command]
pub async fn clear_activity(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.discord_rpc.clear_activity();
    Ok(())
}