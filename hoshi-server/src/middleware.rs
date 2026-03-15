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
    auth::service::AuthService,
};

pub async fn session_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    if !path.starts_with("/api/") && !path.starts_with("/ws/") {
        return next.run(req).await;
    }

    let is_public = path.starts_with("/api/auth/")
        || path == "/api/users"
        || path == "/api/register"
        || path == "/api/login"
        // Watchparty — rutas públicas (guests sin sesión)
        || (path == "/api/rooms" && method == axum::http::Method::GET)
        || (matches_room_id(&path) && method == axum::http::Method::GET)
        || (path.starts_with("/api/rooms/") && path.ends_with("/join") && method == axum::http::Method::POST)
        || path.starts_with("/ws/room/")
        || path.starts_with("/api/proxy");

    if is_public {
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
        Json(json!({ "error": "Unauthorized" })),
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

/// Restricts tunnel (Cloudflare) traffic to watchparty-only endpoints.
///
/// Allowed through tunnel:
///   GET  /api/rooms                  — list rooms (guests need to find the room)
///   GET  /api/rooms/:id              — room info (guests need to see room details)
///   POST /api/rooms/:id/join         — guest join (get token)
///   GET  /ws/room/:id?token=...      — WebSocket upgrade (the whole point)
///
/// Everything else is blocked with 404 — not 403, to avoid leaking
/// that other endpoints exist at all.
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
    let method = req.method().clone();

    // WS upgrade — the core reason the tunnel exists
    if path.starts_with("/ws/room/") {
        tracing::debug!("[Tunnel] WS upgrade: {path}");
        return next.run(req).await;
    }

    // Guest join — needs to work before WS
    if path.starts_with("/api/rooms/") && path.ends_with("/join") && method == axum::http::Method::POST {
        tracing::debug!("[Tunnel] Guest join: {path}");
        return next.run(req).await;
    }

    // Room listing and info — read-only, safe to expose
    if path == "/api/rooms" && method == axum::http::Method::GET {
        tracing::debug!("[Tunnel] Room list");
        return next.run(req).await;
    }

    if matches_room_id(path) && method == axum::http::Method::GET {
        tracing::debug!("[Tunnel] Room info: {path}");
        return next.run(req).await;
    }

    tracing::warn!("[Tunnel] Blocked: {method} {path}");
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Not found" })),
    ).into_response()
}

/// Returns true if path matches `/api/rooms/<id>` exactly (no sub-paths).
fn matches_room_id(path: &str) -> bool {
    let rest = match path.strip_prefix("/api/rooms/") {
        Some(r) => r,
        None => return false,
    };
    // Must be a single path segment with no further slashes
    !rest.is_empty() && !rest.contains('/')
}