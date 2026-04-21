import { scheduleApi } from "$lib/api/schedule/schedule";
import type { AiringEntry } from "$lib/api/schedule/types";
import type { CoreError } from "@/api/client";

class ScheduleStore {
    entries = $state<AiringEntry[]>([]);
    isLoading = $state(false);
    error = $state<CoreError | null>(null);
    viewMode = $state<"week" | "month">("week");

    async load() {
        this.isLoading = true;
        this.error = null;

        try {
            const daysAhead = this.viewMode === "week" ? 7 : 30;
            const res = await scheduleApi.get({ daysBack: 0, daysAhead });
            this.entries = res;
        } catch (err) {
            console.error("Failed to load schedule:", err);
            this.error = err as CoreError;
            this.entries = [];
        } finally {
            this.isLoading = false;
        }
    }
}

export const scheduleStore = new ScheduleStore();