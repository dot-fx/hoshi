import { call } from "@/api/client";
import type { TrackerInfo, TrackerIntegration } from "./types";

export const integrationsApi = {
    getAll() {
        return call<TrackerInfo[]>({
            tauri: { cmd: "list_trackers" },
        });
    },

    add(body: TrackerIntegration) {
        return call<void>({
            tauri: { cmd: "add_integration", args: { body } },
        });
    },

    remove(trackerName: string) {
        return call<void>({
            tauri: { cmd: "remove_integration", args: { trackerName } },
        });
    },

    setSyncEnabled(trackerName: string, enabled: boolean) {
        return call<void>({
            tauri: { cmd: "set_sync_enabled", args: { trackerName, enabled } },
        });
    },
};