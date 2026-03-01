import type { ContentStatus } from "@/api/content/types";

export interface AiringEntry {
    id: number;
    cid: string;
    episode: number;
    airingAt: number;
    // core_metadata
    title: string;
    subtype?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    synopsis?: string | null;
    status?: ContentStatus | null;
    genres: string[];
    tags: string[];
    nsfw: boolean;
    rating?: number | null;
    releaseDate?: string | null;
    endDate?: string | null;
    trailerUrl?: string | null;
    studio?: string | null;
    // usuario
    userStatus?: string | null;
    userProgress?: number | null;
    userScore?: number | null;
}

export interface ScheduleResponse {
    success: boolean;
    data: AiringEntry[];
    total: number;
}

export interface ScheduleQuery {
    daysBack?: number;
    daysAhead?: number;
}