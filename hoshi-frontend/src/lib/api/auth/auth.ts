import { api } from "@/api/client";
import type { LoginRequest, RegisterRequest, AuthResponse, SuccessResponse } from "./types";

export const authApi = {
    login(body: LoginRequest) {
        return api<AuthResponse>("login", {
            method: "POST",
            body,
        });
    },

    register(body: RegisterRequest) {
        return api<AuthResponse>("register", {
            method: "POST",
            body,
        });
    },

    logout() {
        return api<SuccessResponse>("logout", {
            method: "POST",
        });
    },
};