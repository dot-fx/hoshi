import { call } from "@/api/client";
import type {
    CollectionListResponse,
    CollectionResponse,
    CollectionImagesResponse,
    CreateCollectionResponse,
    CreateCollectionRequest,
    AddImageToCollectionRequest,
    ReorderCollectionRequest,
} from "./types";

export const collectionsApi = {
    getAll() {
        return call<CollectionListResponse>({
            http:  { path: "collections", method: "GET" },
            tauri: { cmd: "get_collections" },
        });
    },

    get(id: string) {
        return call<CollectionResponse>({
            http:  { path: `collections/${id}`, method: "GET" },
            tauri: { cmd: "get_collection", args: { id } },
        });
    },

    create(payload: CreateCollectionRequest) {
        return call<CreateCollectionResponse>({
            http:  { path: "collections", method: "POST", body: payload },
            tauri: { cmd: "create_collection", args: { payload } },
        });
    },

    update(id: string, payload: CreateCollectionRequest) {
        return call<void>({
            http:  { path: `collections/${id}`, method: "PUT", body: payload },
            tauri: { cmd: "update_collection", args: { id, payload } },
        });
    },

    delete(id: string) {
        return call<void>({
            http:  { path: `collections/${id}`, method: "DELETE" },
            tauri: { cmd: "delete_collection", args: { id } },
        });
    },

    getImages(id: string) {
        return call<CollectionImagesResponse>({
            http:  { path: `collections/${id}/images`, method: "GET" },
            tauri: { cmd: "get_collection_images", args: { id } },
        });
    },

    addImage(id: string, payload: AddImageToCollectionRequest) {
        return call<void>({
            http:  { path: `collections/${id}/images`, method: "POST", body: payload },
            tauri: { cmd: "add_image_to_collection", args: { id, payload } },
        });
    },

    removeImage(id: string, imageId: string) {
        return call<void>({
            http:  { path: `collections/${id}/images/${imageId}`, method: "DELETE" },
            tauri: { cmd: "remove_image_from_collection", args: { id, imageId } },
        });
    },

    reorder(id: string, payload: ReorderCollectionRequest) {
        return call<void>({
            http:  { path: `collections/${id}/reorder`, method: "PUT", body: payload },
            tauri: { cmd: "reorder_collection", args: { id, payload } },
        });
    },
};