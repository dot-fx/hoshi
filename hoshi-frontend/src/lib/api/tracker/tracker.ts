import { call } from "@/api/client";
import type { TrackerInfo, TrackerIntegration, SyncResponse } from "./types";

export const integrationsApi = {
    getAll() {
        return call<TrackerInfo[]>({
            http:  { path: "trackers", method: "GET" },
            tauri: { cmd: "list_trackers" },
        });
    },

    add(body: TrackerIntegration) {
        return call<void>({
            http:  { path: "integrations", method: "POST", body },
            tauri: { cmd: "add_integration", args: { body } },
        });
    },

    remove(trackerName: string) {
        return call<void>({
            http:  { path: `integrations/${trackerName}`, method: "DELETE" },
            tauri: { cmd: "remove_integration", args: { trackerName } },
        });
    },

    sync() {
        return call<SyncResponse>({
            http:  { path: "list/sync", method: "POST" },
            tauri: { cmd: "sync_list" }, // no existe aún en Tauri, ver nota
        });
    },
};