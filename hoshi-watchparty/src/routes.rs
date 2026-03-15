use std::sync::Arc;
use axum::{
    extract::{Path, Query, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, delete},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::manager::{JoinError, RoomSummary, SharedManager};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub host_display_name: String,
    pub host_avatar_url: Option<String>,
    pub has_password: bool,
}
use crate::tunnel::TunnelManager;
use crate::types::MemberRole;
use crate::ws::handle_socket;


pub type SessionResolver =
Arc<dyn Fn(&str) -> Option<(String, String, Option<String>)> + Send + Sync>;

pub type TokenStore =
Arc<tokio::sync::RwLock<std::collections::HashMap<String, TokenClaims>>>;

#[derive(Clone)]
pub struct TokenClaims {
    pub room_id: String,
    pub user_id: String,
    pub display_name: String,
    pub role: MemberRole,
}

pub fn new_token_store() -> TokenStore {
    Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()))
}

pub async fn issue_token(
    store: &TokenStore,
    room_id: String,
    user_id: String,
    display_name: String,
    role: MemberRole,
) -> String {
    let token = Uuid::new_v4().to_string();
    store.write().await.insert(
        token.clone(),
        TokenClaims { room_id, user_id, display_name, role },
    );
    token
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomRequest {
    pub name: String,
    pub password: Option<String>,
    pub public: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    pub room_id: String,
    pub room_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_token: Option<String>,
    pub public_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomRequest {
    pub display_name: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomResponse {
    pub guest_token: String,
    pub room_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

pub fn watchparty_guest_routes<S>(
    manager: SharedManager,
    token_store: TokenStore,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/rooms",          get(list_rooms))
        .route("/api/rooms/:id",      get(get_room))
        .route("/api/rooms/:id/join", post(join_room))
        .route("/ws/room/:id",        get(ws_upgrade))
        .layer(Extension(token_store))
        .layer(Extension(manager))
}

pub fn watchparty_routes<S>(
    manager: SharedManager,
    session_resolver: SessionResolver,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let token_store = new_token_store();
    let tunnel = Arc::new(TunnelManager::new());

    Router::new()
        .route("/api/rooms",          get(list_rooms).post(create_room))
        .route("/api/rooms/:id",      get(get_room).delete(delete_room))
        .route("/api/rooms/:id/join", post(join_room))
        .route("/ws/room/:id",        get(ws_upgrade))
        .layer(Extension(session_resolver))
        .layer(Extension(tunnel))
        .layer(Extension(token_store))
        .layer(Extension(manager))
}

async fn list_rooms(
    Extension(manager): Extension<SharedManager>,
) -> Json<Vec<RoomSummary>> {
    Json(manager.list_rooms().await)
}

async fn get_room(
    Path(room_id): Path<String>,
    Extension(manager): Extension<SharedManager>,
) -> Result<Json<RoomInfo>, StatusCode> {
    let room = manager.get_room(&room_id).await.ok_or(StatusCode::NOT_FOUND)?;

    let (host_display_name, host_avatar_url) = {
        let members = room.members.read().await;
        let host = members.get(&room.host_user_id);
        (
            host.map(|m| m.display_name.clone())
                .unwrap_or_else(|| room.host_user_id.clone()),
            host.and_then(|m| m.avatar_url.clone()),
        )
    };

    Ok(Json(RoomInfo {
        id: room.id.clone(),
        name: room.name.clone(),
        host_display_name,
        host_avatar_url,
        has_password: room.password_hash.is_some(),
    }))
}

async fn create_room(
    Extension(manager): Extension<SharedManager>,
    Extension(tunnel): Extension<Arc<TunnelManager>>,
    Extension(tokens): Extension<TokenStore>,
    Extension(resolver): Extension<SessionResolver>,
    headers: HeaderMap,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, (StatusCode, Json<Value>)> {
    if req.name.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "Room name cannot be empty" })),
        ));
    }

    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (user_id, display_name, avatar_url) = resolver(cookie).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Not authenticated" })),
    ))?;

    let public_url = if req.public {
        let port: u16 = std::env::var("WATCHPARTY_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10090);

        match tunnel.open_tunnel(port).await {
            Ok(url) => Some(url),
            Err(e) => return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": e.to_string() })),
            )),
        }
    } else {
        None
    };

    let room = manager
        .create_room(req.name, req.password, user_id.clone(), display_name.clone(), avatar_url, public_url.clone())
        .await;

    Ok(Json(CreateRoomResponse {
        room_id: room.id.clone(),
        room_url: format!("/watchparty/{}", room.id),
        host_token: None,
        public_url,
    }))
}

pub async fn create_room_tauri(
    manager: &SharedManager,
    tokens: &TokenStore,
    name: String,
    password: Option<String>,
    public_url: Option<String>,
    user_id: String,
    display_name: String,
) -> Result<CreateRoomResponse, String> {
    if name.trim().is_empty() {
        return Err("Room name cannot be empty".into());
    }

    let room = manager
        .create_room(name, password, user_id.clone(), display_name.clone(), None, public_url.clone())
        .await;

    let host_token = issue_token(
        tokens,
        room.id.clone(),
        user_id,
        display_name,
        MemberRole::Host,
    )
        .await;

    Ok(CreateRoomResponse {
        room_id: room.id.clone(),
        room_url: format!("/watchparty/{}", room.id),
        host_token: Some(host_token),
        public_url,
    })
}

async fn delete_room(
    Path(room_id): Path<String>,
    Extension(manager): Extension<SharedManager>,
    Extension(tunnel): Extension<Arc<TunnelManager>>,
    Extension(resolver): Extension<SessionResolver>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let (user_id, _, _) = resolver(cookie).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "Not authenticated" })),
    ))?;

    let room = manager.get_room(&room_id).await.ok_or((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "Room not found" })),
    ))?;

    if room.host_user_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Only the host can delete this room" })),
        ));
    }

    room.broadcast(crate::types::ServerEvent::RoomClosed {
        reason: "Room closed by host".to_string(),
    });
    manager.remove_room(&room_id).await;

    if room.public_url.is_some() {
        tunnel.close_tunnel_if_unused().await;
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn join_room(
    Path(room_id): Path<String>,
    Extension(manager): Extension<SharedManager>,
    Extension(tokens): Extension<TokenStore>,
    Json(req): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, (StatusCode, Json<Value>)> {
    if req.display_name.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "Display name cannot be empty" })),
        ));
    }

    let guest_user_id = format!("guest_{}", Uuid::new_v4());

    manager
        .join_room(
            &room_id,
            guest_user_id.clone(),
            req.display_name.clone(),
            req.password.as_deref(),
        )
        .await
        .map_err(|e| match e {
            JoinError::NotFound => (StatusCode::NOT_FOUND, Json(json!({ "error": "Room not found" }))),
            JoinError::WrongPassword => (StatusCode::FORBIDDEN, Json(json!({ "error": "Wrong password" }))),
        })?;

    let guest_token = issue_token(
        &tokens,
        room_id.clone(),
        guest_user_id,
        req.display_name,
        MemberRole::Guest,
    )
        .await;

    Ok(Json(JoinRoomResponse { guest_token, room_id }))
}

async fn ws_upgrade(
    Path(room_id): Path<String>,
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
    Extension(manager): Extension<SharedManager>,
    Extension(tokens): Extension<TokenStore>,
    session_resolver: Option<Extension<SessionResolver>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if let Some(Extension(resolver)) = session_resolver {
        let cookie = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if let Some((user_id, display_name, _avatar_url)) = resolver(cookie) {
            let room = match manager.get_room(&room_id).await {
                Some(r) => r,
                None => return (StatusCode::NOT_FOUND, "Room not found").into_response(),
            };
            if room.host_user_id == user_id {
                return ws.on_upgrade(move |socket| {
                    handle_socket(socket, room_id, user_id, display_name, manager)
                });
            }
        }
    }
    
    let claims = {
        let mut store = tokens.write().await;
        query
            .token
            .as_deref()
            .and_then(|t| store.remove(t))
            .filter(|c| c.room_id == room_id)
    };

    let claims = match claims {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response(),
    };

    ws.on_upgrade(move |socket| {
        handle_socket(socket, room_id, claims.user_id, claims.display_name, manager)
    })
}