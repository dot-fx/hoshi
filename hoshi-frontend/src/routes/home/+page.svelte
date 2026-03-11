<script lang="ts">
    import Hero from '$lib/components/content/ContentHero.svelte';
    import ContentCarousel from '$lib/components/home/ContentCarousel.svelte';
    import { Skeleton } from '$lib/components/ui/skeleton';
    import { fade } from 'svelte/transition';
    import { contentApi } from '@/api/content/content';
    import type { ContentWithMappings, ContentType, HomeMediaItem, HomeView, MediaSection } from '@/api/content/types';

    let loading = $state(true);
    let error = $state(false);

    // Estado para controlar el modo actual
    let currentMode = $state<ContentType>('anime');

    // Estructura adaptada para los 3 modos
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

    // Actualizado para usar HomeMediaItem
    const mapToContentWithMappings = (item: HomeMediaItem): ContentWithMappings => {
        return {
            content: {
                cid: item.cid,
                contentType: item.contentType,
                nsfw: false, // O el valor por defecto que prefieras
                createdAt: Date.now(),
                updatedAt: Date.now()
            },
            metadata: [{
                cid: item.cid,
                sourceName: 'anilist',
                title: item.title,
                altTitles: item.altTitles,
                synopsis: item.synopsis,
                coverImage: item.coverImage,
                bannerImage: item.bannerImage,
                subtype: item.format,
                status: item.status as any,
                releaseDate: item.releaseDate,
                endDate: item.endDate,
                rating: item.rating,
                genres: item.genres,
                tags: item.tags,
                trailerUrl: item.trailerUrl, // <--- ¡AQUÍ ESTÁ EL FIX!
                characters: [],
                staff: [],
                externalIds: {},
                createdAt: Date.now(),
                updatedAt: Date.now()
            }],
            trackerMappings: [],
            extensionSources: [],
            relations: [],
            contentUnits: []
        };
    };

    const mapSection = (section: MediaSection | undefined): MappedSection => ({
        trending: (section?.trending || []).map(mapToContentWithMappings),
        seasonal: (section?.seasonal || []).map(mapToContentWithMappings),
        topRated: (section?.topRated || []).map(mapToContentWithMappings)
    });

    $effect(() => {
        contentApi.getHome()
            .then((res: HomeView) => {
                content = {
                    anime: mapSection(res.anime),
                    manga: mapSection(res.manga),
                    novel: mapSection(res.novel)
                };
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
    <title>Home</title>
</svelte:head>

<div class="min-h-screen bg-background pb-20 overflow-x-hidden relative">

    <!-- Selector de Modo (Flotante o estático según prefieras) -->
    <div class="absolute top-4 left-0 right-0 z-50 flex justify-center gap-2 md:gap-4 pointer-events-auto">
        {#each ['anime', 'manga', 'novel'] as mode}
            <button
                    class="px-4 py-2 rounded-full text-sm font-medium transition-colors backdrop-blur-md
                       {currentMode === mode ? 'bg-primary text-primary-foreground shadow-lg' : 'bg-background/50 hover:bg-background/80 text-foreground'}"
                    onclick={() => currentMode = mode}
            >
                {mode.charAt(0).toUpperCase() + mode.slice(1)}
            </button>
        {/each}
    </div>

    {#if loading}
        <div class="w-full h-[85vh] bg-card/50 flex items-end p-4 md:p-12 animate-pulse pt-20">
            <div class="space-y-6 max-w-3xl w-full mb-10">
                <Skeleton class="h-12 md:h-20 w-3/4 bg-muted rounded-lg" />
                <div class="flex gap-4">
                    <Skeleton class="h-6 w-20 bg-muted rounded" />
                    <Skeleton class="h-6 w-20 bg-muted rounded" />
                    <Skeleton class="h-6 w-20 bg-muted rounded" />
                </div>
                <div class="space-y-2">
                    <Skeleton class="h-4 w-full bg-muted rounded" />
                    <Skeleton class="h-4 w-2/3 bg-muted rounded" />
                </div>
                <div class="flex gap-4 pt-4">
                    <Skeleton class="h-14 w-32 bg-muted rounded" />
                    <Skeleton class="h-14 w-32 bg-muted rounded" />
                </div>
            </div>
        </div>

    {:else if error}
        <div class="h-screen w-full flex flex-col items-center justify-center text-muted-foreground gap-4">
            <p class="text-lg">Failed to load content.</p>
            <button class="text-white hover:underline" onclick={() => location.reload()}>
                Try again
            </button>
        </div>

    {:else}
        {#key currentMode}
            <div in:fade={{ duration: 300 }}>
                {#if content[currentMode].trending.length > 0}
                    <Hero items={content[currentMode].trending.slice(0, 5)} />
                {/if}

                <div class="w-full px-4 md:px-12 py-8 relative z-20 space-y-12 -mt-16 md:-mt-24">
                    {#if content[currentMode].trending.length > 0}
                        <ContentCarousel title="Trending Now" items={content[currentMode].trending} />
                    {/if}

                    <!-- El seasonal suele ser solo para anime, pero el check de .length previene que se renderice vacío en mangas/novelas -->
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