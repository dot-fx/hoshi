export interface SavedImage {
    id: string;
    provider: string;
    title: string;
    artist: string;
    tags: string;
    originalLink: string;
    localPath?: string | null;
    createdAt: number;
}

export interface SearchQuery {
    provider?: string;
    q?: string;
    page?: number;
    perPage?: number;
    [key: string]: unknown;
}

export interface SearchResult {
    id: unknown;
    image: unknown;
    type: string;
    tags: unknown;
    title: unknown;
    headers: unknown;
    provider: string;
}

export interface SearchResponse {
    page: number;
    hasNextPage: boolean;
    total: number;
    results: SearchResult[];
}

export interface ImageInfo {
    id: unknown;
    provider: string;
    image?: unknown;
    type?: unknown;
    tags?: unknown;
    title?: unknown;
    artist?: unknown;
    headers?: unknown;
    originalLink?: unknown;
}