import type { SavedImage } from "@/api/booru/types";

export interface Collection {
    id: string;
    userId: number;
    name: string;
    description: string;
    isPrivate: boolean;
    coverId?: string | null;
    createdAt: number;
}

export interface CreateCollectionRequest {
    name: string;
    description?: string;
    isPrivate?: boolean;
}

export interface AddImageToCollectionRequest {
    id: string;
    provider: string;
    title: string;
    artist: string;
    tags?: string;
    originalLink: string;
    imageUrl: string;
    headers?: Record<string, string>;
}

export interface ReorderCollectionRequest {
    orderedIds: string[];
}

export interface CollectionListResponse {
    collections: Collection[];
}

export interface CollectionResponse {
    collection: Collection;
}

export interface CollectionImagesResponse {
    images: SavedImage[];
}

export interface CreateCollectionResponse {
    success: boolean;
    id: string;
}

export interface SuccessResponse {
    success: boolean;
}