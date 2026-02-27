import { api } from "@/api/client";
import type {
    ContentResponse,
    ContentListResponse,
    HomeResponse,
    ItemsResponse,
    PlayResponse,
    SuccessResponse,
    SuccessWithIdResponse,
    ExtensionSearchResponse,
    CreateContentRequest,
    SearchQuery,
    UpdateTrackerMappingRequest,
    UpdateExtensionMappingRequest,
    CoreMetadata,
    TrackerMapping,
    ExtensionSource,
} from "./types";

export const contentApi = {
    
    create(body: CreateContentRequest) {
        return api<ContentResponse>("content", {
            method: "POST",
            body,
        });
    },

    get(cid: string) {
        return api<ContentResponse>(`content/${cid}`);
    },

    update(cid: string, meta: CoreMetadata) {
        return api<ContentResponse>(`content/${cid}`, {
            method: "PUT",
            body: meta,
        });
    },

    getHome() {
        return api<HomeResponse>("content/home");
    },

    search(query: SearchQuery) {
        return api<ContentListResponse>("content/search", { params: query });
    },
    
    getItems(cid: string, extension: string) {
        return api<ItemsResponse>(`content/${cid}/${extension}/items`);
    },

    play(cid: string, extension: string, number: number, opts?: { server?: string; category?: string }) {
        return api<PlayResponse>(`content/${cid}/${extension}/play/${number}`, {
            params: opts,
        });
    },
    
    addTrackerMapping(cid: string, mapping: TrackerMapping) {
        return api<SuccessResponse>(`content/${cid}/trackers`, {
            method: "POST",
            body: mapping,
        });
    },

    updateTrackerMapping(cid: string, body: UpdateTrackerMappingRequest) {
        return api<SuccessResponse>(`content/${cid}/trackers/update`, {
            method: "POST",
            body,
        });
    },

    deleteTrackerMapping(cid: string, trackerName: string) {
        return api<SuccessResponse>(`content/${cid}/trackers/${trackerName}`, {
            method: "DELETE",
        });
    },
    
    addExtensionSource(cid: string, source: ExtensionSource) {
        return api<SuccessWithIdResponse>(`content/${cid}/extensions`, {
            method: "POST",
            body: source,
        });
    },

    updateExtensionMapping(cid: string, body: UpdateExtensionMappingRequest) {
        return api<ContentResponse>(`content/${cid}/extensions/update`, {
            method: "POST",
            body,
        });
    },
    
    resolveByTracker(tracker: string, id: string) {
        return api<ContentResponse>(`content/resolve/tracker/${tracker}/${id}`);
    },

    resolveByExtension(extension: string, id: string) {
        return api<ContentResponse>(`content/resolve/extension/${extension}/${id}`);
    },
    
    searchExtension(extension: string, query: Pick<SearchQuery, "query" | "extensionFilters">) {
        return api<ExtensionSearchResponse>(`extensions/${extension}/search`, {
            params: query,
        });
    },
};