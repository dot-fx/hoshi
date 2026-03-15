use std::sync::Arc;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tauri::path::BaseDirectory;

use hoshi_watchparty::{WatchPartyServerState, PlaylistItem};
use hoshi_watchparty::manager::RoomSummary;
use hoshi_watchparty::routes::{issue_token, RoomInfo};

use crate::TauriSession;

const WATCHPARTY_PORT: u16 = 10090;

fn spa_dir(app: &AppHandle) -> PathBuf {
    #[cfg(debug_assertions)]
    { PathBuf::from("../hoshi-frontend/build") }

    #[cfg(not(debug_assertions))]
    {
        app.path()
            .resolve(".", BaseDirectory::Resource)
            .unwrap_or_else(|_| PathBuf::from("."))
    }
}

#[tauri::command]
pub async fn start_watchparty(
    app: AppHandle,
    wp: State<'_, WatchPartyServerState>,
) -> Result<String, String> {
    let addr = wp.start(WATCHPARTY_PORT, spa_dir(&app)).await.map_err(|e| e.to_string())?;
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
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResult {
    pub room_id: String,
    pub room_url: String,
    pub host_token: String,
    pub public_url: Option<String>,
}

#[tauri::command]
pub async fn create_watchparty_room(
    app: AppHandle,
    wp: State<'_, WatchPartyServerState>,
    session: State<'_, TauriSession>,
    args: CreateRoomArgs,
) -> Result<CreateRoomResult, String> {
    let user_id = crate::require_auth(&session).await?;

    if !wp.is_running().await {
        wp.start(WATCHPARTY_PORT, spa_dir(&app)).await.map_err(|e| e.to_string())?;
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

    // TODO: obtener display_name y avatar del perfil de usuario en Tauri
    let host_display_name = user_id.clone();
    let host_avatar_url: Option<String> = None;

    let room = wp.manager
        .create_room(
            args.name,
            args.password,
            user_id.clone(),
            host_display_name.clone(),
            host_avatar_url,
            public_url.clone(),
        )
        .await;

    let host_token = issue_token(
        &wp.token_store,
        room.id.clone(),
        user_id,
        host_display_name,
        hoshi_watchparty::types::MemberRole::Host,
    ).await;

    Ok(CreateRoomResult {
        room_id: room.id.clone(),
        room_url: format!("/watchparty/{}", room.id),
        host_token,
        public_url,
    })
}

#[tauri::command]
pub async fn list_watchparty_rooms(
    wp: State<'_, WatchPartyServerState>,
) -> Result<Vec<RoomSummary>, String> {
    Ok(wp.manager.list_rooms().await)
}

#[tauri::command]
pub async fn get_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    room_id: String,
) -> Result<RoomInfo, String> {
    let room = wp.manager.get_room(&room_id).await.ok_or("Room not found")?;
    let (host_display_name, host_avatar_url) = {
        let members = room.members.read().await;
        let host = members.get(&room.host_user_id);
        (
            host.map(|m| m.display_name.clone()).unwrap_or_else(|| room.host_user_id.clone()),
            host.and_then(|m| m.avatar_url.clone()),
        )
    };
    Ok(RoomInfo {
        id: room.id.clone(),
        name: room.name.clone(),
        host_display_name,
        host_avatar_url,
        has_password: room.password_hash.is_some(),
    })
}

#[derive(serde::Deserialize)]
pub struct JoinRoomArgs {
    pub room_id: String,
    pub display_name: String,
    pub password: Option<String>,
}

#[derive(serde::Serialize)]
pub struct JoinRoomResult {
    pub guest_token: String,
    pub room_id: String,
}

#[tauri::command]
pub async fn join_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    args: JoinRoomArgs,
) -> Result<JoinRoomResult, String> {
    if args.display_name.trim().is_empty() {
        return Err("Display name cannot be empty".to_string());
    }

    let guest_user_id = format!("guest_{}", uuid::Uuid::new_v4());

    wp.manager
        .join_room(&args.room_id, guest_user_id.clone(), args.display_name.clone(), args.password.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let guest_token = issue_token(
        &wp.token_store,
        args.room_id.clone(),
        guest_user_id,
        args.display_name,
        hoshi_watchparty::types::MemberRole::Guest,
    ).await;

    Ok(JoinRoomResult { guest_token, room_id: args.room_id })
}

#[tauri::command]
pub async fn delete_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    session: State<'_, TauriSession>,
    room_id: String,
) -> Result<(), String> {
    let user_id = crate::require_auth(&session).await?;

    let room = wp.manager.get_room(&room_id).await.ok_or("Room not found")?;

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