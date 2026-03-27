use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    pub id: String,
    pub url: String,
    #[serde(alias = "language")]
    pub label: String,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub title: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSource {
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub subtitles: Vec<SubtitleTrack>,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemMetadata {
    pub content_id: String,
    pub series_title: String,
    pub cover_image: Option<String>,
    pub unit_number: f64,
    pub studio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub id: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub metadata: Option<ItemMetadata>,
}

impl PlaylistItem {
    pub fn new(title: String, thumbnail: Option<String>, metadata: ItemMetadata) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            thumbnail,
            metadata: Some(metadata),
        }
    }

    pub fn from_url(title: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            thumbnail: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Buffering,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoState {
    pub position: f64,
    pub status: PlaybackStatus,
    pub updated_at: u64,
}

impl Default for VideoState {
    fn default() -> Self {
        Self {
            position: 0.0,
            status: PlaybackStatus::Paused,
            updated_at: now_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub text: String,
    pub timestamp: u64,
}

impl ChatMessage {
    pub fn new(user_id: String, display_name: String, avatar_url: Option<String>, text: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            display_name,
            avatar_url,
            text,
            timestamp: now_ms(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Host,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub user_id: String,
    pub display_name: String,
    pub role: MemberRole,
    pub avatar_url: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum ServerEvent {
    RoomState(RoomSnapshot),
    MemberJoined(Member),
    MemberLeft { user_id: String },
    VideoSync(VideoState),
    TrackChanged {
        item: PlaylistItem,
        video_state: VideoState,
    },
    SourceResolved {
        source: VideoSource,
        video_state: VideoState,
    },
    QueueUpdated(Vec<PlaylistItem>),
    ChatMessage(ChatMessage),
    RoomClosed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomSnapshot {
    pub room_id: String,
    pub name: String,
    pub has_password: bool,
    pub public_url: Option<String>,
    pub members: Vec<Member>,
    pub current_item: Option<PlaylistItem>,
    pub current_source: Option<VideoSource>,
    pub queue: Vec<PlaylistItem>,
    pub video_state: VideoState,
    pub chat_history: Vec<ChatMessage>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "data", rename_all = "snake_case")]
pub enum ClientAction {
    // Host only
    Play,
    Pause,
    Seek { position: f64 },
    AddToQueue(PlaylistItem),
    RemoveFromQueue {
        #[serde(rename = "itemId")]
        item_id: String
    },
    ReorderQueue {
        #[serde(rename = "orderedIds")]
        ordered_ids: Vec<String>
    },
    SkipToItem {
        #[serde(rename = "itemId")]
        item_id: String
    },
    SkipNext,
    ResolveSource { source: VideoSource },
    SendChat { text: String },
    Heartbeat { position: f64, status: PlaybackStatus },
}

pub struct Room {
    pub id: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub host_user_id: String,
    pub public_url: Option<String>,

    pub members: RwLock<HashMap<String, Member>>,
    pub current_item: RwLock<Option<PlaylistItem>>,
    pub current_source: RwLock<Option<VideoSource>>,
    pub queue: RwLock<Vec<PlaylistItem>>,
    pub video_state: RwLock<VideoState>,
    pub chat_history: RwLock<Vec<ChatMessage>>,

    pub tx: broadcast::Sender<ServerEvent>,
}

impl Room {
    pub fn new(
        name: String,
        password: Option<String>,
        host_user_id: String,
        host_display_name: String,
        host_avatar_url: Option<String>,
        public_url: Option<String>,
    ) -> Arc<Self> {
        let (tx, _) = broadcast::channel(256);
        let password_hash = password.map(|p| bcrypt_hash(&p));
        let host = Member {
            user_id: host_user_id.clone(),
            display_name: host_display_name,
            role: MemberRole::Host,
            avatar_url: host_avatar_url,
        };
        let mut members = HashMap::new();
        members.insert(host_user_id.clone(), host);
        Arc::new(Self {
            id: Uuid::new_v4().to_string(),
            name,
            password_hash,
            host_user_id,
            public_url,
            members: RwLock::new(members),
            current_item: RwLock::new(None),
            current_source: RwLock::new(None),
            queue: RwLock::new(Vec::new()),
            video_state: RwLock::new(VideoState::default()),
            chat_history: RwLock::new(Vec::new()),
            tx,
        })
    }

    pub fn check_password(&self, password: Option<&str>) -> bool {
        match (&self.password_hash, password) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(hash), Some(pw)) => bcrypt_verify(pw, hash),
        }
    }

    pub async fn snapshot(&self) -> RoomSnapshot {
        RoomSnapshot {
            room_id: self.id.clone(),
            name: self.name.clone(),
            has_password: self.password_hash.is_some(),
            public_url: self.public_url.clone(),
            members: self.members.read().await.values().cloned().collect(),
            current_item: self.current_item.read().await.clone(),
            current_source: self.current_source.read().await.clone(),
            queue: self.queue.read().await.clone(),
            video_state: self.video_state.read().await.clone(),
            chat_history: self.chat_history.read().await.clone(),
        }
    }

    pub fn broadcast(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn bcrypt_hash(password: &str) -> String {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).expect("bcrypt hash failed")
}

fn bcrypt_verify(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}