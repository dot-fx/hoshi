import { api } from "@/api/client";
import type {
    ListResponse,
    SingleEntryResponse,
    UpsertEntryResponse,
    SuccessResponse,
    UpsertEntryBody,
    FilterQuery,
} from "./types";

export const listApi = {
    getList(query?: FilterQuery) {
        return api<ListResponse>("list", { params: query });
    },

    getEntry(cid: string) {
        return api<SingleEntryResponse>(`list/entry/${cid}`);
    },

    upsert(body: UpsertEntryBody) {
        return api<UpsertEntryResponse>("list/entry", {
            method: "POST",
            body,
        });
    },

    delete(cid: string) {
        return api<SuccessResponse>(`list/entry/${cid}`, {
            method: "DELETE",
        });
    },
};