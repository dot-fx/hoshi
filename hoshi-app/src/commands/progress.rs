use hoshi_core::{
    progress::service::{
        ContinueWatchingResponse, ContentProgressResponse, ProgressResponse, ProgressService,
        UpdateAnimeProgressBody, UpdateChapterProgressBody,
    },
    state::AppState,
};
use std::sync::Arc;
use tauri::State;

use crate::{require_auth, TauriSession};

#[tauri::command]
pub async fn update_anime_progress(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpdateAnimeProgressBody,
) -> Result<ProgressResponse, String> {
    let user_id = require_auth(&session_state)
        .await?
        .parse::<i32>()
        .map_err(|_| "Invalid user ID")?;

    ProgressService::update_anime_progress(&state, user_id, body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_chapter_progress(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpdateChapterProgressBody,
) -> Result<ProgressResponse, String> {
    let user_id = require_auth(&session_state)
        .await?
        .parse::<i32>()
        .map_err(|_| "Invalid user ID")?;

    ProgressService::update_chapter_progress(&state, user_id, body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_continue_watching(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    limit: Option<i64>,
) -> Result<ContinueWatchingResponse, String> {
    let user_id = require_auth(&session_state)
        .await?
        .parse::<i32>()
        .map_err(|_| "Invalid user ID")?;

    ProgressService::get_continue_watching(&state, user_id, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_content_progress(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<ContentProgressResponse, String> {
    let user_id = require_auth(&session_state)
        .await?
        .parse::<i32>()
        .map_err(|_| "Invalid user ID")?;

    ProgressService::get_content_progress(&state, user_id, cid)
        .await
        .map_err(|e| e.to_string())
}