import { call, isTauri } from "@/api/client";
import type { SearchQuery, SearchResponse, ImageInfo } from "./types";

export const booruApi = {
    search(query: SearchQuery) {
        return call<SearchResponse>({
            http:  { path: "booru/search", method: "GET", params: query },
            tauri: { cmd: "booru_search", args: { params: query } },
        });
    },

    getInfo(id: string, provider: string) {
        return call<ImageInfo>({
            http:  { path: `booru/info/${id}`, method: "GET", params: { provider } },
            tauri: { cmd: "booru_get_info", args: { id, provider } },
        });
    },

    autocomplete(provider: string, q?: string) {
        return call<unknown[]>({
            http:  { path: `booru/${provider}/autocomplete`, method: "GET", params: { q } },
            tauri: { cmd: "booru_autocomplete", args: { provider, q } },
        });
    },

    localImageUrl(provider: string, filename: string): string {
        if (isTauri()) {
            return `booru://local/${provider}/${filename}`;
        }
        return `/api/booru/local/${provider}/${filename}`;
    },
};