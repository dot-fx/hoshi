use crate::TauriSession;
use hoshi_core::{
    auth::service::{AuthService, LoginRequest, RegisterRequest},
    state::AppState,
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
) -> Result<Value, String> {
    let payload = LoginRequest { user_id, password };
    let user = AuthService::login(&state, payload).map_err(|e| e.to_string())?;

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
) -> Result<Value, String> {
    let payload = RegisterRequest { username, password };
    let user = AuthService::register(&state, payload).map_err(|e| e.to_string())?;

    let mut guard = session_state.user_id.write().await;
    *guard = Some(user.id.to_string());

    Ok(json!({ "user": user }))
}

#[tauri::command]
pub async fn logout(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<(), String> {
    AuthService::logout(&state).map_err(|e| e.to_string())?;

    let mut guard = session_state.user_id.write().await;
    *guard = None;

    Ok(())
}

#[tauri::command]
pub async fn get_current_profile(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<Value, String> {
    match AuthService::get_active_user(&state) {
        Ok(Some(user)) => {
            let mut guard = session_state.user_id.write().await;
            *guard = Some(user.id.to_string());
            Ok(json!({ "user": user }))
        }
        Ok(None) => Err("No active profile".to_string()),
        Err(e) => Err(e.to_string()),
    }
}