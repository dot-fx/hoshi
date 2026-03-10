export type ListStatus =
    | "CURRENT"
    | "COMPLETED"
    | "PLANNING"
    | "PAUSED"
    | "DROPPED"
    | "REPEATING";

export interface ListEntry {
    id?: number | null;
    userId: number;
    cid: string;
    status: ListStatus;
    progress: number;
    score?: number | null;
    startDate?: string | null;
    endDate?: string | null;
    repeatCount: number;
    notes?: string | null;
    isPrivate: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface EnrichedListEntry extends ListEntry {
    title: string;
    coverImage?: string | null;
    contentType: string;
    nsfw: boolean;
    totalUnits?: number | null;
    trackerIds: Record<string, string>;
    externalIds: unknown;
    hasExtensionSource: boolean;
}

export interface UserStats {
    totalEntries: number;
    watching: number;
    completed: number;
    planning: number;
    paused: number;
    dropped: number;
    totalEpisodes: number;
    totalChapters: number;
    meanScore?: number | null;
}

export interface UpsertEntryBody {
    cid: string;
    status: ListStatus;
    progress?: number;
    score?: number;
    startDate?: string;
    endDate?: string;
    repeatCount?: number;
    notes?: string;
    isPrivate?: boolean;
}

export interface FilterQuery {
    status?: ListStatus;
    contentType?: string;
}

export interface ListResponse {
    results: EnrichedListEntry[];
}

export interface SingleEntryResponse {
    found: boolean;
    entry?: EnrichedListEntry | null;
}

export interface UpsertEntryResponse {
    changes: number;
    isNew: boolean;
}