import { call } from "@/api/client";
import type {
    ContentWithMappings,
    ContentListResponse,
    PlayResponse,
    ResolveExtensionResponse,
    ExtensionSearchResponse,
    CreateContentRequest,
    SearchQuery,
    UpdateTrackerMappingRequest,
    UpdateExtensionMappingRequest,
    ContentMetadata,
    TrackerMapping,
    ExtensionSource,
    LinkTrackerRequest,
} from "./types";

export const contentApi = {

    getHome() {
        return call<unknown>({
            http:  { path: "content/home", method: "GET" },
            tauri: { cmd: "get_home_content" },
        });
    },

    create(req: CreateContentRequest) {
        return call<ContentWithMappings>({
            http:  { path: "content", method: "POST", body: req },
            tauri: { cmd: "create_content", args: { req } },
        });
    },

    get(cid: string) {
        return call<ContentWithMappings>({
            http:  { path: `content/${cid}`, method: "GET" },
            tauri: { cmd: "get_content", args: { cid } },
        });
    },

    update(cid: string, meta: ContentMetadata) {
        return call<ContentWithMappings>({
            http:  { path: `content/${cid}`, method: "PUT", body: meta },
            tauri: { cmd: "update_content", args: { cid, meta } },
        });
    },

    search(query: SearchQuery) {
        return call<ContentListResponse>({
            http:  { path: "content/search", method: "GET", params: query as Record<string, unknown> },
            tauri: { cmd: "search_content", args: { query } },
        });
    },

    getItems(cid: string, extName: string) {
        return call<unknown>({
            http:  { path: `content/${cid}/${extName}/items`, method: "GET" },
            tauri: { cmd: "get_content_items", args: { cid, ext_name: extName } },
        });
    },

    play(cid: string, extName: string, number: number, opts?: { server?: string; category?: string }) {
        return call<PlayResponse>({
            http:  { path: `content/${cid}/${extName}/play/${number}`, method: "GET", params: opts },
            tauri: { cmd: "play_content_by_number", args: { cid, extName, number, ...opts } },
        });
    },

    addTrackerMapping(cid: string, mapping: TrackerMapping) {
        return call<void>({
            http:  { path: `content/${cid}/trackers`, method: "POST", body: mapping },
            tauri: { cmd: "add_tracker_mapping", args: { cid, mapping } },
        });
    },

    updateTrackerMapping(cid: string, req: UpdateTrackerMappingRequest) {
        return call<void>({
            http:  { path: `content/${cid}/trackers/update`, method: "POST", body: req },
            tauri: { cmd: "update_tracker_mapping", args: { cid, req } },
        });
    },

    deleteTrackerMapping(cid: string, trackerName: string) {
        return call<void>({
            http:  { path: `content/${cid}/trackers/${trackerName}`, method: "DELETE" },
            tauri: { cmd: "delete_tracker_mapping", args: { cid, tracker_name: trackerName } },
        });
    },

    addExtensionSource(cid: string, source: ExtensionSource) {
        return call<number>({
            http:  { path: `content/${cid}/extensions`, method: "POST", body: source },
            tauri: { cmd: "add_extension_source", args: { cid, source } },
        });
    },

    updateExtensionMapping(cid: string, req: UpdateExtensionMappingRequest) {
        return call<ContentWithMappings>({
            http:  { path: `content/${cid}/extensions/update`, method: "POST", body: req },
            tauri: { cmd: "update_extension_mapping", args: { cid, req } },
        });
    },

    resolveByTracker(tracker: string, id: string) {
        return call<ContentWithMappings>({
            http:  { path: `content/resolve/tracker/${tracker}/${id}`, method: "GET" },
            tauri: { cmd: "resolve_by_tracker", args: { tracker, id } },
        });
    },

    resolveByExtension(extName: string, extId: string) {
        return call<ContentWithMappings>({
            http:  { path: `content/resolve/extension/${extName}/${extId}`, method: "GET" },
            tauri: { cmd: "resolve_by_extension", args: { ext_name: extName, ext_id: extId } },
        });
    },

    linkTracker(cid: string, req: LinkTrackerRequest) {
        return call<ContentWithMappings>({
            http:  { path: `content/${cid}/link-tracker`, method: "POST", body: req },
            tauri: { cmd: "link_tracker", args: { cid, req } },
        });
    },

    resolveExtensionItem(extName: string, extId: string) {
        return call<ResolveExtensionResponse>({
            http:  { path: `content/resolve/extension/${extName}/${extId}/link`, method: "POST" },
            tauri: { cmd: "resolve_extension_item", args: { ext_name: extName, ext_id: extId } },
        });
    },

    searchExtension(extName: string, params: Pick<SearchQuery, "query" | "extensionFilters">) {
        return call<ExtensionSearchResponse>({
            http:  { path: `extensions/${extName}/search`, method: "GET", params: params as Record<string, unknown> },
            tauri: { cmd: "search_extension_direct", args: { ext_name: extName, params } },
        });
    },
};