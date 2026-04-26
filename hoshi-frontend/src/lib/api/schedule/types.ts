export interface AiringEntry {
    trackerId: string;
    episode: number;
    airingAt: number;
    title: string;
    titleI18n?: Record<string, string>;
    subtype?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    synopsis?: string | null;
    status?: string | null;
    genres: string[];
    nsfw: boolean;
    rating?: number | null;
    releaseDate?: string | null;
    endDate?: string | null;
    trailerUrl?: string | null;
    studio?: string | null;
    userStatus?: string | null;
    userProgress?: number | null;
    userScore?: number | null;
}

export interface ScheduleQuery {
    daysBack?: number;
    daysAhead?: number;
}

export interface ScheduleResponse {
    success: boolean;
    data: AiringEntry[];
    total: number;
}