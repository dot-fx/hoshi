<script lang="ts">
    import { Skeleton } from '@/components/ui/skeleton';
    import { Tv, Book, BookText } from "lucide-svelte";
    import Hero from '@/components/content/Hero.svelte';
    import ContentCarousel from '@/components/content/Carousel.svelte';
    import ContinueCarousel from '@/components/content/Continue.svelte';
    import { fade } from 'svelte/transition';
    import { contentApi } from '@/api/content/content';
    import { progressApi } from '@/api/progress/progress';
    import type { ContentWithMappings, ContentType, HomeMediaItem, MediaSection } from '@/api/content/types';
    import type { ContinueItem } from '@/api/progress/types';
    import { appConfig } from "@/config.svelte.js";
    import { layoutState } from '@/layoutState.svelte.js';
    import { i18n } from '@/i18n/index.svelte.js';
    import { onMount } from 'svelte';

    let loading = $state(true);
    let error = $state(false);

    let currentMode = $state<ContentType>('anime');
    let initializedMode = $state(false);

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

    const modes = [
        { id: 'anime', label: 'Anime', icon: Tv },
        { id: 'manga', label: 'Manga', icon: Book },
        { id: 'novel', label: 'Novel', icon: BookText }
    ] as const;

    $effect(() => {
        if (appConfig.data && !initializedMode) {
            currentMode = appConfig.data.ui.defaultHomeSection as ContentType;
            initializedMode = true;
        }

        // Dejamos el título vacío para que el selector tenga espacio y sea el protagonista
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileHeaderAction;

        return () => {
            layoutState.headerAction = undefined;
        };
    });

    let currentContinueItems = $derived(continueItems.filter(item => item.contentType === currentMode));
    let currentTrending = $derived(content[currentMode].trending);
    let currentSeasonal = $derived(content[currentMode].seasonal);
    let currentTopRated = $derived(content[currentMode].topRated);

    const mapToContentWithMappings = (item: HomeMediaItem): ContentWithMappings => {
        return {
            content: { cid: item.cid, contentType: item.contentType, nsfw: false, createdAt: Date.now(), updatedAt: Date.now() },
            metadata: [{
                cid: item.cid, sourceName: 'anilist', title: item.title, altTitles: item.altTitles,
                synopsis: item.synopsis, coverImage: item.coverImage, bannerImage: item.bannerImage,
                subtype: item.format, status: item.status as any, releaseDate: item.releaseDate,
                endDate: item.endDate, rating: item.rating, genres: item.genres, tags: item.tags,
                trailerUrl: item.trailerUrl, characters: [], staff: [], externalIds: {}, createdAt: Date.now(), updatedAt: Date.now()
            }],
            trackerMappings: [], extensionSources: [], relations: [],
            contentUnits: []
        };
    };

    const mapSection = (section: MediaSection | undefined): MappedSection => ({
        trending: (section?.trending || []).map(mapToContentWithMappings),
        seasonal: (section?.seasonal || []).map(mapToContentWithMappings),
        topRated: (section?.topRated || []).map(mapToContentWithMappings)
    });

    onMount(() => {
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

</script>

<svelte:head>
    <title>{i18n.t("home.title")}</title>
</svelte:head>

{#snippet mobileHeaderAction()}
    <div class="flex items-center bg-muted/40 p-1 rounded-xl border border-border/40 shadow-inner h-10 mr-1" in:fade={{duration: 150}}>
        {#each modes as { id, label, icon: Icon }}
            <button
                    class="relative flex items-center justify-center px-3 sm:px-4 h-full rounded-lg text-xs font-bold transition-all duration-300 {currentMode === id ? 'bg-background text-primary shadow-sm ring-1 ring-border/30' : 'text-muted-foreground hover:text-foreground'}"
                    onclick={() => currentMode = id}
                    aria-label={label}
            >
                <Icon class="w-[18px] h-[18px] transition-transform duration-300 {currentMode === id ? 'scale-110' : ''} sm:mr-1.5" />

                <span class="hidden sm:inline-block tracking-wide">{label}</span>
            </button>
        {/each}
    </div>
{/snippet}

<div class="min-h-screen bg-background pb-20 overflow-x-hidden relative">

    <div class="hidden md:flex fixed top-6 left-1/2 -translate-x-1/2 z-30 items-center p-1.5 bg-background/80 backdrop-blur-xl border border-border/50 rounded-full shadow-lg transition-all">
        {#each modes as { id, label, icon: Icon }}
            <button
                    class="relative flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-bold transition-all duration-300 {currentMode === id ? 'bg-primary text-primary-foreground shadow-lg scale-105' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
                    onclick={() => currentMode = id}
            >
                <Icon class="size-4 shrink-0" />
                <span>{label}</span>
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
        <div in:fade={{ duration: 400 }}>
            {#if currentTrending.length > 0}
                <div class="pt-0 md:pt-0">
                    <Hero items={currentTrending.slice(0, 5)} />
                </div>
            {/if}

            <div class="w-full px-4 md:px-12 py-8 relative z-20 space-y-12 -mt-16 md:-mt-24">
                {#if currentContinueItems.length > 0}
                    <ContinueCarousel items={currentContinueItems} mode={currentMode} />
                {/if}

                {#if currentTrending.length > 0}
                    <ContentCarousel title={i18n.t("home.trending")} items={currentTrending} />
                {/if}

                {#if currentSeasonal.length > 0}
                    <ContentCarousel title={currentMode === 'anime' ? i18n.t("home.simulcast") : i18n.t("home.latest")} items={currentSeasonal} />
                {/if}

                {#if currentTopRated.length > 0}
                    <ContentCarousel title={i18n.t("home.critically_acclaimed")} items={currentTopRated} />
                {/if}
            </div>
        </div>
    {/if}
</div>