export type ContentType = "anime" | "manga" | "novel";

export type ContentStatus =
    | "Completed"
    | "Ongoing"
    | "Planned"
    | "Cancelled"
    | "Hiatus";

export interface Character {
    name: string;
    role: string;
    actor?: string | null;
    image?: string | null;
}

export interface StaffMember {
    name: string;
    role: string;
    image?: string | null;
}

export interface ContentRelation {
    id?: number | null;
    sourceCid: string;
    targetCid: string;
    relationType: string;
    createdAt: number;
}

export interface ContentUnit {
    id?: number | null;
    cid: string;
    unitNumber: number;
    contentType: string;
    title?: string | null;
    description?: string | null;
    thumbnailUrl?: string | null;
    releasedAt?: string | null;
    duration?: number | null;
    absoluteNumber?: number | null;
    createdAt: number;
}

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
    metadata: unknown;
    nsfw: boolean;
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
    epsOrChapters?: number | null;
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
    characters: Character[];
    staff: StaffMember[];
}

export interface TrackerCandidate {
    trackerName: string;
    trackerId: string;
    title: string;
    coverImage?: string | null;
    score: number;
}

export type ContentWithMappings = CoreMetadata & {
    trackerMappings: TrackerMapping[];
    extensionSources: ExtensionSource[];
    relations: ContentRelation[];
    contentUnits: ContentUnit[];
};

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

export interface LinkTrackerRequest {
    trackerName: string;
    trackerId: string;
}

export interface UpdateExtensionMappingRequest {
    extensionName: string;
    extensionId: string;
    metadata?: unknown;
}

export interface ContentListResponse {
    data: ContentWithMappings[];
    total: number;
    limit: number;
    offset: number;
}

export interface PlayResponse {
    type: "video" | "reader";
    data: unknown;
}

export interface ResolveExtensionResponse {
    data: ContentWithMappings;
    trackerCandidates?: TrackerCandidate[];
    autoLinked: boolean;
}

export interface ExtensionSearchResponse {
    results: unknown;
}