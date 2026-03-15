use std::sync::Arc;
use tauri::State;

use hoshi_watchparty::{WatchPartyServerState, PlaylistItem};
use hoshi_watchparty::routes::issue_token;

use crate::TauriSession;

#[cfg(feature = "watchparty")]
use axum::Router;
#[cfg(feature = "watchparty")]
use crate::router::{static_handler, spa_fallback};

const WATCHPARTY_PORT: u16 = 10090;

#[cfg(feature = "watchparty")]
fn build_spa_router() -> Router {
    Router::new()
        .route("/_app/*file", axum::routing::get(static_handler))
        .route("/robots.txt", axum::routing::get(static_handler))
        .fallback(spa_fallback)
}

#[tauri::command]
pub async fn start_watchparty(
    wp: State<'_, WatchPartyServerState>,
) -> Result<String, String> {
    let addr = wp.start(WATCHPARTY_PORT, build_spa_router()).await.map_err(|e| e.to_string())?;
    Ok(addr.to_string())
}

#[tauri::command]
pub async fn stop_watchparty(
    wp: State<'_, WatchPartyServerState>,
) -> Result<(), String> {
    wp.stop().await;
    Ok(())
}

#[tauri::command]
pub async fn watchparty_status(
    wp: State<'_, WatchPartyServerState>,
) -> Result<bool, String> {
    Ok(wp.is_running().await)
}

#[derive(serde::Deserialize)]
pub struct CreateRoomArgs {
    pub name: String,
    pub password: Option<String>,
    pub public: bool,
}

#[derive(serde::Serialize)]
pub struct CreateRoomResult {
    pub room_id: String,
    pub room_url: String,
    pub public_url: Option<String>,
}

#[tauri::command]
pub async fn create_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    session: State<'_, TauriSession>,
    args: CreateRoomArgs,
) -> Result<CreateRoomResult, String> {
    let user_id = crate::require_auth(&session).await?;

    if !wp.is_running().await {
        wp.start(WATCHPARTY_PORT, build_spa_router()).await.map_err(|e| e.to_string())?;
    }

    if args.name.trim().is_empty() {
        return Err("Room name cannot be empty".to_string());
    }

    let public_url = if args.public {
        match wp.tunnel.open_tunnel(WATCHPARTY_PORT).await {
            Ok(url) => Some(url),
            Err(e) => return Err(e.to_string()),
        }
    } else {
        None
    };

    let host_display_name = user_id.clone();

    let room = wp.manager
        .create_room(
            args.name,
            args.password,
            user_id.clone(),
            host_display_name.clone(),
            public_url.clone(),
        )
        .await;

    Ok(CreateRoomResult {
        room_id: room.id.clone(),
        room_url: format!("/watchparty/{}", room.id),
        public_url,
    })
}

#[tauri::command]
pub async fn delete_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    session: State<'_, TauriSession>,
    room_id: String,
) -> Result<(), String> {
    let user_id = crate::require_auth(&session).await?;

    let room = wp.manager.get_room(&room_id).await
        .ok_or("Room not found")?;

    if room.host_user_id != user_id {
        return Err("Only the host can delete this room".to_string());
    }

    room.broadcast(hoshi_watchparty::ServerEvent::RoomClosed {
        reason: "Room closed by host".to_string(),
    });
    wp.manager.remove_room(&room_id).await;

    if room.public_url.is_some() {
        wp.tunnel.close_tunnel_if_unused().await;
    }

    Ok(())
}

async fn get_host_room(
    wp: &WatchPartyServerState,
    room_id: &str,
    user_id: &str,
) -> Result<Arc<hoshi_watchparty::types::Room>, String> {
    let room = wp.manager.get_room(room_id).await.ok_or("Room not found")?;
    if room.host_user_id != user_id {
        return Err("Only the host can perform this action".to_string());
    }
    Ok(room)
}