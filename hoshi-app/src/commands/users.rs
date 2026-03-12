use crate::{require_auth, TauriSession};
use hoshi_core::{
    state::AppState,
    users::service::{
        ChangePasswordBody, DeleteUserBody, UpdateUserBody, UserPrivate,
        UserPublic, UserService
    }
};
use std::sync::Arc;
use serde::Serialize;
use tauri::State;
use hoshi_core::users::service::UserResponse;

#[derive(Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserResponse>,
}

#[tauri::command]
pub async fn get_all_users(
    state: State<'_, Arc<AppState>>
) -> Result<UsersListResponse, String> {
    let users = UserService::get_all_users(&state).map_err(|e| e.to_string())?;
    Ok(UsersListResponse { users })
}

#[tauri::command]
pub async fn get_user(
    state: State<'_, Arc<AppState>>,
    id: i32,
) -> Result<UserPublic, String> {
    UserService::get_user_public(&state, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_me(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<UserPrivate, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    UserService::get_me(&state, user_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_me(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    updates: UpdateUserBody,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    UserService::update_user(&state, user_id, updates).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_me(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: DeleteUserBody,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    UserService::delete_user(&state, user_id, body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn change_password(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    body: ChangePasswordBody,
) -> Result<bool, String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    UserService::change_password(&state, user_id, body).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upload_avatar(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    data: Vec<u8>,
    content_type: String,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    UserService::upload_avatar(&state, user_id, data, content_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_avatar(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<(), String> {
    let user_id = require_auth(&session_state).await?
        .parse::<i32>().map_err(|_| "Invalid user ID")?;

    UserService::delete_avatar(&state, user_id).map_err(|e| e.to_string())
}