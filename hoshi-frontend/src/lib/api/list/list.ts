import { call } from "@/api/client";
import type {
    ListResponse,
    SingleEntryResponse,
    UpsertEntryResponse,
    UpsertEntryBody,
    FilterQuery,
} from "./types";

export const listApi = {
    getList(query?: FilterQuery) {
        return call<ListResponse>({
            http:  { path: "list", method: "GET", params: query as Record<string, unknown> },
            tauri: { cmd: "get_list", args: { query: query ?? {} } },
        });
    },

    getEntry(cid: string) {
        return call<SingleEntryResponse>({
            http:  { path: `list/entry/${cid}`, method: "GET" },
            tauri: { cmd: "get_single_entry", args: { cid } },
        });
    },

    upsert(body: UpsertEntryBody) {
        return call<UpsertEntryResponse>({
            http:  { path: "list/entry", method: "POST", body },
            tauri: { cmd: "upsert_entry", args: { body } },
        });
    },

    delete(cid: string) {
        return call<void>({
            http:  { path: `list/entry/${cid}`, method: "DELETE" },
            tauri: { cmd: "delete_entry", args: { cid } },
        });
    },
};