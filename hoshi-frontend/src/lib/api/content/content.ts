import { call } from "@/api/client";
import type {
    FullContent,
    ContentListResponse,
    PlayResponse,
    SearchQuery,
    UpdateTrackerMappingRequest,
    UpdateExtensionMappingRequest,
    Metadata,
    TrackerMapping,
    ExtensionSource,
    HomeView,
    ContentType,
} from "./types";

export const contentApi = {

    getHome() {
        return call<HomeView>({
            tauri: { cmd: "get_home_content" },
        });
    },

    getTrending(mediaType: ContentType) {
        return call<FullContent[]>({
            tauri: { cmd: "get_trending", args: { media_type: mediaType } },
        });
    },

    get(source: string, sourceId: string) {
        return call<FullContent>({
            tauri: {
                cmd: "get_content",
                args: {
                    source,
                    source_id: sourceId
                }
            },
        });
    },

    update(cid: string, meta: Metadata) {
        return call<FullContent>({
            tauri: { cmd: "update_content", args: { cid, meta } },
        });
    },

    search(query: SearchQuery) {
        return call<ContentListResponse>({
            tauri: { cmd: "search", args: { query } },
        });
    },

    getItems(cid: string, extName: string) {
        return call<unknown>({
            tauri: { cmd: "get_content_items", args: { cid, ext_name: extName } },
        });
    },

    play(cid: string, extName: string, number: number, opts?: { server?: string; category?: string }) {
        return call<PlayResponse>({
            tauri: { cmd: "play_content_by_number", args: { cid, ext_name: extName, number, ...opts } },
        });
    },

    addTrackerMapping(cid: string, mapping: TrackerMapping) {
        return call<void>({
            tauri: { cmd: "add_tracker_mapping", args: { cid, mapping } },
        });
    },

    updateTrackerMapping(cid: string, req: UpdateTrackerMappingRequest) {
        return call<void>({
            tauri: { cmd: "update_tracker_mapping", args: { cid, req } },
        });
    },

    deleteTrackerMapping(cid: string, trackerName: string) {
        return call<void>({
            tauri: { cmd: "delete_tracker_mapping", args: { cid, tracker_name: trackerName } },
        });
    },

    addExtensionSource(cid: string, source: ExtensionSource) {
        return call<number>({
            tauri: { cmd: "add_extension_mapping", args: { cid, source } },
        });
    },

    updateExtensionMapping(cid: string, req: UpdateExtensionMappingRequest) {
        return call<FullContent>({
            tauri: { cmd: "update_extension_mapping", args: { cid, req } },
        });
    },

    resolveByTracker(tracker: string, id: string) {
        return call<FullContent>({
            tauri: { cmd: "resolve_by_tracker", args: { tracker, id } },
        });
    },

    resolveByExtension(extName: string, extId: string) {
        return call<FullContent>({
            tauri: { cmd: "resolve_by_extension", args: { ext_name: extName, ext_id: extId } },
        });
    },

    searchExtension(extName: string, params: Pick<SearchQuery, "query" | "extensionFilters">) {
        return call<SearchQuery>({
            tauri: {
                cmd: "search_extension",
                args: {
                    ext_name: extName,
                    params: {
                        query: params.query,
                        extension_filters: params.extensionFilters,
                    },
                },
            },
        });
    },
};