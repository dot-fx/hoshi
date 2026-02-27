import { api } from "@/api/client";
import type { SearchQuery, SearchResponse, ImageInfo } from "./types";

export const booruApi = {
    search(query: SearchQuery) {
        return api<SearchResponse>("booru/search", { params: query });
    },

    getInfo(id: string, provider: string) {
        return api<ImageInfo>(`booru/info/${id}`, {
            params: { provider },
        });
    },

    autocomplete(provider: string, q?: string) {
        return api<unknown[]>(`booru/${provider}/autocomplete`, {
            params: { q },
        });
    },

    localImageUrl(provider: string, filename: string): string {
        return `/api/booru/local/${provider}/${filename}`;
    },
};