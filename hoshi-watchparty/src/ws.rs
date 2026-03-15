use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;

use crate::types::{
    ClientAction, PlaybackStatus, PlaylistItem, ServerEvent, VideoSource, VideoState, ChatMessage,
    now_ms,
};
use crate::manager::WatchPartyManager;

pub async fn handle_socket(
    socket: WebSocket,
    room_id: String,
    user_id: String,
    display_name: String,
    manager: Arc<WatchPartyManager>,
) {
    let room = match manager.get_room(&room_id).await {
        Some(r) => r,
        None => return,
    };

    let is_host = room.host_user_id == user_id;
    let mut rx: broadcast::Receiver<ServerEvent> = room.tx.subscribe();
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Enviar snapshot completo al conectar.
    let snapshot = room.snapshot().await;
    let snapshot_json = serde_json::to_string(&ServerEvent::RoomState(snapshot)).unwrap();
    if ws_tx.send(Message::Text(snapshot_json.into())).await.is_err() {
        return;
    }

    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let json = match serde_json::to_string(&event) {
                        Ok(j) => j,
                        Err(_) => continue,
                    };
                    if ws_tx.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });

    let room_recv = room.clone();
    let manager_recv = manager.clone();
    let user_id_recv = user_id.clone();
    let display_name_recv = display_name.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_rx.next().await {
            let text = match msg {
                Ok(Message::Text(t)) => t,
                Ok(Message::Close(_)) | Err(_) => break,
                _ => continue,
            };

            let action: ClientAction = match serde_json::from_str(&text) {
                Ok(a) => {
                    println!("[WS] parsed action: {:?}", a);
                    a
                },
                Err(e) => {
                    println!("[WS] parse error: {} raw: {}", e, text);
                    continue;
                }
            };

            handle_action(action, &room_recv, is_host, &user_id_recv, &display_name_recv).await;
        }

        on_disconnect(&room_recv, &user_id_recv, is_host, &manager_recv).await;
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

async fn handle_action(
    action: ClientAction,
    room: &crate::types::Room,
    is_host: bool,
    user_id: &str,
    display_name: &str,
) {
    println!("[WS] handle_action is_host={}", is_host);
    match action { ClientAction::Play if is_host => {
            let mut vs = room.video_state.write().await;
            vs.status = PlaybackStatus::Playing;
            vs.updated_at = now_ms();
            room.broadcast(ServerEvent::VideoSync(vs.clone()));
        }

        ClientAction::Pause if is_host => {
            let mut vs = room.video_state.write().await;
            vs.status = PlaybackStatus::Paused;
            vs.updated_at = now_ms();
            room.broadcast(ServerEvent::VideoSync(vs.clone()));
        }

        ClientAction::Seek { position } if is_host => {
            let mut vs = room.video_state.write().await;
            vs.position = position;
            vs.updated_at = now_ms();
            room.broadcast(ServerEvent::VideoSync(vs.clone()));
        }

        ClientAction::AddToQueue(item) if is_host => {
            let no_current = room.current_item.read().await.is_none();
            let queue_empty = room.queue.read().await.is_empty();

            if no_current && queue_empty {
                start_track(room, item).await;
            } else {
                let mut queue = room.queue.write().await;
                queue.push(item);
                room.broadcast(ServerEvent::QueueUpdated(queue.clone()));
            }
        }

        ClientAction::RemoveFromQueue { item_id } if is_host => {
            let mut queue = room.queue.write().await;
            queue.retain(|i| i.id != item_id);
            room.broadcast(ServerEvent::QueueUpdated(queue.clone()));
        }

        ClientAction::ReorderQueue { ordered_ids } if is_host => {
            let mut queue = room.queue.write().await;
            let mut reordered: Vec<PlaylistItem> = Vec::with_capacity(queue.len());
            for id in &ordered_ids {
                if let Some(item) = queue.iter().find(|i| &i.id == id).cloned() {
                    reordered.push(item);
                }
            }
            *queue = reordered;
            room.broadcast(ServerEvent::QueueUpdated(queue.clone()));
        }

        ClientAction::SkipToItem { item_id } if is_host => {
            let mut queue = room.queue.write().await;
            if let Some(pos) = queue.iter().position(|i| i.id == item_id) {
                let item = queue.remove(pos);
                drop(queue);
                start_track(room, item).await;
            }
        }

        ClientAction::SkipNext if is_host => {
            let next = room.queue.write().await.drain(..1).next();
            if let Some(item) = next {
                start_track(room, item).await;
            }
        }

        ClientAction::ResolveSource { source } if is_host => {
            let vs = room.video_state.read().await.clone();
            *room.current_source.write().await = Some(source.clone());
            room.broadcast(ServerEvent::SourceResolved {
                source,
                video_state: vs,
            });
        }

        ClientAction::SendChat { text } => {
            if text.trim().is_empty() || text.len() > 500 {
                return;
            }
            let avatar_url = room.members.read().await
                .get(user_id)
                .and_then(|m| m.avatar_url.clone());
            let msg = ChatMessage::new(
                user_id.to_string(),
                display_name.to_string(),
                avatar_url,
                text.trim().to_string(),
            );
            room.chat_history.write().await.push(msg.clone());
            room.broadcast(ServerEvent::ChatMessage(msg));
        }

        ClientAction::Heartbeat { position, status } => {
            if is_host {
                let mut vs = room.video_state.write().await;
                vs.position = position;
                vs.status = status;
                vs.updated_at = now_ms();
            }
        }

        _ => {}
    }
}

async fn start_track(room: &crate::types::Room, item: PlaylistItem) {
    let new_vs = VideoState {
        position: 0.0,
        status: PlaybackStatus::Paused,
        updated_at: now_ms(),
    };
    *room.current_item.write().await = Some(item.clone());
    *room.current_source.write().await = None;
    *room.video_state.write().await = new_vs.clone();
    room.broadcast(ServerEvent::TrackChanged { item, video_state: new_vs });
}

async fn on_disconnect(
    room: &crate::types::Room,
    user_id: &str,
    is_host: bool,
    manager: &WatchPartyManager,
) {
    if is_host {
        room.broadcast(ServerEvent::RoomClosed {
            reason: "Host disconnected".to_string(),
        });
        manager.remove_room(&room.id).await;
        tracing::info!("[WatchParty] Room {} closed (host left)", room.id);
    } else {
        room.members.write().await.remove(user_id);
        room.broadcast(ServerEvent::MemberLeft { user_id: user_id.to_string() });
        tracing::info!("[WatchParty] Guest {user_id} left room {}", room.id);
    }
}