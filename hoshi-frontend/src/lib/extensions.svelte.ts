import { extensionsApi } from "@/api/extensions/extensions";
import type { Extension } from "@/api/extensions/types";

class ExtensionsStore {
    installed = $state<Extension[]>([]);
    loading = $state(false);
    initialized = $state(false);
    error = $state<string | null>(null);

    anime = $derived(this.installed.filter(ext => ext.ext_type === "anime"));
    manga = $derived(this.installed.filter(ext => ext.ext_type === "manga"));
    novel = $derived(this.installed.filter(ext => ext.ext_type === "novel"));

    async load(force = false) {
        if (this.initialized && !force) return;

        this.loading = true;
        this.error = null;

        try {
            this.installed = await extensionsApi.getAll();
        } catch (err: any) {
            this.error = err?.message ?? "Failed to load extensions";
            this.installed = [];
            console.error(err);
        } finally {
            this.loading = false;
            this.initialized = true;
        }
    }

    async install(manifestUrl: string) {
        this.loading = true;
        this.error = null;

        try {
            const res = await extensionsApi.install(manifestUrl);
            if (res.ok && res.extension) {
                this.installed = [...this.installed, res.extension];
            }
            return res;
        } catch (err: any) {
            this.error = err?.message ?? "Install failed";
            throw err;
        } finally {
            this.loading = false;
        }
    }

    async uninstall(id: string) {
        this.loading = true;
        this.error = null;

        try {
            const res = await extensionsApi.uninstall(id);
            if (res.ok) {
                this.installed = this.installed.filter(ext => ext.id !== id);
            }
            return res;
        } catch (err: any) {
            this.error = err?.message ?? "Uninstall failed";
            throw err;
        } finally {
            this.loading = false;
        }
    }
}

export const extensions = new ExtensionsStore();