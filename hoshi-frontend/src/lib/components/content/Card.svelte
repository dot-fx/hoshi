<script lang="ts">
    import {
        primaryMetadata,
        type TrackerMedia,
        type ExtensionSearchResult,
        type FullContent
    } from '@/api/content/types';
    import { AspectRatio } from '@/components/ui/aspect-ratio';
    import * as HoverCard from '@/components/ui/hover-card';
    import { fade, fly, scale } from 'svelte/transition';
    import { i18n } from '@/i18n/index.svelte.js';
    import { Star, Play, BookmarkPlus, Tv, BookOpen } from 'lucide-svelte';

    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { appConfig } from '@/stores/config.svelte.js';

    let {
        item,
        source = 'anilist',
        disableHover = false
    }: {
        item: TrackerMedia | ExtensionSearchResult | FullContent | any,
        source?: string,
        disableHover?: boolean
    } = $props();

    let normalized = $derived.by(() => {
        if (!item) return null;

        if ('content' in item && 'metadata' in item) {
            const meta = primaryMetadata(item, appConfig.data?.content?.preferredMetadataProvider);
            return {
                ...meta,
                id: item.content.cid,
                contentType: item.content.contentType,
                episodeCount: meta?.epsOrChapters,
                chapterCount: meta?.epsOrChapters,
                image: meta?.coverImage,
                trackerId: meta?.sourceId
            };
        }
        return item;
    });

    let cover = $derived(normalized?.coverImage || normalized?.image || '');
    let internalId = $derived(normalized?.trackerId || normalized?.id || normalized?.cid);
    let displayTitle = $derived(
        normalized?.titleI18n?.[appConfig.data?.ui?.titleLanguage || 'romaji'] || normalized?.title || ''
    );

    const isTracker = ['anilist', 'mal', 'kitsu'].includes(source.toLowerCase());
    let href = $derived(
        item.cid
            ? `/c/${item.cid}`
            : `/c/${source}/${internalId}`
    );

    let year = $derived(normalized?.releaseDate ? normalized.releaseDate.split('-')[0] : null);
    let formattedScore = $derived(normalized?.rating ? Math.round(normalized.rating * (normalized.rating <= 10 ? 10 : 1)) : null);

    let isListDialogOpen = $state(false);

    const YOUTUBE_REGEXP = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;
    const getYoutubeId = (url: string | undefined | null) => {
        if (!url) return null;
        const match = url.match(YOUTUBE_REGEXP);
        return (match && match[2].length === 11) ? match[2] : null;
    };
    let ytId = $derived(appConfig.data?.ui?.disableCardTrailers ? null : getYoutubeId(normalized?.trailerUrl));

    let isExplicitlyNsfw = $derived(normalized?.nsfw ?? false);
    let hasAdultGenre = $derived(
        normalized?.genres?.some((g: string) => g.toLowerCase() === "hentai" || g.toLowerCase() === "adult")
    );
    let isAdultContent = $derived(isExplicitlyNsfw || hasAdultGenre);
    let shouldBlur = $derived(isAdultContent && appConfig.data?.general?.blurAdultContent);

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        const key = type.toUpperCase();
        if (key === 'TV') return i18n.t('card.TV');
        return i18n.t(`card.${key}`) || type;
    };

    const formatStatus = (status: string | undefined | null) => {
        if (!status) return '';
        const key = `status_api.${status.toUpperCase()}` as any;
        const translated = i18n.t(key);
        return translated === key ? status : translated;
    };

    // Responsive
    let isMobile = $state(false);
    $effect(() => {
        const mql = window.matchMedia('(max-width: 768px)');
        isMobile = mql.matches;
        const update = (e: MediaQueryListEvent) => { isMobile = e.matches; };
        mql.addEventListener('change', update);
        return () => mql.removeEventListener('change', update);
    });
    let effectiveDisableHover = $derived(disableHover || isMobile);
</script>

{#if normalized}
    <ListEditor
            bind:open={isListDialogOpen}
            cid={item.content.cid}
            title={displayTitle}
            contentType={normalized.contentType}
            coverImage={cover}
    />

    {#snippet baseCard()}
        <div class="flex flex-col gap-2.5 group h-full">
            <div class="relative overflow-hidden rounded-xl bg-muted/20 shadow-sm ring-1 ring-border/10 transition-all duration-400 group-hover:shadow-xl group-hover:shadow-primary/10 group-hover:ring-primary/30">
                <AspectRatio ratio={2/3}>
                    <img
                            src={cover}
                            alt={displayTitle}
                            loading="lazy"
                            class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-110 {shouldBlur ? 'blur-xl scale-110' : ''}"
                    />
                    <div class="absolute inset-0 bg-gradient-to-t from-background/85 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-400 pointer-events-none"></div>
                </AspectRatio>

                {#if formattedScore}
                    <div class="absolute top-2 right-2 bg-black/70 backdrop-blur-md px-2 py-1 rounded-lg text-xs font-black text-white flex items-center gap-1.5 shadow-sm border border-white/10">
                        <Star class="w-3.5 h-3.5 text-yellow-500 fill-yellow-500" />
                        {formattedScore}%
                    </div>
                {/if}
            </div>

            <div class="space-y-1.5 px-1">
                <div class="flex justify-between items-center text-xs font-bold text-muted-foreground/80">
                    <span class="uppercase tracking-wider">{formatType(normalized.format || normalized.subtype)}</span>
                    {#if year}<span>{year}</span>{/if}
                </div>
                <h3 class="font-bold text-sm md:text-base leading-tight line-clamp-2 text-foreground/90 group-hover:text-primary transition-colors duration-300">
                    {displayTitle}
                </h3>
            </div>
        </div>
    {/snippet}

    {#if effectiveDisableHover}
        <a href={href} class="block w-full outline-none cursor-pointer h-full">
            {@render baseCard()}
        </a>
    {:else}
        <HoverCard.Root openDelay={400} closeDelay={150}>
            <HoverCard.Trigger>
                {#snippet child({ props })}
                    <a
                            href={href}
                            {...props}
                            class="block w-full outline-none cursor-pointer h-full"
                            onclick={(e) => { props.onclick?.(e); }}
                    >
                        {@render baseCard()}
                    </a>
                {/snippet}
            </HoverCard.Trigger>

            <HoverCard.Content
                    side="right"
                    align="start"
                    sideOffset={16}
                    class="w-[320px] p-0 overflow-hidden shadow-2xl border-border/40 rounded-2xl z-50 hidden md:flex flex-col bg-card"
            >
                <div class="w-full h-full" in:scale={{ duration: 220, start: 0.94 }} out:fade={{ duration: 180 }}>
                    <div class="relative w-full aspect-video bg-black block overflow-hidden">
                        {#if ytId}
                            <iframe
                                    loading="lazy"
                                    src="https://www.youtube.com/embed/{ytId}?autoplay=1&mute=1&controls=0&loop=1&playlist={ytId}&showinfo=0&modestbranding=1"
                                    class="absolute inset-0 w-full h-full object-cover scale-[1.35] pointer-events-none opacity-90"
                                    frameborder="0"
                                    allow="autoplay; encrypted-media"
                                    title={i18n.t('card.trailer')}
                            ></iframe>
                        {:else if normalized.bannerImage}
                            <img src={normalized.bannerImage} alt="" class="w-full h-full object-cover opacity-90 {shouldBlur ? 'blur-2xl scale-125' : ''}" />
                        {:else}
                            <img src={cover} alt="" class="w-full h-full object-cover scale-110 opacity-70 {shouldBlur ? 'blur-2xl scale-125' : 'blur-md'}" />
                        {/if}
                    </div>

                    <div class="p-4 flex flex-col gap-3 -mt-2 relative z-10">
                        <h3 class="font-bold text-base leading-tight" in:fly={{ y: 12, duration: 300, delay: 80 }}>
                            {displayTitle}
                        </h3>

                        <div class="flex flex-wrap gap-2 items-center" in:fly={{ y: 12, duration: 300, delay: 140 }}>
                            {#if normalized.format || normalized.subtype}
                                <span class="text-[10px] uppercase font-bold bg-muted px-2 py-1 rounded border border-border/50 text-muted-foreground">{formatType(normalized.format || normalized.subtype)}</span>
                            {/if}
                            {#if normalized.status}
                                <span class="text-[10px] uppercase font-bold {normalized.status.toUpperCase() === 'RELEASING' ? 'text-green-500 border-green-500/30 bg-green-500/10' : 'text-primary border-primary/30 bg-primary/10'} border px-2 py-1 rounded">
                                    {formatStatus(normalized.status)}
                                </span>
                            {/if}
                        </div>

                        <div class="flex items-center gap-4 text-xs text-muted-foreground font-semibold" in:fly={{ y: 12, duration: 300, delay: 200 }}>
                            {#if formattedScore}
                                <span class="flex items-center gap-1 text-yellow-500"><Star class="w-3.5 h-3.5 fill-yellow-500" /> {formattedScore}%</span>
                            {/if}
                            {#if normalized.episodeCount || normalized.chapterCount}
                                <span class="flex items-center gap-1">
                                    {#if normalized.contentType === 'anime'}<Tv class="w-3.5 h-3.5" />{:else}<BookOpen class="w-3.5 h-3.5" />{/if}
                                    {normalized.episodeCount || normalized.chapterCount}
                                </span>
                            {/if}
                        </div>

                        {#if normalized.synopsis}
                            <p class="text-xs text-muted-foreground line-clamp-3 leading-relaxed mt-1" in:fly={{ y: 12, duration: 300, delay: 260 }}>
                                {normalized.synopsis.replace(/<[^>]*>?/gm, '')}
                            </p>
                        {/if}

                        <div class="flex gap-2 mt-3">
                            <a href={href} class="flex-1 bg-primary text-primary-foreground text-sm font-bold py-2.5 rounded-xl flex items-center justify-center gap-2 hover:opacity-90 transition-all shadow-sm">
                                <Play class="w-4 h-4 fill-current" /> {i18n.t("card.watch")}
                            </a>
                            {#if isTracker}
                                <button
                                        onclick={(e) => { e.preventDefault(); isListDialogOpen = true; }}
                                        class="w-11 h-11 rounded-xl bg-muted border border-border/50 flex items-center justify-center hover:bg-muted/80 hover:text-primary transition-all shrink-0 shadow-sm"
                                >
                                    <BookmarkPlus class="w-5 h-5" />
                                </button>
                            {/if}
                        </div>
                    </div>
                </div>
            </HoverCard.Content>
        </HoverCard.Root>
    {/if}
{/if}