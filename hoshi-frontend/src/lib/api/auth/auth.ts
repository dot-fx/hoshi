import { call } from "@/api/client";
import type { LoginRequest, RegisterRequest, AuthResponse } from "./types";

export const authApi = {
    login(body: LoginRequest) {
        return call<AuthResponse>({
            http:  { path: "login", method: "POST", body },
            tauri: { cmd: "login", args: body },
        });
    },

    register(body: RegisterRequest) {
        return call<AuthResponse>({
            http:  { path: "register", method: "POST", body },
            tauri: { cmd: "register", args: body },
        });
    },

    logout() {
        return call<void>({
            http:  { path: "logout", method: "POST" },
            tauri: { cmd: "logout" },
        });
    },

    restoreSession(sessionId: string) {
        return call<void>({
            http:  { path: "restore", method: "POST", body: { sessionId } },
            tauri: { cmd: "restore_session", args: { sessionId } },
        });
    },
};