import { page } from "$app/state";
import { goto } from "$app/navigation";

import { contentApi } from "@/api/content/content";
import { i18n } from "@/stores/i18n.svelte.js";
import { primaryMetadata } from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { layoutState } from '@/stores/layout.svelte.js';
import { listApi } from "@/api/list/list";
import { appConfig } from "@/stores/config.svelte.js";

export class ContentDetailState {
    isLoading = $state(true);
    error = $state<any>(null);
    fullContent = $state<FullContent | null>(null);
    isEntryLoading = $state(false);
    hasEntry = $state(false);

    params = $derived(page.params as Record<string, string>);

    pathParts = $derived(this.params.path ? this.params.path.split('/') : []);

    source = $derived(this.pathParts.length === 2 ? this.pathParts[0] : "");
    id = $derived(this.pathParts.length === 2 ? this.pathParts[1] : "");
    cid = $derived(this.pathParts.length === 1 ? this.pathParts[0] : "");

    constructor() {
        $effect(() => {
            if (this.cid) {
                this.loadContentByCid(this.cid);
            } else if (this.source && this.id) {
                this.loadContent(this.source, this.id);
            }
        });
    }

    async loadContentByCid(cid: string) {
        this.isLoading = true;
        this.error = null;
        this.fullContent = null;

        try {
            const res = await contentApi.get_by_cid(cid);
            console.log(res);
            await this.handleResponse(res);
        } catch (e) {
            this.handleError(e);
        } finally {
            this.isLoading = false;
        }
    }

    async loadContent(src: string, entryId: string) {
        this.isLoading = true;
        this.error = null;
        this.fullContent = null;

        try {
            const res = await contentApi.get(src, entryId);
            console.log(res);
            await this.handleResponse(res);
        } catch (e) {
            this.handleError(e);
        } finally {
            this.isLoading = false;
        }
    }

    private async handleResponse(res: FullContent) {
        this.fullContent = res;

        const meta = primaryMetadata(res, appConfig.data?.content?.preferredMetadataProvider);
        if (meta) {
            const pref = appConfig.data?.ui?.titleLanguage || 'romaji';
            const title = meta.titleI18n?.[pref] || meta.title || '';
            layoutState.title = title.length > 35 ? title.slice(0, 35).trim() + '...' : title;
        }

        this.isEntryLoading = true;
        try {
            const listRes = await listApi.getEntry(res.content.cid);
            this.hasEntry = listRes.found;
        } catch {
            this.hasEntry = false;
        } finally {
            this.isEntryLoading = false;
        }
    }

    private handleError(e: any) {
        this.error = e;
        console.log(e);
        layoutState.title = i18n.t('errors.error');
    }

    watchNow(contentData: FullContent) {
        const cid = contentData.content.cid;
        if (contentData.content.contentType === 'anime') {
            goto(`/watch/${cid}/1`);
        } else {
        }
    }

    retry() {
        if (this.cid) {
            this.loadContentByCid(this.cid);
        } else if (this.source && this.id) {
            this.loadContent(this.source, this.id);
        }
    }
}