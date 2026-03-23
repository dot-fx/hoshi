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
    sourceName: string;
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
    nsfw: boolean;
    language?: string | null;
    createdAt: number;
    updatedAt: number;
}

export interface Content {
    cid: string;
    contentType: ContentType;
    nsfw: boolean;
    createdAt: number;
    updatedAt: number;
}

export interface ContentMetadata {
    id?: number | null;
    cid: string;
    sourceName: string;
    sourceId?: string | null;
    subtype?: string | null;
    title: string;
    altTitles?: string[];
    synopsis?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    epsOrChapters?: number | null;
    status?: ContentStatus | null;
    tags?: string[];
    genres?: string[];
    releaseDate?: string | null;
    endDate?: string | null;
    rating?: number | null;
    trailerUrl?: string | null;
    characters: Character[];
    studio?: string | null;
    staff: StaffMember[];
    externalIds: unknown;
    createdAt: number;
    updatedAt: number;
}

export interface TrackerCandidate {
    trackerName: string;
    trackerId: string;
    title: string;
    coverImage?: string | null;
    score: number;
}

export interface ContentWithMappings {
    content: Content;
    metadata: ContentMetadata[];
    trackerMappings: TrackerMapping[];
    extensionSources: ExtensionSource[];
    relations: ContentRelation[];
    contentUnits: ContentUnit[];
}

export function primaryMetadata(
    content: ContentWithMappings | null | undefined,
    preferredProvider: string = 'anilist'
) {
    if (!content || !content.metadata || content.metadata.length === 0) {
        return undefined;
    }
    const preferred = content.metadata.find(m =>
        m.sourceName.toLowerCase() === preferredProvider.toLowerCase()
    );
    if (preferred) return preferred;

    const anilist = content.metadata.find(m =>
        m.sourceName.toLowerCase() === 'anilist'
    );
    if (anilist) return anilist;

    return content.metadata[0];
}

export interface CreateContentRequest {
    contentType: ContentType;
    nsfw: boolean;
    metadata: ContentMetadata;
    trackerMappings?: TrackerMapping[];
    extensionSources?: ExtensionSource[];
}

export type SearchTracker = "anilist" | "mal" | "kitsu";

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
    /** Tracker to search against. Omit to use the default (anilist). */
    tracker?: SearchTracker;
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

export interface HomeMediaItem {
    cid: string;
    trackerId: string;
    title: string;
    altTitles: string[];
    synopsis?: string | null;
    coverImage?: string | null;
    bannerImage?: string | null;
    contentType: ContentType;
    format?: string | null;
    status?: string | null;
    releaseDate?: string | null;
    endDate?: string | null;
    rating?: number | null;
    genres: string[];
    tags: string[];
    trailerUrl?: string | null;
}

export interface MediaSection {
    trending: HomeMediaItem[];
    topRated: HomeMediaItem[];
    seasonal?: HomeMediaItem[] | null;
}

export interface HomeView {
    anime: MediaSection;
    manga: MediaSection;
    novel: MediaSection;
    cachedAt: number;
}