import { api } from "@/api/client";
import type {ExtensionFiltersResponse, ExtensionSettingsResponse, ExtensionsListResponse} from "@/api/extensions/types";

export const extensionsApi = {
    getAll() {
        return api<ExtensionsListResponse>("extensions");
    },

    getAnime() {
        return api<ExtensionsListResponse>("extensions/anime");
    },

    getManga() {
        return api<ExtensionsListResponse>("extensions/manga");
    },

    getNovel() {
        return api<ExtensionsListResponse>("extensions/novel");
    },

    getBooru() {
        return api<ExtensionsListResponse>("extensions/booru");
    },

    getSettings(id: string) {
        return api<ExtensionSettingsResponse>(`extensions/${id}/settings`);
    },

    getFilters(id: string) {
        return api<ExtensionFiltersResponse>(`extensions/${id}/filters`);
    },
};