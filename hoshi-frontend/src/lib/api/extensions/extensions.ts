import { call } from "@/api/client";
import type {
    ExtensionFiltersResponse,
    ExtensionSettingsResponse,
} from "./types";

async function callExtList(path: string, cmd: string): Promise<string[]> {
    const res = await call<{ extensions: string[] }>({
        http: { path, method: "GET" },
        tauri: { cmd }
    });
    return res.extensions ?? [];
}

export const extensionsApi = {
    getAll() {
        return callExtList("extensions", "get_extensions");
    },

    getAnime() {
        return callExtList("extensions/anime", "get_anime_extensions");
    },

    getManga() {
        return callExtList("extensions/manga", "get_manga_extensions");
    },

    getNovel() {
        return callExtList("extensions/novel", "get_novel_extensions");
    },

    getBooru() {
        return callExtList("extensions/booru", "get_booru_extensions");
    },

    getSettings(id: string) {
        return call<ExtensionSettingsResponse>({
            http:  { path: `extensions/${id}/settings`, method: "GET" },
            tauri: { cmd: "get_extension_settings", args: { id } },
        });
    },

    getFilters(name: string) {
        return call<ExtensionFiltersResponse>({
            http:  { path: `extensions/${name}/filters`, method: "GET" },
            tauri: { cmd: "get_extension_filters", args: { name } },
        });
    },
};