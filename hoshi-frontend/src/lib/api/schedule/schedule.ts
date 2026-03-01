import { api } from "@/api/client";
import type { ScheduleQuery, ScheduleResponse } from "./types";

export const scheduleApi = {
    get(query?: ScheduleQuery) {
        return api<ScheduleResponse>("schedule", { params: query });
    },
};