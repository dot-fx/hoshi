use std::sync::Arc;
use axum::{
    extract::{Path, Query, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::manager::{JoinError, RoomSummary, SharedManager};
use crate::types::MemberRole;
use crate::ws::handle_socket;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub host_display_name: String,
    pub host_avatar_url: Option<String>,
    pub has_password: bool,
}

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

async fn join_room(
    Path(room_id): Path<String>,
    Extension(manager): Extension<SharedManager>,
    Extension(tokens): Extension<TokenStore>,
    Json(req): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, (StatusCode, Json<String>)> {
    if req.display_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json("error.watchparty.empty_display_name".into()),
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
            JoinError::NotFound => (StatusCode::NOT_FOUND, Json("error.watchparty.room_not_found".into())),
            JoinError::WrongPassword => (StatusCode::FORBIDDEN, Json("error.watchparty.wrong_password".into())),
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
    Extension(manager): Extension<SharedManager>,
    Extension(tokens): Extension<TokenStore>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
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