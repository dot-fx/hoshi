import type { ClientAction, ServerEvent, PlaybackStatus, PlaylistItem, VideoSource } from "./types";

export interface WatchPartySocketOptions {
    baseUrl: string;
    roomId: string;
    token: string | null;
    onEvent: (event: ServerEvent) => void;
    onClose?: (reason: string) => void;
    onError?: (err: Event) => void;
}

export class WatchPartySocket {
    private ws: WebSocket | null = null;
    private intentionallyClosed = false;

    constructor(private readonly opts: WatchPartySocketOptions) {}

    connect(): this {
        const { baseUrl, roomId, token } = this.opts;
        const url = token
            ? `${baseUrl}/ws/room/${roomId}?token=${encodeURIComponent(token)}`
            : `${baseUrl}/ws/room/${roomId}`;
        this.ws = new WebSocket(url);

        this.ws.onmessage = (msg) => {
            try {
                const event = JSON.parse(msg.data) as ServerEvent;
                this.opts.onEvent(event);
                if (event.event === "room_closed") {
                    this.intentionallyClosed = true;
                    this.opts.onClose?.(event.data.reason);
                }
            } catch {
                console.warn("[WatchParty] Failed to parse server event:", msg.data);
            }
        };

        this.ws.onclose = () => {
            if (!this.intentionallyClosed) {
                this.opts.onClose?.("Connection lost");
            }
        };

        this.ws.onerror = (err) => {
            this.opts.onError?.(err);
        };

        return this;
    }

    send(action: ClientAction): void {
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(action));
        }
    }

    close(): void {
        this.intentionallyClosed = true;
        this.ws?.close();
    }

    play()                     { this.send({ action: "play" }); }
    pause()                    { this.send({ action: "pause" }); }
    seek(position: number)     { this.send({ action: "seek", data: { position } }); }
    skipNext()                 { this.send({ action: "skip_next" }); }
    skipToItem(itemId: string) { this.send({ action: "skip_to_item", data: { itemId } }); }

    resolveSource(source: VideoSource) {
        this.send({ action: "resolve_source", data: { source } });
    }

    addToQueue(item: PlaylistItem) {
        this.send({ action: "add_to_queue", data: item });
    }

    removeFromQueue(itemId: string) {
        this.send({ action: "remove_from_queue", data: { itemId } });
    }

    reorderQueue(orderedIds: string[]) {
        this.send({ action: "reorder_queue", data: { orderedIds } });
    }

    sendChat(text: string) {
        this.send({ action: "send_chat", data: { text } });
    }

    heartbeat(position: number, status: PlaybackStatus) {
        this.send({ action: "heartbeat", data: { position, status } });
    }
}