import { call } from "@/api/client";
import type {
    ListResponse,
    SingleEntryResponse,
    UpsertEntryResponse,
    UpsertEntryBody,
    FilterQuery,
    UserStats,
} from "./types";

export const listApi = {
    getList(query?: FilterQuery) {
        return call<ListResponse>({
            http:  { path: "list", method: "GET", params: query as Record<string, unknown> },
            tauri: { cmd: "get_list", args: { query: query ?? {} } },
        });
    },

    getStats() {
        return call<UserStats>({
            tauri: { cmd: "get_stats", args: {} },
        });
    },

    getEntry(cid: string) {
        return call<SingleEntryResponse>({
            tauri: { cmd: "get_single_entry", args: { cid } },
        });
    },

    upsert(body: UpsertEntryBody) {
        return call<UpsertEntryResponse>({
            tauri: { cmd: "upsert_entry", args: { body } },
        });
    },

    delete(cid: string) {
        return call<void>({
            tauri: { cmd: "delete_entry", args: { cid } },
        });
    },
};