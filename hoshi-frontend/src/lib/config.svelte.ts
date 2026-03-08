import { configApi, type AppConfig } from "@/api/config/config";

class ConfigStore {
    data = $state<AppConfig | null>(null);
    loading = $state(false);
    error = $state<string | null>(null);

    async load() {
        this.loading = true;
        this.error = null;

        try {
            this.data = await configApi.getConfig();
        } catch (err: any) {
            this.error = err?.message ?? "Failed to load config";
            this.data = null;
        } finally {
            this.loading = false;
        }
    }

    async update(patch: Partial<AppConfig>) {
        this.loading = true;
        this.error = null;

        try {
            // El backend devuelve la configuración ya actualizada y mergeada
            const updated = await configApi.patchConfig(patch);
            this.data = updated;
        } catch (err: any) {
            this.error = err?.message ?? "Failed to update config";
            throw err;
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