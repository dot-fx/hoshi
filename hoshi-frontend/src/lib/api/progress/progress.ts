import { call } from "@/api/client";
import type {
    ContinueWatchingResponse,
    ContentProgressResponse,
    ProgressResponse,
    UpdateAnimeProgressBody,
    UpdateChapterProgressBody,
} from "./types";

export const progressApi = {
    updateAnimeProgress(body: UpdateAnimeProgressBody) {
        return call<ProgressResponse>({
            http:  { path: "progress/anime", method: "POST", body },
            tauri: { cmd: "update_anime_progress", args: { body } },
        });
    },

    updateChapterProgress(body: UpdateChapterProgressBody) {
        return call<ProgressResponse>({
            http:  { path: "progress/chapter", method: "POST", body },
            tauri: { cmd: "update_chapter_progress", args: { body } },
        });
    },

    getContinueWatching(limit?: number) {
        return call<ContinueWatchingResponse>({
            http:  { path: "progress/continue", method: "GET", params: limit ? { limit } : undefined },
            tauri: { cmd: "get_continue_watching", args: { limit: limit ?? null } },
        });
    },

    getContentProgress(cid: string) {
        return call<ContentProgressResponse>({
            http:  { path: `progress/${cid}`, method: "GET" },
            tauri: { cmd: "get_content_progress", args: { cid } },
        });
    },
};