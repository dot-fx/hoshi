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
    let (user, session_id) = AuthService::login(&state, payload).map_err(|e| e.to_string())?;

    let mut guard = session_state.user_id.write().await;
    *guard = Some(user.id.to_string());

    Ok(json!({ "user": user, "sessionId": session_id }))
}

#[tauri::command]
pub async fn register(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    username: String,
    password: Option<String>,
) -> Result<Value, String> {
    let payload = RegisterRequest { username, password };
    let (user, session_id) = AuthService::register(&state, payload).map_err(|e| e.to_string())?;

    let mut guard = session_state.user_id.write().await;
    *guard = Some(user.id.to_string());

    Ok(json!({ "user": user, "sessionId": session_id }))
}

#[tauri::command]
pub async fn logout(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
) -> Result<(), String> {
    let mut guard = session_state.user_id.write().await;
    *guard = None;

    Ok(())
}

#[tauri::command]
pub async fn restore_session(
    state: State<'_, Arc<AppState>>,
    session_state: State<'_, TauriSession>,
    // Camelcase en el arg: Tauri convierte sessionId -> session_id automáticamente
    session_id: String,
) -> Result<(), String> {
    match AuthService::get_session(&state, &session_id) {
        Ok(Some(session)) => {
            let mut guard = session_state.user_id.write().await;
            *guard = Some(session.user_id.to_string());
            Ok(())
        }
        _ => Err("Invalid or expired session".to_string()),
    }
}