use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{Room, Member, MemberRole};

pub type SharedManager = Arc<WatchPartyManager>;

pub struct WatchPartyManager {
    rooms: RwLock<HashMap<String, Arc<Room>>>,
}

impl WatchPartyManager {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_room(
        &self,
        name: String,
        password: Option<String>,
        host_user_id: String,
        host_display_name: String,
        host_avatar_url: Option<String>,
        public_url: Option<String>,
    ) -> Arc<Room> {
        let room = Room::new(name, password, host_user_id, host_display_name, host_avatar_url, public_url);
        self.rooms.write().await.insert(room.id.clone(), room.clone());
        room
    }

    pub async fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.read().await.get(room_id).cloned()
    }

    pub async fn remove_room(&self, room_id: &str) {
        self.rooms.write().await.remove(room_id);
    }

    pub async fn list_rooms(&self) -> Vec<RoomSummary> {
        self.rooms
            .read()
            .await
            .values()
            .map(|r| RoomSummary {
                id: r.id.clone(),
                name: r.name.clone(),
                has_password: r.password_hash.is_some(),
                public_url: r.public_url.clone(),
            })
            .collect()
    }

    pub async fn join_room(
        &self,
        room_id: &str,
        user_id: String,
        display_name: String,
        password: Option<&str>,
    ) -> Result<Arc<Room>, JoinError> {
        let room = self.get_room(room_id).await.ok_or(JoinError::NotFound)?;

        if !room.check_password(password) {
            return Err(JoinError::WrongPassword);
        }

        let member = Member {
            user_id: user_id.clone(),
            display_name,
            role: MemberRole::Guest,
            avatar_url: None,
        };

        room.members.write().await.insert(user_id, member.clone());
        room.broadcast(crate::types::ServerEvent::MemberJoined(member));

        Ok(room)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JoinError {
    #[error("Room not found")]
    NotFound,
    #[error("Wrong password")]
    WrongPassword,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSummary {
    pub id: String,
    pub name: String,
    pub has_password: bool,
    pub public_url: Option<String>,
}