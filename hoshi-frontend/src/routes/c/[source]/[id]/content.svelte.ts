import { page } from "$app/state";
import { goto } from "$app/navigation";

import { contentApi } from "@/api/content/content";
import { i18n } from "@/i18n/index.svelte.js";
import { primaryMetadata } from "@/api/content/types";
import type { FullContent } from "@/api/content/types";
import { layoutState } from '@/stores/layout.svelte.js';
import { listApi } from "@/api/list/list";
import { appConfig } from "@/stores/config.svelte.js";
import { contentCache } from "@/stores/contentCache.svelte.js";

export class ContentDetailState {
    isLoading = $state(true);
    error = $state<any>(null);
    fullContent = $state<FullContent | null>(null);
    isEntryLoading = $state(false);
    hasEntry = $state(false);

    #source = $derived(page.params.source || "");
    #id = $derived(page.params.id || "");

    constructor() {
        $effect(() => {
            if (this.#source && this.#id) {
                this.loadContent(this.#source, this.#id);
            }
        });
    }

    async loadContent(src: string, entryId: string) {
        this.isLoading = true;
        this.error = null;
        this.fullContent = null;

        try {
            // const cacheKey = `${src}:${entryId}`;
            // const cachedData = contentCache.get(cacheKey);

            const res = await contentApi.get(src, entryId);
            console.log(res)

            // if (!cachedData) {
            //     contentCache.set(cacheKey, res);
            // }

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

        } catch (e) {
            this.error = e;
            console.log(e);
            layoutState.title = i18n.t('errors.error');
        } finally {
            this.isLoading = false;
        }
    }

    watchNow(contentData: FullContent) {
        const cid = contentData.content.cid;
        if (contentData.content.contentType === 'anime') {
            goto(`/watch/${cid}/1`);
        } else {
        }
    }

    retry() {
        if (this.#source && this.#id) {
            this.loadContent(this.#source, this.#id);
        }
    }
}