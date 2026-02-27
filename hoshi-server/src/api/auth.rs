use crate::error::AppResult;
use crate::middleware::extract_session_cookie;
use axum::{extract::State, http::{header::{HeaderMap, SET_COOKIE}, StatusCode}, response::{IntoResponse, Response}, routing::post, Json, Router};
use hoshi_core::{
    auth::service::{AuthResponse, AuthService, LoginRequest, RegisterRequest},
    state::AppState
};
use serde_json::json;
use std::sync::Arc;

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Response> {

    let (user, session_id) =
        AuthService::login(&state, payload)?;

    let cookie = build_cookie(&session_id, 60 * 60 * 24 * 7);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((
        headers,
        Json(AuthResponse { success: true, user }),
    ).into_response())
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Response> {

    let (user, session_id) =
        AuthService::register(&state, payload)?;

    let cookie = build_cookie(&session_id, 60 * 60 * 24 * 7);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((
        StatusCode::CREATED,
        headers,
        Json(AuthResponse { success: true, user }),
    ).into_response())
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> AppResult<Response> {
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookies) = cookie_header.to_str() {
            if let Some(session_id) = extract_session_cookie(cookies) {
                AuthService::logout(&state, &session_id)?;
            }
        }
    }

    let cookie = build_cookie("", 0);

    let mut response_headers = HeaderMap::new();
    response_headers.insert(SET_COOKIE, cookie.parse().unwrap());

    Ok((
        response_headers,
        Json(json!({ "success": true })),
    ).into_response())
}

fn build_cookie(value: &str, max_age: u64) -> String {
    format!(
        "session_id={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        value, max_age
    )
}