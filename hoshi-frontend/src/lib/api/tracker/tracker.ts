import { api } from "@/api/client";
import type { IntegrationsResponse, SuccessResponse, SyncResponse, TrackerIntegration } from "./types";

export const integrationsApi = {
    getAll() {
        return api<IntegrationsResponse>("integrations");
    },

    add(body: TrackerIntegration) {
        return api<SuccessResponse>("integrations", {
            method: "POST",
            body,
        });
    },

    remove(trackerName: string) {
        return api<SuccessResponse>(`integrations/${trackerName}`, {
            method: "DELETE",
        });
    },

    sync() {
        return api<SyncResponse>("list/sync", {
            method: "POST",
        });
    },
};