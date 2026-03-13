import { call } from "@/api/client";
import type { TrackerInfo, TrackerIntegration } from "./types";

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

    setSyncEnabled(trackerName: string, enabled: boolean) {
        return call<void>({
            http:  { path: `integrations/${trackerName}/sync`, method: "PATCH", body: { enabled } },
            tauri: { cmd: "set_sync_enabled", args: { trackerName, enabled } },
        });
    },
};