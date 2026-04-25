import { scheduleApi } from "@/api/schedule/schedule";
import type { AiringEntry } from "@/api/schedule/types";
import type { CoreError } from "@/api/client";

class ScheduleStore {
    weekEntries = $state<AiringEntry[]>([]);
    monthEntries = $state<AiringEntry[]>([]);

    isLoading = $state(false);
    error = $state<CoreError | null>(null);
    viewMode = $state<"week" | "month">("week");

    get entries() {
        return this.viewMode === "week" ? this.weekEntries : this.monthEntries;
    }

    async load(force = false) {
        if (!force && this.entries.length > 0) return;

        this.isLoading = true;
        this.error = null;

        try {
            const daysAhead = this.viewMode === "week" ? 7 : 30;
            const res = await scheduleApi.get({ daysBack: 0, daysAhead });

            if (this.viewMode === "week") {
                this.weekEntries = res;
            } else {
                this.monthEntries = res;
            }
        } catch (err) {
            console.error("Failed to load schedule:", err);
            this.error = err as CoreError;
            if (this.viewMode === "week") this.weekEntries = [];
            else this.monthEntries = [];
        } finally {
            this.isLoading = false;
        }
    }
}

export const scheduleStore = new ScheduleStore();