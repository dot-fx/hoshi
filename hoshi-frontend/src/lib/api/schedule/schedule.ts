import { call } from "@/api/client";
import type { AiringEntry, ScheduleQuery } from "./types";

export const scheduleApi = {
    get(window?: ScheduleQuery) {
        return call<AiringEntry[]>({
            http:  { path: "schedule", method: "GET", params: window as Record<string, unknown> },
            tauri: { cmd: "get_schedule", args: { window: window ?? {} } },
        });
    },
};