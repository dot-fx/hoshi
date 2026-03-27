use crate::TauriSession;
use hoshi_core::{
    auth::service::{AuthService, LoginRequest, RegisterRequest},
    state::AppState,
    error::CoreError,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn login(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    user_id: i32,
    password: Option<String>,
) -> Result<Value, CoreError> {
    let payload = LoginRequest { user_id, password };
    let user = AuthService::login(&state, payload)?;

    let mut guard = session_state.user_id.write().await;
    *guard = Some(user.id.to_string());

    Ok(json!({ "user": user }))
}

#[tauri::command]
pub async fn register(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    username: String,
    password: Option<String>,
) -> Result<Value, CoreError> {
    let payload = RegisterRequest { username, password };
    let user = AuthService::register(&state, payload)?;

    let mut guard = session_state.user_id.write().await;
    *guard = Some(user.id.to_string());

    Ok(json!({ "user": user }))
}

#[tauri::command]
pub async fn logout(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<(), CoreError> {
    AuthService::logout(&state)?;

    let mut guard = session_state.user_id.write().await;
    *guard = None;

    Ok(())
}

#[tauri::command]
pub async fn get_current_profile(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Value, CoreError> {
    match AuthService::get_active_user(&state) {
        Ok(Some(user)) => {
            let mut guard = session_state.user_id.write().await;
            *guard = Some(user.id.to_string());
            Ok(json!({ "user": user }))
        }
        Ok(None) => Err(CoreError::NotFound("error.auth.no_active_profile".into())),
        Err(e) => Err(e),
    }
}