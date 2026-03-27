import { configApi } from "@/api/config/config";
import type { AppConfig } from "@/api/config/types";
import { goto } from "$app/navigation";
import type { CoreError } from "@/api/client";

class ConfigStore {
    data = $state<AppConfig | null>(null);
    loading = $state(false);
    error = $state<CoreError | null>(null);

    async load() {
        this.loading = true;
        this.error = null;

        try {
            this.data = await configApi.getConfig();

            if (this.data?.general?.needSetup) {
                goto("/setup");
            }
        } catch (err) {
            this.error = err as CoreError;
            this.data = null;

            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    async update(patch: Partial<AppConfig>) {
        this.loading = true;

        try {
            const updated = await configApi.patchConfig(patch);
            this.data = updated;
        } catch (err) {
            throw err as CoreError;
        } finally {
            this.loading = false;
        }
    }

    clear() {
        this.data = null;
        this.error = null;
    }
}

export const appConfig = new ConfigStore();