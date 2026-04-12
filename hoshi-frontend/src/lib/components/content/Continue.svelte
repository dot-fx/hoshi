<script lang="ts">
    import { PlayCircle, FileText, Play } from "lucide-svelte";
    import type { ContinueItem } from '@/api/progress/types';
    import type { ContentType } from '@/api/content/types';
    import { i18n } from "@/i18n/index.svelte";
    import { appConfig } from "@/stores/config.svelte.js";

    let {
        items,
        mode
    }: {
        items: ContinueItem[];
        mode: ContentType;
    } = $props();

    let visibleItems = $derived(items.filter(item => {
        if (item.nsfw && !appConfig.data?.general?.showAdultContent) return false;
        return true;
    }));

    function isBlurred(item: ContinueItem) {
        return item.nsfw && appConfig.data?.general?.blurAdultContent;
    }

    function getDisplayTitle(item: ContinueItem) {
        if (!appConfig.data) return item.title;
        const lang = appConfig.data.ui.titleLanguage || 'romaji';
        return item.titleI18n?.[lang] || item.titleI18n?.['romaji'] || item.title;
    }

    function processImageUrl(url: string | null | undefined) {
        if (!url) return null;
        return url.replace('_m.', '_w.');
    }

    function getContinueUrl(item: ContinueItem) {
        if (item.contentType === 'anime' && item.episode) {
            const ratio = (item.episodeDurationSeconds && item.timestampSeconds)
                ? item.timestampSeconds / item.episodeDurationSeconds
                : 0;
            if (ratio >= 0.95) {
                return `/watch/${item.cid}/${item.episode + 1}`;
            } else if (item.timestampSeconds && item.timestampSeconds > 0) {
                return `/watch/${item.cid}/${item.episode}?t=${item.timestampSeconds}`;
            } else {
                return `/watch/${item.cid}/${item.episode}`;
            }
        }
        return `/content/${item.cid}`;
    }
</script>

{#if visibleItems.length > 0}
    <div class="space-y-4">
        <h2 class="text-xl md:text-2xl font-black tracking-tight flex items-center gap-2 text-foreground px-1">
            {mode === 'anime' ? i18n.t("home.continue.continue_watching") : i18n.t("home.continue.continue_reading")}
        </h2>

        <div class="flex overflow-x-auto gap-4 pb-6 pt-2 custom-scrollbar snap-x px-1">
            {#each visibleItems as item}
                {@const rawImg = item.unit?.thumbnailUrl || item.coverImage}
                {@const imageUrl = processImageUrl(rawImg)}

                {@const displayTitle = getDisplayTitle(item)}
                {@const progressPercent = (item.timestampSeconds && item.episodeDurationSeconds)
                    ? (item.timestampSeconds / item.episodeDurationSeconds) * 100
                    : 0}
                {@const isEnriched = !!item.unit?.thumbnailUrl}
                <a
                        href={getContinueUrl(item)}
                        class="group flex flex-col gap-3 shrink-0 snap-start transition-all w-[280px] sm:w-[320px] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary rounded-xl"
                >
                    <div class="relative w-full {mode === 'anime' ? 'aspect-video' : 'aspect-[3/4] max-w-[200px]'} rounded-xl overflow-hidden bg-muted/30 border border-border/40 shadow-sm group-hover:shadow-md group-hover:border-primary/40 transition-all">

                        {#if imageUrl}
                            <img
                                    src={imageUrl}
                                    alt={displayTitle}
                                    class="w-full h-full object-cover ... {isBlurred(item) ? 'blur-xl' : ''}"
                            />
                        {:else}
                            <div class="w-full h-full flex items-center justify-center opacity-30">
                                <PlayCircle class="size-8" />
                            </div>
                        {/if}

                        <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-60"></div>

                        <div class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300 bg-black/20 backdrop-blur-[2px]">
                            <div class="bg-primary/90 text-primary-foreground rounded-full p-3 shadow-xl transform scale-90 group-hover:scale-100 transition-transform">
                                {#if mode === 'anime'}
                                    <Play class="size-6 fill-current ml-1" />
                                {:else}
                                    <FileText class="size-6 fill-current" />
                                {/if}
                            </div>
                        </div>

                        <div class="absolute top-2 left-2 bg-black/60 backdrop-blur-md text-white text-xs font-bold px-2 py-1 rounded-md">
                            {#if mode === 'anime' && item.episode}
                                {i18n.t('home.continue.episodes', { num: item.episode })}
                            {:else if item.chapter}
                                {i18n.t('home.continue.chapters', { num: item.chapter })}
                            {/if}
                        </div>

                        {#if item.nsfw}
                            <div class="absolute top-2 right-2 bg-destructive/90 text-destructive-foreground text-[10px] font-black uppercase px-2 py-0.5 rounded-sm tracking-wider shadow-sm">
                                18+
                            </div>
                        {/if}

                        {#if mode === 'anime' && progressPercent > 0}
                            <div class="absolute bottom-0 left-0 right-0 h-1.5 bg-white/20">
                                <div class="h-full bg-primary" style="width: {progressPercent}%"></div>
                            </div>
                        {/if}
                    </div>

                    <div class="flex flex-col px-1">
                        <h3 class="font-bold text-sm sm:text-base leading-tight line-clamp-1 group-hover:text-primary transition-colors" title={displayTitle}>
                            {displayTitle}
                        </h3>

                        {#if mode === 'anime'}
                            <p class="text-xs sm:text-sm text-muted-foreground line-clamp-1 mt-0.5" title={item.unit?.title || ''}>
                                {#if item.unit?.title}
                                    {item.unit.title}
                                {:else}
                                    {i18n.t('home.continue.episodes', { num: item.episode })}
                                {/if}
                            </p>
                        {/if}
                    </div>
                </a>
            {/each}
        </div>
    </div>
{/if}

<style>
    .custom-scrollbar::-webkit-scrollbar { height: 6px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(150,150,150,0.2);
        border-radius: 10px;
    }
    .custom-scrollbar:hover::-webkit-scrollbar-thumb {
        background: rgba(150,150,150,0.4);
    }
</style>