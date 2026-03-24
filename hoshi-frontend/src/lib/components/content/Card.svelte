<script lang="ts">
    import type { ContentWithMappings } from '@/api/content/types';
    import { primaryMetadata } from '@/api/content/types';
    import { AspectRatio } from '@/components/ui/aspect-ratio';
    import * as HoverCard from '@/components/ui/hover-card';
    import { i18n } from '@/i18n/index.svelte.js';
    import { Star, Play, BookmarkPlus, Tv, Calendar } from 'lucide-svelte';

    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { appConfig } from '@/config.svelte.js';

    let { item, disableHover = false }: { item: ContentWithMappings, disableHover?: boolean } = $props();

    let meta = $derived(item ? primaryMetadata(item, appConfig.data?.content?.preferredMetadataProvider) : undefined);
    let displayTitle = $derived(() => {
        if (!meta) return '';
        const pref = appConfig.data?.ui?.titleLanguage || 'romaji';

        return meta.titleI18n?.[pref] || meta.title;
    });
    let href = $derived(item?.content?.cid ? `/content/${item.content.cid}` : '#');
    let year = $derived(meta?.releaseDate ? meta.releaseDate.split('-')[0] : null);

    let formattedScore = $derived(meta?.rating ? Math.round(meta.rating * 10) : null);
    let isListDialogOpen = $state(false);

    const YOUTUBE_REGEXP = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;

    const getYoutubeId = (url: string | undefined | null) => {
        if (!url) return null;
        const match = url.match(YOUTUBE_REGEXP);
        return (match && match[2].length === 11) ? match[2] : null;
    };

    let ytId = $derived(appConfig.data?.ui?.disableCardTrailers ? null : getYoutubeId(meta?.trailerUrl));
    let isExplicitlyNsfw = $derived(item?.content?.nsfw ?? false);

    let hasAdultGenre = $derived(
        meta?.genres?.some(g =>
            g.toLowerCase() === "hentai" ||
            g.toLowerCase() === "adult"
        )
    );

    let isAdultContent = $derived(isExplicitlyNsfw || hasAdultGenre);
    let shouldBlur = $derived(isAdultContent && appConfig.data?.general?.blurAdultContent);

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        if (type === 'TV') return i18n.t('card.TV');
        const formatted = type.replace('_', ' ').toLowerCase();
        const translationKey = formatted.replace(' ', '_') as any;
        const translated = i18n.t(translationKey);
        return translated === translationKey ? formatted.replace(/\b\w/g, l => l.toUpperCase()) : translated;
    };

    const formatStatus = (status: string | undefined | null) => {
        if (!status) return '';
        const key = `status_api.${status.toUpperCase()}` as any;
        const translated = i18n.t(key);
        return translated === key ? status : translated;
    };

    const formatGenre = (genre: string | undefined | null) => {
        if (!genre) return '';
        const key = `tags.${genre}` as any;
        const translated = i18n.t(key);
        return translated === key ? genre : translated;
    };

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

{#if item && meta}
    <ListEditor
            bind:open={isListDialogOpen}
            cid={item.content.cid}
            title={displayTitle()}
            contentType={item.content.contentType}
            coverImage={meta.coverImage || ''}
    />

    {#snippet baseCard()}
        <div class="flex flex-col gap-2.5">
            <div class="relative overflow-hidden rounded-xl bg-muted/20 shadow-sm ring-1 ring-border/10 transition-all duration-300 group-hover:shadow-lg group-hover:shadow-primary/5 group-hover:ring-primary/20">
                <AspectRatio ratio={2/3}>
                    <img src={meta.coverImage} alt={displayTitle()} loading="lazy" class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105 {shouldBlur ? 'blur-xl scale-110' : ''}" />
                    <div class="absolute inset-0 bg-gradient-to-t from-background/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"></div>
                </AspectRatio>

                {#if formattedScore}
                    <div class="absolute top-2 right-2 bg-black/70 backdrop-blur-md px-2 py-1 rounded-lg text-xs font-black text-white flex items-center gap-1.5 shadow-sm border border-white/10">
                        <Star class="w-3.5 h-3.5 text-yellow-500 fill-yellow-500" />
                        {(formattedScore / 10).toFixed(1)}
                    </div>
                {/if}
            </div>

            <div class="space-y-1.5 px-1">
                <div class="flex justify-between items-center text-xs font-bold text-muted-foreground/80">
                    <span class="uppercase tracking-wider">{formatType(meta.subtype)}</span>
                    {#if year}<span>{year}</span>{/if}
                </div>

                <div class="flex items-start gap-2">
                    {#if meta.status?.toUpperCase() === 'RELEASING'}
                        <span class="relative flex h-2 w-2 mt-1.5 shrink-0" title={i18n.t('card.airing')}>
                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                            <span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                        </span>
                    {/if}

                    <h3 class="font-bold text-sm md:text-base leading-tight line-clamp-2 text-foreground/90 group-hover:text-primary transition-colors">
                        {displayTitle()}
                    </h3>
                </div>
            </div>
        </div>
    {/snippet}

    {#if effectiveDisableHover}
        <a {href} class="block w-full outline-none group cursor-pointer">
            {@render baseCard()}
        </a>
    {:else}
        <HoverCard.Root openDelay={350} closeDelay={150}>

            <HoverCard.Trigger>
                {#snippet child({ props })}
                    <a {href} {...props} class="block w-full outline-none group cursor-pointer">
                        {@render baseCard()}
                    </a>
                {/snippet}
            </HoverCard.Trigger>

            <HoverCard.Content
                    side="right"
                    align="start"
                    sideOffset={12}
                    class="w-[320px] p-0 overflow-hidden shadow-2xl border-border/40 rounded-xl z-50 hidden md:flex flex-col bg-card"
            >
                <a {href} class="relative w-full aspect-video bg-black block group cursor-pointer overflow-hidden">
                    {#if ytId}
                        <iframe
                                loading="lazy"
                                src="https://www.youtube.com/embed/{ytId}?autoplay=1&mute=1&controls=0&loop=1&playlist={ytId}&showinfo=0&modestbranding=1"
                                class="absolute inset-0 w-full h-full object-cover scale-[1.35] pointer-events-none opacity-90"
                                frameborder="0"
                                allow="autoplay; encrypted-media"
                                title={i18n.t('card.trailer')}
                        ></iframe>
                    {:else if meta.bannerImage}
                        <img src={meta.bannerImage} alt={i18n.t('card.banner')} class="w-full h-full object-cover opacity-90 {shouldBlur ? 'blur-2xl scale-125' : ''}" />
                    {:else}
                        <img src={meta.coverImage} alt={i18n.t('card.cover')} class="w-full h-full object-cover scale-110 opacity-70 {shouldBlur ? 'blur-2xl scale-125' : 'blur-md'}" />
                    {/if}
                </a>

                <div class="p-4 flex flex-col gap-3 -mt-2 relative z-10">
                    <h3 class="font-bold text-base leading-tight">{displayTitle()}</h3>

                    <div class="flex flex-wrap gap-2 items-center">
                        {#if meta.subtype}
                            <span class="text-[10px] uppercase font-bold bg-muted px-2 py-1 rounded border border-border/50 text-muted-foreground">{formatType(meta.subtype)}</span>
                        {/if}
                        {#if meta.status}
                            <span class="text-[10px] uppercase font-bold {meta.status.toUpperCase() === 'RELEASING' ? 'text-green-500 border-green-500/30 bg-green-500/10' : 'text-primary border-primary/30 bg-primary/10'} border px-2 py-1 rounded">
                                {formatStatus(meta.status)}
                            </span>
                        {/if}
                        {#if meta.genres?.[0]}
                            <span class="text-[10px] font-bold bg-muted px-2 py-1 rounded border border-border/50 text-muted-foreground">{formatGenre(meta.genres[0])}</span>
                        {/if}
                    </div>

                    <div class="flex items-center gap-4 text-xs text-muted-foreground font-semibold">
                        {#if formattedScore}
                            <span class="flex items-center gap-1 text-yellow-500"><Star class="w-3.5 h-3.5 fill-yellow-500" /> {formattedScore}%</span>
                        {/if}
                        {#if meta.epsOrChapters}
                            <span class="flex items-center gap-1"><Tv class="w-3.5 h-3.5" /> {i18n.t('card.episodes', { count: meta.epsOrChapters })}</span>
                        {/if}
                        {#if meta.releaseDate}
                            <span class="flex items-center gap-1"><Calendar class="w-3.5 h-3.5" /> {meta.releaseDate}</span>
                        {/if}
                    </div>

                    {#if meta.synopsis}
                        <p class="text-xs text-muted-foreground line-clamp-3 leading-relaxed mt-1" title={meta.synopsis}>
                            {meta.synopsis.replace(/<[^>]*>?/gm, '')}
                        </p>
                    {/if}

                    <div class="flex gap-2 mt-3">
                        <a {href} class="flex-1 bg-primary text-primary-foreground text-sm font-bold py-2.5 rounded-lg flex items-center justify-center gap-2 hover:opacity-90 transition-opacity shadow-sm">
                            <Play class="w-4 h-4 fill-current" /> {i18n.t("card.watch")}
                        </a>

                        <button
                                onclick={(e) => { e.preventDefault(); isListDialogOpen = true; }}
                                class="w-11 h-11 rounded-lg bg-muted border border-border/50 flex items-center justify-center hover:bg-muted/80 hover:text-primary transition-colors shrink-0 shadow-sm"
                                title={i18n.t('list.add_to_list')}
                        >
                            <BookmarkPlus class="w-5 h-5" />
                        </button>
                    </div>
                </div>
            </HoverCard.Content>
        </HoverCard.Root>
    {/if}
{/if}