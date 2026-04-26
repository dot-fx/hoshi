import type {ContentUnit} from "@/api/content/types";

export type ContentType = "anime" | "manga" | "novel";

export interface AnimeProgress {
    id: number;
    userId: number;
    cid: string;
    episode: number;
    timestampSeconds: number;
    episodeDurationSeconds?: number | null;
    completed: boolean;
    lastAccessed: number;
}

export interface ChapterProgress {
    id: number;
    userId: number;
    cid: string;
    chapter: number;
    completed: boolean;
    lastAccessed: number;
}

export interface UpdateAnimeProgressBody {
    cid: string;
    episode: number;
    timestampSeconds: number;
    episodeDurationSeconds?: number;
    completed?: boolean;
}

export interface UpdateChapterProgressBody {
    cid: string;
    chapter: number;
    completed?: boolean;
}

export interface ContinueItem {
    cid: string;
    contentType: ContentType;
    title: string;
    titleI18n: Record<string, string>;
    coverImage?: string | null;
    nsfw: boolean;
    episode?: number | null;
    timestampSeconds?: number | null;
    episodeDurationSeconds?: number | null;
    chapter?: number | null;
    unit?: ContentUnit | null;
    lastAccessed: number;
}

export interface ContinueWatchingResponse {
    items: ContinueItem[];
}

export interface ProgressResponse {
    success: boolean;
}

export interface ContentProgressResponse {
    cid: string;
    animeProgress: AnimeProgress[];
    chapterProgress: ChapterProgress[];
}