import type { RegisterRequest, UserInfo } from "@/api/auth/types";
import { authApi } from "$lib/api/auth/auth";
import { usersApi } from "$lib/api/users/users";
import { appConfig } from "@/config.svelte";

function isTauri(): boolean {
    return typeof window !== "undefined" && "__TAURI__" in window;
}

async function getStore() {
    const { load } = await import("@tauri-apps/plugin-store");
    return load("session.json", { autoSave: true, defaults: {} });
}

async function saveSession(id: string): Promise<void> {
    const store = await getStore();
    await store.set("session_id", id);
}

async function getSession(): Promise<string | null> {
    const store = await getStore();
    return await store.get<string>("session_id") ?? null;
}

async function clearSession(): Promise<void> {
    const store = await getStore();
    await store.delete("session_id");
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
                await saveSession(res.sessionId);
            }
            await appConfig.load();

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
                await saveSession(res.sessionId);
            }

            if (avatarFile) {
                await usersApi.uploadAvatar(avatarFile);
                const updatedUser = await usersApi.getMe();
                this.user = updatedUser;
            }
            await appConfig.load();

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
            appConfig.clear();

            if (isTauri()) {
                await clearSession();
            }
        }
    }

    async restore(force = false) {
        if (this.initialized && !force) return;

        this.loading = true;

        try {
            if (isTauri()) {
                const sessionId = await getSession();
                if (sessionId) {
                    await authApi.restoreSession(sessionId);
                }
            }

            const user = await usersApi.getMe();
            this.user = user;
            await appConfig.load();

        } catch {
            this.user = null;
            appConfig.clear();
        } finally {
            this.loading = false;
            this.initialized = true;
        }
    }
}

export const auth = new AuthStore();