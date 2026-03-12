<script lang="ts">
    import Hero from '$lib/components/content/ContentHero.svelte';
    import ContentCarousel from '$lib/components/home/ContentCarousel.svelte';
    import { Skeleton } from '$lib/components/ui/skeleton';
    import { Progress } from "$lib/components/ui/progress";
    import { fade } from 'svelte/transition';
    import { contentApi } from '@/api/content/content';
    import { progressApi } from '@/api/progress/progress';
    import type { ContentWithMappings, ContentType, HomeMediaItem, MediaSection } from '@/api/content/types';
    import type { ContinueItem } from '@/api/progress/types';
    import { appConfig } from "@/config.svelte";
    import { Tv, Book, BookText, PlayCircle, FileText } from "lucide-svelte";
    import { layoutState } from '$lib/layoutState.svelte';

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

    function getContinueUrl(item: ContinueItem) {
        if (item.contentType === 'anime' && item.episode) {
            const ratio = (item.episodeDurationSeconds && item.timestampSeconds)
                ? item.timestampSeconds / item.episodeDurationSeconds
                : 0;

            if (ratio >= 0.95) {
                return `/watch/${item.cid}/${item.episode + 1}`;
            }
            else if (item.timestampSeconds && item.timestampSeconds > 0) {
                return `/watch/${item.cid}/${item.episode}?t=${item.timestampSeconds}`;
            }
            else {
                return `/watch/${item.cid}/${item.episode}`;
            }
        }

        // Para Manga/Novela lo mandamos a la info del contenido de momento
        return `/content/${item.cid}`;
    }

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
            trackerMappings: [], extensionSources: [], relations: [], contentUnits: []
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
    <title>Home</title>
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
            <p class="text-lg font-bold">Failed to load content.</p>
            <button class="text-primary hover:underline font-medium" onclick={() => location.reload()}>Try again</button>
        </div>
    {:else}
        {#key currentMode}
            <div in:fade={{ duration: 300 }}>
                {#if content[currentMode].trending.length > 0}
                    <Hero items={content[currentMode].trending.slice(0, 5)} />
                {/if}

                <div class="w-full px-4 md:px-12 py-8 relative z-20 space-y-12 -mt-16 md:-mt-24">

                    {#if currentContinueItems.length > 0}
                        <div class="space-y-4">
                            <h2 class="text-xl md:text-2xl font-black tracking-tight flex items-center gap-2 text-foreground">
                                {currentMode === 'anime' ? 'Seguir Viendo' : 'Seguir Leyendo'}
                            </h2>
                            <div class="flex overflow-x-auto gap-4 pb-4 custom-scrollbar snap-x">
                                {#each currentContinueItems as item}
                                    <a
                                            href={getContinueUrl(item)}
                                            class="group flex items-center gap-4 p-3 bg-card/90 backdrop-blur-md border border-border/60 hover:border-primary/50 rounded-2xl min-w-[280px] max-w-[320px] snap-start transition-all hover:shadow-md"
                                    >
                                        <div class="relative h-24 w-16 shrink-0 rounded-lg overflow-hidden bg-muted/50 border border-border/50">
                                            {#if item.coverImage}
                                                <img src={item.coverImage} alt={item.title} class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                                            {:else}
                                                <div class="w-full h-full flex items-center justify-center opacity-30 text-muted-foreground">
                                                    {#if currentMode === 'anime'} <PlayCircle class="size-6" /> {:else} <FileText class="size-6" /> {/if}
                                                </div>
                                            {/if}
                                        </div>

                                        <div class="flex flex-col flex-1 min-w-0 h-full py-0.5">
                                            <h3 class="font-bold text-sm leading-tight line-clamp-2 mb-1 group-hover:text-primary transition-colors" title={item.title}>{item.title}</h3>
                                            <div class="mt-auto">
                                                {#if currentMode === 'anime' && item.episode}
                                                    <div class="flex items-center justify-between mb-2">
                                                        <span class="text-xs font-bold text-muted-foreground">Episodio {item.episode}</span>
                                                    </div>
                                                    {#if item.episodeDurationSeconds && item.timestampSeconds}
                                                        <Progress value={(item.timestampSeconds / item.episodeDurationSeconds) * 100} max={100} class="h-1.5 w-full bg-muted" />
                                                    {/if}
                                                {:else if item.chapter}
                                                    <span class="text-xs font-bold text-muted-foreground bg-foreground/5 px-2 py-1 rounded-md">Capítulo {item.chapter}</span>
                                                {/if}
                                            </div>
                                        </div>
                                    </a>
                                {/each}
                            </div>
                        </div>
                    {/if}

                    {#if content[currentMode].trending.length > 0}
                        <ContentCarousel title="Trending Now" items={content[currentMode].trending} />
                    {/if}
                    {#if content[currentMode].seasonal.length > 0}
                        <ContentCarousel title={currentMode === 'anime' ? "Simulcast Season" : "Latest Releases"} items={content[currentMode].seasonal} />
                    {/if}
                    {#if content[currentMode].topRated.length > 0}
                        <ContentCarousel title="Critically Acclaimed" items={content[currentMode].topRated} />
                    {/if}
                </div>
            </div>
        {/key}
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { height: 6px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150,150,150,0.3); border-radius: 10px; }
</style>