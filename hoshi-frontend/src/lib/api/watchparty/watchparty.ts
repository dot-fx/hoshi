import { call } from "@/api/client";
import type {
    RoomSummary,
    RoomInfo,
    CreateRoomRequest,
    CreateRoomResponse,
    JoinRoomRequest,
    JoinRoomResponse,
} from "./types";

export const watchpartyApi = {
    listRooms() {
        return call<RoomSummary[]>({
            http:  { path: "rooms", method: "GET" },
            tauri: { cmd: "list_watchparty_rooms", args: {} },
        });
    },

    getRoom(roomId: string) {
        return call<RoomInfo>({
            http:  { path: `rooms/${roomId}`, method: "GET" },
            tauri: { cmd: "get_watchparty_room", args: { roomId } },
        });
    },

    joinRoom(roomId: string, req: JoinRoomRequest) {
        return call<JoinRoomResponse>({
            http:  { path: `rooms/${roomId}/join`, method: "POST", body: req },
            tauri: { cmd: "join_watchparty_room", args: { roomId, req } },
        });
    },

    createRoom(req: CreateRoomRequest) {
        return call<CreateRoomResponse>({
            http:  { path: "rooms", method: "POST", body: req },
            tauri: { cmd: "create_watchparty_room", args: { args: req } },
        });
    },

    deleteRoom(roomId: string) {
        return call<void>({
            http:  { path: `rooms/${roomId}`, method: "DELETE" },
            tauri: { cmd: "delete_watchparty_room", args: { roomId } },
        });
    },

    startServer() {
        return call<string>({
            tauri: { cmd: "start_watchparty", args: {} },
        });
    },

    stopServer() {
        return call<void>({
            tauri: { cmd: "stop_watchparty", args: {} },
        });
    },

    serverStatus() {
        return call<boolean>({
            tauri: { cmd: "watchparty_status", args: {} },
        });
    },
};