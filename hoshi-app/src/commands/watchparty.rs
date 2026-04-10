use std::sync::Arc;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tauri::path::BaseDirectory;
use hoshi_core::state::AppState;
use hoshi_core::users::service::UserService;
use hoshi_core::error::{CoreError, CoreResult};
use hoshi_watchparty::{WatchPartyServerState, PlaylistItem};
use hoshi_watchparty::manager::RoomSummary;
use hoshi_watchparty::routes::{issue_token, RoomInfo};
use tracing::{info, warn, error, instrument};
use crate::TauriSession;

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
) -> Result<String, CoreError> {
    let addr = wp.start(WATCHPARTY_PORT, spa_dir(&app)).await
        .map_err(|e| {
            error!(error = ?e, "Failed to start WatchParty server");
            CoreError::Internal("error.system.internal".into())
        })?;
    Ok(addr.to_string())
}

#[tauri::command]
pub async fn stop_watchparty(
    wp: State<'_, WatchPartyServerState>,
) -> Result<(), CoreError> {
    wp.stop().await;
    Ok(())
}

#[tauri::command]
pub async fn watchparty_status(
    wp: State<'_, WatchPartyServerState>,
) -> Result<bool, CoreError> {
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
    app_state: State<'_, Arc<AppState>>,
    args: crate::commands::watchparty::CreateRoomArgs,
) -> Result<crate::commands::watchparty::CreateRoomResult, CoreError> {
    let user_id_int = crate::require_auth(&session).await?;
    let user_id_str = user_id_int.to_string();

    let user_profile = UserService::get_me(&app_state, user_id_int).await?;

    if !wp.is_running().await {
        wp.start(WATCHPARTY_PORT, spa_dir(&app))
            .await
            .map_err(|e| {
                error!(error = ?e, "Failed to auto-start WatchParty server");
                CoreError::Internal("error.system.internal".into())
            })?;
    }

    if args.name.trim().is_empty() {
        return Err(CoreError::BadRequest("error.watchparty.empty_display_name".into()));
    }

    let public_url = if args.public {
        match wp.tunnel.open_tunnel(WATCHPARTY_PORT).await {
            Ok(url) => Some(url),
            Err(e) => {
                tracing::error!(error = ?e, "Failed to open Cloudflare tunnel");

                return Err(CoreError::Network(e.to_string()));
            }
        }
    } else {
        None
    };

    let host_display_name = user_profile.username;
    let host_avatar_url = user_profile.avatar;

    let room = wp.manager
        .create_room(
            args.name,
            args.password,
            user_id_str.clone(),
            host_display_name.clone(),
            host_avatar_url,
            public_url.clone(),
        )
        .await;

    let host_token = issue_token(
        &wp.token_store,
        room.id.clone(),
        user_id_str,
        host_display_name,
        hoshi_watchparty::types::MemberRole::Host,
    ).await;

    info!(room_id = %room.id, public = args.public, "WatchParty room created successfully");

    Ok(crate::commands::watchparty::CreateRoomResult {
        room_id: room.id.clone(),
        room_url: format!("/watchparty/{}", room.id),
        host_token,
        public_url,
    })
}

#[tauri::command]
pub async fn list_watchparty_rooms(
    wp: State<'_, WatchPartyServerState>,
) -> Result<Vec<RoomSummary>, CoreError> {
    Ok(wp.manager.list_rooms().await)
}

#[tauri::command]
pub async fn get_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    room_id: String,
) -> Result<RoomInfo, CoreError> {
    let room = wp.manager.get_room(&room_id).await
        .ok_or_else(|| CoreError::NotFound("error.watchparty.room_not_found".into()))?;

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

#[tauri::command]
pub async fn join_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    args: crate::commands::watchparty::JoinRoomArgs,
) -> Result<crate::commands::watchparty::JoinRoomResult, CoreError> {
    if args.display_name.trim().is_empty() {
        return Err(CoreError::BadRequest("error.watchparty.empty_display_name".into()));
    }

    let guest_user_id = format!("guest_{}", uuid::Uuid::new_v4());

    wp.manager
        .join_room(&args.room_id, guest_user_id.clone(), args.display_name.clone(), args.password.as_deref())
        .await
        .map_err(|e| match e {
            hoshi_watchparty::manager::JoinError::NotFound => CoreError::NotFound("error.watchparty.room_not_found".into()),
            hoshi_watchparty::manager::JoinError::WrongPassword => CoreError::AuthError("error.watchparty.wrong_password".into()),
        })?;

    let guest_token = issue_token(
        &wp.token_store,
        args.room_id.clone(),
        guest_user_id,
        args.display_name,
        hoshi_watchparty::types::MemberRole::Guest,
    ).await;

    Ok(crate::commands::watchparty::JoinRoomResult { guest_token, room_id: args.room_id })
}

#[tauri::command]
pub async fn delete_watchparty_room(
    wp: State<'_, WatchPartyServerState>,
    session: State<'_, TauriSession>,
    room_id: String,
) -> Result<(), CoreError> {
    let user_id = crate::require_auth(&session).await?.to_string();

    let room = wp.manager.get_room(&room_id).await
        .ok_or_else(|| CoreError::NotFound("error.watchparty.room_not_found".into()))?;

    if room.host_user_id != user_id {
        warn!(user = %user_id, room = %room_id, "Unauthorized attempt to delete room");
        return Err(CoreError::AuthError("error.watchparty.host_only".into()));
    }

    room.broadcast(hoshi_watchparty::types::ServerEvent::RoomClosed {
        reason: "Room closed by host".to_string(),
    });
    wp.manager.remove_room(&room_id).await;

    if room.public_url.is_some() {
        wp.tunnel.close_tunnel_if_unused().await;
    }

    info!(room_id = %room_id, "WatchParty room deleted by host");
    Ok(())
}