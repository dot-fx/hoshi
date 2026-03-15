export interface SubtitleTrack {
    label: string;
    url: string;
    language?: string | null;
}

export interface Chapter {
    title: string;
    start: number; // seconds
}

export interface VideoSource {
    url: string;
    headers: Record<string, string>;
    subtitles: SubtitleTrack[];
    chapters: Chapter[];
}

export interface ItemMetadata {
    contentId: string;
    seriesTitle: string;
    coverImage?: string | null;
    unitNumber: number;
    studio?: string | null;
}

export interface PlaylistItem {
    id: string;
    title: string;
    thumbnail?: string | null;
    metadata?: ItemMetadata | null;
}

export type PlaybackStatus = "playing" | "paused" | "buffering" | "ended";

export interface VideoState {
    position: number;
    status: PlaybackStatus;
    updatedAt: number;
}

export type MemberRole = "host" | "guest";

export interface Member {
    userId: string;
    displayName: string;
    role: MemberRole;
    avatarUrl?: string | null;
}

export interface ChatMessage {
    id: string;
    userId: string;
    displayName: string;
    avatarUrl?: string | null;
    text: string;
    timestamp: number;
}

export interface RoomSummary {
    id: string;
    name: string;
    hasPassword: boolean;
    publicUrl?: string | null;
}

export interface RoomInfo {
    id: string;
    name: string;
    hostDisplayName: string;
    hostAvatarUrl?: string | null;
    hasPassword: boolean;
}

export interface RoomSnapshot {
    roomId: string;
    name: string;
    hasPassword: boolean;
    publicUrl?: string | null;
    members: Member[];
    currentItem?: PlaylistItem | null;
    currentSource?: VideoSource | null;
    queue: PlaylistItem[];
    videoState: VideoState;
    chatHistory: ChatMessage[];
}

export interface CreateRoomRequest {
    name: string;
    password?: string;
    public: boolean;
}

export interface CreateRoomResponse {
    roomId: string;
    roomUrl: string;
    hostToken?: string | null;
    publicUrl?: string | null;
}

export interface JoinRoomRequest {
    displayName: string;
    password?: string;
}

export interface JoinRoomResponse {
    guestToken: string;
    roomId: string;
}


export type ServerEvent =
    | { event: "room_state";      data: RoomSnapshot }
    | { event: "member_joined";   data: Member }
    | { event: "member_left";     data: { userId: string } }
    | { event: "video_sync";      data: VideoState }
    | { event: "track_changed";   data: { item: PlaylistItem; videoState: VideoState } }
    | { event: "source_resolved"; data: { source: VideoSource; videoState: VideoState } }
    | { event: "queue_updated";   data: PlaylistItem[] }
    | { event: "chat_message";    data: ChatMessage }
    | { event: "room_closed";     data: { reason: string } };


export type ClientAction =
    | { action: "play" }
    | { action: "pause" }
    | { action: "seek";              data: { position: number } }
    | { action: "add_to_queue";      data: PlaylistItem }
    | { action: "remove_from_queue"; data: { itemId: string } }
    | { action: "reorder_queue";     data: { orderedIds: string[] } }
    | { action: "skip_to_item";      data: { itemId: string } }
    | { action: "skip_next" }
    | { action: "resolve_source";    data: VideoSource }
    | { action: "send_chat";  data: { text: string } }
    | { action: "heartbeat";  data: { position: number; status: PlaybackStatus } };


export interface CreateRoomArgs {
    name: string;
    password?: string;
    public: boolean;
}

export interface CreateRoomResult {
    roomId: string;
    hostToken: string;
    publicUrl?: string | null;
}