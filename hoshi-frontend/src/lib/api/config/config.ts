import { call } from "@/api/client";
import type {AppConfig} from "@/api/config/types";

export const configApi = {
    getConfig() {
        return call<AppConfig>({
            http:  { path: "config", method: "GET" },
            tauri: { cmd: "get_user_config" },
        });
    },

    // Usamos Partial<AppConfig> para poder enviar solo lo que cambia
    patchConfig(patch: Partial<AppConfig>) {
        return call<AppConfig>({
            http:  { path: "config", method: "PATCH", body: patch },
            tauri: { cmd: "patch_user_config", args: { patch } },
        });
    }
};