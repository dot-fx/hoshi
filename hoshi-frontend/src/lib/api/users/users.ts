import { api } from "@/api/client";
import type {
    UsersListResponse,
    SingleUserResponse,
    UserPrivate,
    MessageResponse,
    SuccessResponse,
    UpdateUserBody,
    ChangePasswordBody,
    DeleteUserBody,
} from "./types";

export const usersApi = {
    getAll() {
        return api<UsersListResponse>("users");
    },

    getById(id: number) {
        return api<SingleUserResponse>(`users/${id}`);
    },

    getMe() {
        return api<UserPrivate>("me");
    },

    updateMe(body: UpdateUserBody) {
        return api<MessageResponse>("me", {
            method: "PUT",
            body,
        });
    },

    deleteMe(body: DeleteUserBody) {
        return api<SuccessResponse>("me", {
            method: "DELETE",
            body,
        });
    },

    changePassword(body: ChangePasswordBody) {
        return api<MessageResponse>("me/password", {
            method: "PUT",
            body,
        });
    },

    uploadAvatar(file: File) {
        return api<SuccessResponse>("me/avatar", {
            method: "PUT",
            headers: {
                "Content-Type": file.type,
            },
            body: file,
        });
    },

    deleteAvatar() {
        return api<SuccessResponse>("me/avatar", {
            method: "DELETE",
        });
    },
};