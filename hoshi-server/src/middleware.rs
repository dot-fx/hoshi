use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use hoshi_core::{
    state::AppState,
    auth::service::AuthService
};

pub async fn session_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    if !path.starts_with("/api/") {
        return next.run(req).await;
    }

    if path.starts_with("/api/auth/") || path == "/api/users" || path == "/api/register" || path == "/api/login" {
        return next.run(req).await;
    }

    let session_id = req
        .headers()
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .and_then(extract_session_cookie);

    match session_id {
        Some(sid) => {
            match AuthService::get_session(&state, &sid) {
                Ok(Some(session)) => {
                    req.extensions_mut().insert(session.user_id);
                    next.run(req).await
                }
                _ => unauthorized_api(),
            }
        }
        None => unauthorized_api(),
    }
}

fn unauthorized_api() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Unauthorized"
        })),
    ).into_response()
}

pub(crate) fn extract_session_cookie(cookies: &str) -> Option<String> {
    cookies
        .split(';')
        .find_map(|cookie| {
            let mut parts = cookie.trim().splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some("session_id"), Some(value)) => Some(value.to_string()),
                _ => None,
            }
        })
}

pub async fn tunnel_security_middleware(
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Response {
    let is_tunnel = headers.contains_key("cf-connecting-ip") || headers.contains_key("cf-ray");

    if !is_tunnel {
        return next.run(req).await;
    }

    let path = req.uri().path();

    if path.starts_with("/public/")
        || path.starts_with("/assets/")
        || path == "/"
        || path == "/index.html"
    {
        return next.run(req).await;
    }

    if path.starts_with("/room") {
        if let Some(query) = req.uri().query() {
            if let Some(room_id) = extract_query_param(query, "id") {
                tracing::debug!("[Tunnel] Room access request: {}", room_id);
                return next.run(req).await;
            }
        }
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Room ID required" })),
        ).into_response();
    }

    if let Some(captures) = extract_path_param(path, r"^/ws/room/([a-f0-9]+)") {
        let room_id = &captures[0];
        tracing::debug!("[Tunnel] WebSocket room access: {}", room_id);
        return next.run(req).await;
    }

    if let Some(captures) = extract_path_param(path, r"^/api/rooms/([a-f0-9]+)") {
        let room_id = &captures[0];
        tracing::debug!("[Tunnel] API room access: {}", room_id);
        return next.run(req).await;
    }

    let allowed_endpoints = [
        "/api/watch/stream",
        "/api/proxy",
        "/api/extensions",
        "/api/search",
    ];

    for endpoint in &allowed_endpoints {
        if path.starts_with(endpoint) {
            tracing::info!("[Tunnel] ✓ Allowing utility endpoint: {}", endpoint);
            return next.run(req).await;
        }
    }

    tracing::warn!("[Tunnel] ✗ Denied access to: {}", path);
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Not found" })),
    ).into_response()
}

fn extract_query_param(query: &str, param: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == param {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn extract_path_param(path: &str, pattern: &str) -> Option<Vec<String>> {
    let re = regex::Regex::new(pattern).ok()?;
    let captures = re.captures(path)?;

    let mut results = Vec::new();
    for i in 1..captures.len() {
        if let Some(m) = captures.get(i) {
            results.push(m.as_str().to_string());
        }
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}