<script lang="ts">
    import { Skeleton } from '$lib/components/ui/skeleton';
    import { Tv, Book, BookText } from "lucide-svelte";

    import Hero from '$lib/components/content/Hero.svelte';
    import ContentCarousel from '@/components/content/Carousel.svelte';
    import ContinueCarousel from '@/components/content/Continue.svelte';
    import { fade } from 'svelte/transition';

    import { contentApi } from '@/api/content/content';
    import { progressApi } from '@/api/progress/progress';
    import type { ContentWithMappings, ContentType, HomeMediaItem, MediaSection } from '@/api/content/types';
    import type { ContinueItem } from '@/api/progress/types';
    import { appConfig } from "@/config.svelte";
    import { layoutState } from '$lib/layoutState.svelte';
    import { i18n } from '$lib/i18n/index.svelte';

    let loading = $state(true);
    let error = $state(false);

    let currentMode = $state<ContentType>('anime');
    let initializedMode = $state(false);

    $effect(() => {
        if (appConfig.data && !initializedMode) {
            currentMode = appConfig.data.ui.defaultHomeSection as ContentType;
            initializedMode = true;
        }
    });

    $effect(() => {
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    type MappedSection = {
        trending: ContentWithMappings[];
        seasonal: ContentWithMappings[];
        topRated: ContentWithMappings[];
    };

    let content = $state<Record<ContentType, MappedSection>>({
        anime: { trending: [], seasonal: [], topRated: [] },
        manga: { trending: [], seasonal: [], topRated: [] },
        novel: { trending: [], seasonal: [], topRated: [] }
    });

    let continueItems = $state<ContinueItem[]>([]);

    let currentContinueItems = $derived(
        continueItems.filter(item => item.contentType === currentMode)
    );

    const mapToContentWithMappings = (item: HomeMediaItem): ContentWithMappings => {
        return {
            content: { cid: item.cid, contentType: item.contentType, nsfw: false, createdAt: Date.now(), updatedAt: Date.now() },
            metadata: [{
                cid: item.cid, sourceName: 'anilist', title: item.title, altTitles: item.altTitles,
                synopsis: item.synopsis, coverImage: item.coverImage,
                bannerImage: item.bannerImage,
                subtype: item.format, status: item.status as any, releaseDate: item.releaseDate,
                endDate: item.endDate, rating: item.rating, genres: item.genres, tags: item.tags,
                trailerUrl: item.trailerUrl, characters: [], staff: [], externalIds: {}, createdAt: Date.now(), updatedAt: Date.now()
            }],
            trackerMappings: [],
            extensionSources: [], relations: [], contentUnits: []
        };
    };

    const mapSection = (section: MediaSection | undefined): MappedSection => ({
        trending: (section?.trending || []).map(mapToContentWithMappings),
        seasonal: (section?.seasonal || []).map(mapToContentWithMappings),
        topRated: (section?.topRated || []).map(mapToContentWithMappings)
    });

    $effect(() => {
        Promise.all([
            contentApi.getHome(),
            progressApi.getContinueWatching(20)
        ])
            .then(([res, progRes]) => {
                content = {
                    anime: mapSection(res.anime),
                    manga: mapSection(res.manga),
                    novel: mapSection(res.novel)
                };
                continueItems = progRes.items || [];
            })
            .catch((err) => {
                console.error("Failed to load home content", err);
                error = true;
            })
            .finally(() => {
                loading = false;
            });
    });

    const modes = [
        { id: 'anime', label: 'Anime', icon: Tv },
        { id: 'manga', label: 'Manga', icon: Book },
        { id: 'novel', label: 'Novel', icon: BookText }
    ];
</script>

<svelte:head>
    <title>{i18n.t("home.title")}</title>
</svelte:head>

<div class="min-h-screen bg-background pb-20 overflow-x-hidden relative">
    <div class="fixed top-6 left-1/2 -translate-x-1/2 z-50 flex items-center p-1.5 bg-background/70 backdrop-blur-xl border border-border/50 rounded-full shadow-2xl transition-all">
        {#each modes as { id, label, icon: Icon }}
            <button
                    class="relative flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-bold transition-all duration-300
                    {currentMode === id ? 'bg-primary text-primary-foreground shadow-lg scale-105' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
                    onclick={() => currentMode = id}
            >
                <Icon class="size-4 shrink-0" />
                <span class="hidden sm:inline">{label}</span>
            </button>
        {/each}
    </div>

    {#if loading}
        <div class="w-full h-[85vh] bg-card/50 flex items-end p-4 md:p-12 animate-pulse pt-20">
            <div class="space-y-6 max-w-3xl w-full mb-10"><Skeleton class="h-12 md:h-20 w-3/4 bg-muted rounded-lg" /></div>
        </div>
    {:else if error}
        <div class="h-screen w-full flex flex-col items-center justify-center text-muted-foreground gap-4">
            <p class="text-lg font-bold">{i18n.t("errors.network")}</p>
            <button class="text-primary hover:underline font-medium" onclick={() => location.reload()}>{i18n.t("home.try_again")}</button>
        </div>
    {:else}
        {#key currentMode}
            <div in:fade={{ duration: 300 }}>
                {#if content[currentMode].trending.length > 0}
                    <Hero items={content[currentMode].trending.slice(0, 5)} />
                {/if}

                <div class="w-full px-4 md:px-12 py-8 relative z-20 space-y-12 -mt-16 md:-mt-24">

                    {#if currentContinueItems.length > 0}
                        <ContinueCarousel items={currentContinueItems} mode={currentMode} />
                    {/if}

                    {#if content[currentMode].trending.length > 0}
                        <ContentCarousel title={i18n.t("home.trending")} items={content[currentMode].trending} />
                    {/if}
                    {#if content[currentMode].seasonal.length > 0}
                        <ContentCarousel title={currentMode === 'anime' ? i18n.t("home.simulcast") : i18n.t("home.latest")} items={content[currentMode].seasonal} />
                    {/if}
                    {#if content[currentMode].topRated.length > 0}
                        <ContentCarousel title={i18n.t("home.critically_aclaimed")} items={content[currentMode].topRated} />
                    {/if}
                </div>
            </div>
        {/key}
    {/if}
</div>