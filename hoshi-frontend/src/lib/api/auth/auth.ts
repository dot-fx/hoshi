import { call } from "@/api/client";
import type { LoginRequest, RegisterRequest, AuthResponse } from "./types";

export const authApi = {
    login(body: LoginRequest) {
        return call<AuthResponse>({
            tauri: { cmd: "login", args: body },
        });
    },

    register(body: RegisterRequest) {
        return call<AuthResponse>({
            tauri: { cmd: "register", args: body },
        });
    },

    logout() {
        return call<void>({
            tauri: { cmd: "logout" },
        });
    },

    restoreSession(sessionId: string) {
        return call<void>({
            tauri: { cmd: "restore_session", args: { sessionId } },
        });
    },
};