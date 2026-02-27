import type {RegisterRequest, UserInfo} from "@/api/auth/types";
import { authApi } from "$lib/api/auth/auth";
import { usersApi } from "$lib/api/users/users";

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
        } catch (err: any) {
            this.error = err?.message ?? "Login failed";
            throw err;
        } finally {
            this.loading = false;
        }
    }

    async register(data: RegisterRequest) {
        this.loading = true;
        this.error = null;

        try {
            const res = await authApi.register(data);
            this.user = res.user;
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
        }
    }

    async restore() {
        if (this.initialized) return;

        this.loading = true;

        try {
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