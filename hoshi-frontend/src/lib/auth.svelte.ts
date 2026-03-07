import type { RegisterRequest, UserInfo } from "@/api/auth/types";
import { authApi } from "$lib/api/auth/auth";
import { usersApi } from "$lib/api/users/users";

function isTauri(): boolean {
    return typeof window !== "undefined" && "__TAURI__" in window;
}

class AuthStore {
    user = $state<UserInfo | null>(null);
    loading = $state(false);
    initialized = $state(false);
    error = $state<string | null>(null);

    isAuthenticated = $derived(this.user !== null);

    async login(userId: number, password?: string) {
        this.loading = true;
        this.error = null;

        try {
            const res = await authApi.login({ userId, password });
            this.user = res.user;

            if (isTauri() && res.sessionId) {
                localStorage.setItem("session_id", res.sessionId);
            }
        } catch (err: any) {
            this.error = err?.message ?? "Login failed";
            throw err;
        } finally {
            this.loading = false;
        }
    }

    async register(data: RegisterRequest, avatarFile?: File | null) {
        this.loading = true;
        this.error = null;

        try {
            const res = await authApi.register(data);
            this.user = res.user;

            if (isTauri() && res.sessionId) {
                localStorage.setItem("session_id", res.sessionId);
            }

            if (avatarFile) {
                await usersApi.uploadAvatar(avatarFile);
                const updatedUser = await usersApi.getMe();
                this.user = updatedUser;
            }

            return res;
        } catch (err: any) {
            this.error = err?.message ?? "Register failed";
            throw err;
        } finally {
            this.loading = false;
        }
    }

    async logout() {
        try {
            await authApi.logout();
        } finally {
            this.user = null;
            if (isTauri()) {
                localStorage.removeItem("session_id");
            }
        }
    }

    async restore(force = false) {
        if (this.initialized && !force) return;

        this.loading = true;

        try {
            if (isTauri()) {
                const sessionId = localStorage.getItem("session_id");
                if (sessionId) {
                    await authApi.restoreSession(sessionId);
                }
            }

            const user = await usersApi.getMe();
            this.user = user;
        } catch {
            this.user = null;
        } finally {
            this.loading = false;
            this.initialized = true;
        }
    }
}

export const auth = new AuthStore();