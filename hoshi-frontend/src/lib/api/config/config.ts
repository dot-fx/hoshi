import { call } from "@/api/client";
import type { AppConfig, GeneralConfig, AnimeConfig, MangaConfig, NovelConfig } from "@/api/config/types";

type ConfigSection = {
    general?: Partial<GeneralConfig>;
    anime?: Partial<AnimeConfig>;
    manga?: Partial<MangaConfig>;
    novel?: Partial<NovelConfig>;
};

export const configApi = {
    getConfig() {
        return call<AppConfig>({
            http:  { path: "config", method: "GET" },
            tauri: { cmd: "get_user_config" },
        });
    },

    patchConfig(patch: ConfigSection) {
        return call<AppConfig>({
            http:  { path: "config", method: "PATCH", body: patch },
            tauri: { cmd: "patch_user_config", args: { patch } },
        });
    },
};