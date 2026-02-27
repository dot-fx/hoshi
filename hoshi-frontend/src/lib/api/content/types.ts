export type ContentType = "anime" | "manga" | "novel" | "booru";

export type ContentStatus =
    | "Completed"
    | "Ongoing"
    | "Planned"
    | "Cancelled"
    | "Hiatus";

export interface TrackerMapping {
    cid: string;
    trackerName: string;
    trackerId: string;
    trackerUrl?: string | null;
    syncEnabled: boolean;
    lastSynced?: number | null;
    createdAt: number;
    updatedAt: number;
}

export interface ExtensionSource {
    id?: number | null;
    cid: string;
    extensionName: string;
    extensionId: string;
    contentUrl?: string | null;
    streamUrl?: string | null;
    readUrl?: string | null;
    downloadUrl?: string | null;
    metadata: unknown;
    nsfw: boolean;
    quality?: string | null;
    language?: string | null;
    createdAt: number;
    updatedAt: number;
}

export interface CoreMetadata {
    cid: string;
    contentType: ContentType;
    subtype?: string | null;
    title: string;
    altTitles?: string[];
    synopsis?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    status?: ContentStatus | null;
    tags?: string[];
    genres?: string[];
    nsfw: boolean;
    releaseDate?: string | null;
    endDate?: string | null;
    rating?: number | null;
    trailerUrl?: string | null;
    studio?: string | null;
    sources?: string | null;
    externalIds: unknown;
    createdAt: number;
    updatedAt: number;
}

export interface ContentWithMappings {
    metadata: CoreMetadata;
    trackerMappings: TrackerMapping[];
    extensionSources: ExtensionSource[];
}

export interface CreateContentRequest {
    content: CoreMetadata;
    trackerMappings?: TrackerMapping[];
    extensionSources?: ExtensionSource[];
}

export interface SearchQuery {
    type?: ContentType;
    nsfw?: boolean;
    status?: string;
    query?: string;
    limit?: number;
    offset?: number;
    extension?: string;
    sort?: string;
    genre?: string;
    format?: string;
    extensionFilters?: string;
}

export interface UpdateTrackerMappingRequest {
    trackerName: string;
    trackerId: string;
}

export interface UpdateExtensionMappingRequest {
    extensionName: string;
    extensionId: string;
    metadata?: unknown;
}

export interface ContentResponse {
    success: boolean;
    data: ContentWithMappings;
}

export interface ContentListResponse {
    success: boolean;
    data: ContentWithMappings[];
    total: number;
    limit: number;
    offset: number;
}

export interface HomeResponse {
    success: boolean;
    data: Record<string, unknown[]>;
}

export interface ItemsResponse {
    success: boolean;
    data: unknown;
}

export interface PlayResponse {
    success: boolean;
    type: "video" | "reader";
    data: unknown;
}

export interface SuccessResponse {
    success: boolean;
}

export interface SuccessWithIdResponse {
    success: boolean;
    id: number;
}

export interface ExtensionSearchResponse {
    success: boolean;
    results: unknown;
}