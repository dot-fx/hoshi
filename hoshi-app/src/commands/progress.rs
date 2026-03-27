use hoshi_core::{
    progress::service::{
        ContinueWatchingResponse, ContentProgressResponse, ProgressResponse, ProgressService,
        UpdateAnimeProgressBody, UpdateChapterProgressBody,
    },
    state::AppState,
    error::CoreError,
};
use std::sync::Arc;
use tauri::State;

use crate::{require_auth, TauriSession};

#[tauri::command]
pub async fn update_anime_progress(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpdateAnimeProgressBody,
) -> Result<ProgressResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ProgressService::update_anime_progress(&state, user_id, body).await
}

#[tauri::command]
pub async fn update_chapter_progress(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: UpdateChapterProgressBody,
) -> Result<ProgressResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ProgressService::update_chapter_progress(&state, user_id, body).await
}

#[tauri::command]
pub async fn get_continue_watching(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    limit: Option<i64>,
) -> Result<ContinueWatchingResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ProgressService::get_continue_watching(&state, user_id, limit).await
}

#[tauri::command]
pub async fn get_content_progress(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    cid: String,
) -> Result<ContentProgressResponse, CoreError> {
    let user_id = require_auth(&session_state).await?;
    ProgressService::get_content_progress(&state, user_id, cid).await
}