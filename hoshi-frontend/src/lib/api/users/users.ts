import { call, isTauri } from "@/api/client";
import type {
    UserResponse,
    UserPublic,
    UserPrivate,
    UpdateUserBody,
    ChangePasswordBody,
    DeleteUserBody,
} from "./types";

export const usersApi = {
    getAll() {
        return call<UserResponse[]>({
            http:  { path: "users", method: "GET" },
            tauri: { cmd: "get_all_users" },
        });
    },

    getById(id: number) {
        return call<UserPublic>({
            http:  { path: `users/${id}`, method: "GET" },
            tauri: { cmd: "get_user", args: { id } },
        });
    },

    getMe() {
        return call<UserPrivate>({
            http:  { path: "me", method: "GET" },
            tauri: { cmd: "get_me" },
        });
    },

    updateMe(body: UpdateUserBody) {
        return call<void>({
            http:  { path: "me", method: "PUT", body },
            tauri: { cmd: "update_me", args: body },
        });
    },

    deleteMe(body: DeleteUserBody) {
        return call<void>({
            http:  { path: "me", method: "DELETE", body },
            tauri: { cmd: "delete_me", args: body },
        });
    },

    changePassword(body: ChangePasswordBody) {
        return call<boolean>({
            http:  { path: "me/password", method: "PUT", body },
            tauri: { cmd: "change_password", args: body },
        });
    },

    async uploadAvatar(file: File) {
        if (isTauri()) {
            const { invoke } = await import("@tauri-apps/api/core");
            const data = Array.from(new Uint8Array(await file.arrayBuffer()));
            return invoke<void>("upload_avatar", { data, contentType: file.type });
        }

        return call<void>({
            http: { path: "me/avatar", method: "PUT", body: file, headers: { "Content-Type": file.type } },
            tauri: { cmd: "upload_avatar" }, // never reached
        });
    },

    deleteAvatar() {
        return call<void>({
            http:  { path: "me/avatar", method: "DELETE" },
            tauri: { cmd: "delete_avatar" },
        });
    },
};