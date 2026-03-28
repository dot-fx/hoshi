import { listApi } from "$lib/api/list/list";
import type { EnrichedListEntry, UserStats } from "$lib/api/list/types";
import type { CoreError } from "@/api/client";

class ListStore {
    entries = $state<EnrichedListEntry[]>([]);
    stats = $state<UserStats | null>(null);
    isLoading = $state(false);
    error = $state<CoreError | null>(null);
    isInitialized = $state(false);

    async loadData(forceRefresh = false) {
        if (this.isInitialized && !forceRefresh) return;

        this.isLoading = true;
        this.error = null;

        try {
            const [listRes, statsRes] = await Promise.all([
                listApi.getList({}),
                listApi.getStats()
            ]);

            this.entries = listRes.results;
            this.stats = statsRes;
            this.isInitialized = true;
        } catch (err) {
            console.error("Failed to load collection data:", err);
            this.error = err as CoreError;
        } finally {
            this.isLoading = false;
        }
    }

    async refresh() {
        await this.loadData(true);
    }
}

export const listStore = new ListStore();