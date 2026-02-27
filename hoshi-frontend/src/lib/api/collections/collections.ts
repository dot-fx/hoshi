import { api } from "@/api/client";
import type {
    CollectionListResponse,
    CollectionResponse,
    CollectionImagesResponse,
    CreateCollectionResponse,
    SuccessResponse,
    CreateCollectionRequest,
    AddImageToCollectionRequest,
    ReorderCollectionRequest,
} from "./types";

export const collectionsApi = {
    
    getAll() {
        return api<CollectionListResponse>("collections");
    },

    get(id: string) {
        return api<CollectionResponse>(`collections/${id}`);
    },

    create(body: CreateCollectionRequest) {
        return api<CreateCollectionResponse>("collections", {
            method: "POST",
            body,
        });
    },

    update(id: string, body: CreateCollectionRequest) {
        return api<SuccessResponse>(`collections/${id}`, {
            method: "PUT",
            body,
        });
    },

    delete(id: string) {
        return api<SuccessResponse>(`collections/${id}`, {
            method: "DELETE",
        });
    },
    //import type {} from "./types";

    getImages(id: string) {
        return api<CollectionImagesResponse>(`collections/${id}/images`);
    },

    addImage(id: string, body: AddImageToCollectionRequest) {
        return api<SuccessResponse>(`collections/${id}/images`, {
            method: "POST",
            body,
        });
    },

    removeImage(id: string, imageId: string) {
        return api<SuccessResponse>(`collections/${id}/images/${imageId}`, {
            method: "DELETE",
        });
    },

    reorder(id: string, body: ReorderCollectionRequest) {
        return api<SuccessResponse>(`collections/${id}/reorder`, {
            method: "PUT",
            body,
        });
    },
};