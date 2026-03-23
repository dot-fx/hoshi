<script lang="ts">
    import { untrack } from "svelte";
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { extensions } from "$lib/extensions.svelte";
    import type { ContentWithMappings, ContentType, HomeMediaItem, SearchTracker } from "$lib/api/content/types";
    import { i18n } from "$lib/i18n/index.svelte";
    import SearchFilters from "$lib/components/search/SearchFilters.svelte";
    import ContentCard from "@/components/content/Card.svelte";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Drawer from "$lib/components/ui/drawer";
    import * as Popover from "$lib/components/ui/popover";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Search, SearchX, Database, Plug, SlidersHorizontal, Tv, Book, BookOpen, Loader2, LayoutGrid, ListFilter, X } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { layoutState } from '$lib/layoutState.svelte';

    // --- State Variables ---
    let searchQuery = $state("");
    let contentType = $state<ContentType>("anime");
    let searchMode = $state<"database" | "extension">("database");

    let availableExtensions = $derived(
        contentType === "anime" ? extensions.anime :
            contentType === "manga" ? extensions.manga :
                contentType === "novel" ? extensions.novel : []
    );
    let selectedExtension = $state<string>("");

    // Estados de UI
    let isSourcePopoverOpen = $state(false); // Popover solo para PC
    let isDrawerOpen = $state(false); // Drawer único para Móvil (Tipo + Fuente + Filtros)
    let isMobileSearchActive = $state(false);

    let dbStatus = $state<string>("");
    let dbGenre = $state<string>("");
    let dbFormat = $state<string>("");
    let dbNsfw = $state<boolean>(false);
    let dbTracker = $state<SearchTracker>("anilist");

    let extFiltersSchema = $state<Record<string, any>>({});
    let extFilterValues = $state<Record<string, any>>({});
    let results = $state<ContentWithMappings[]>([]);
    let isLoading = $state(true);
    let hasSearched = $state(false);

    // --- Layout State Management ---
    $effect(() => {
        layoutState.title = isMobileSearchActive ? "" : i18n.t('search.title');
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileHeaderAction;

        return () => {
            layoutState.headerAction = undefined;
        };
    });

    const mapTrendingToMappings = (item: HomeMediaItem): ContentWithMappings => {
        return {
            content: {
                cid: item.cid,
                contentType: item.contentType,
                nsfw: false,
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
                trailerUrl: item.trailerUrl,
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

    $effect(() => {
        const currentExts = availableExtensions;

        untrack(() => {
            if (searchMode === "extension" && (!selectedExtension || !currentExts.find(e => e.id === selectedExtension))) {
                if (currentExts.length > 0) {
                    selectedExtension = currentExts[0].id;
                } else {
                    selectedExtension = "";
                    searchMode = "database";
                }
            }
            searchQuery = "";
            performSearch();
        });
    });

    $effect(() => {
        if (searchMode === "extension" && selectedExtension) {
            extensionsApi.getFilters(selectedExtension)
                .then(res => {
                    extFiltersSchema = res.filters || {};
                    extFilterValues = {};
                })
                .catch(() => { extFiltersSchema = {}; });
        } else {
            extFiltersSchema = {};
            extFilterValues = {};
        }
    });

    function selectSource(mode: "database" | "extension", extId: string = "", tracker: SearchTracker = "anilist", isMobile = false) {
        searchMode = mode;
        if (mode === "extension") {
            selectedExtension = extId;
        } else {
            dbTracker = tracker;
        }

        if (!isMobile) {
            isSourcePopoverOpen = false;
            performSearch();
        }
    }

    const performSearch = async () => {
        isLoading = true;
        hasSearched = true;
        results = [];

        try {
            if (searchMode === "database") {
                const isSearchEmpty = !searchQuery.trim() && !dbStatus && !dbGenre && !dbFormat && !dbNsfw;
                if (isSearchEmpty) {
                    const res = await contentApi.getTrending(contentType);
                    results = (res || []).map(mapTrendingToMappings);
                } else {
                    let requestFormat = dbFormat;
                    if (!dbFormat) {
                        if (contentType === "novel") requestFormat = "NOVEL";
                        else if (contentType === "manga") requestFormat = "MANGA";
                    }
                    const res = await contentApi.search({
                        query: searchQuery,
                        type: contentType,
                        tracker: dbTracker,
                        ...(dbStatus && { status: dbStatus }),
                        ...(dbGenre && { genre: dbGenre }),
                        ...(requestFormat && { format: requestFormat }),
                        nsfw: dbNsfw
                    });
                    results = res.data ? res.data : [];
                }

            } else if (searchMode === "extension" && selectedExtension) {
                const activeExtFilters = Object.fromEntries(
                    Object.entries(extFilterValues).filter(([_, v]) => {
                        if (Array.isArray(v)) return v.length > 0;
                        if (typeof v === 'string') return v.trim() !== "";
                        if (typeof v === 'boolean') return v === true;
                        if (v === null || v === undefined) return false;
                        return true;
                    })
                );
                const currentExt = availableExtensions.find(e => e.id === selectedExtension);
                const skipProcessing = currentExt?.skip_default_processing === true;

                if (skipProcessing) {
                    const res = await contentApi.search({
                        query: searchQuery,
                        type: contentType,
                        extension: selectedExtension,
                        extensionFilters: Object.keys(activeExtFilters).length > 0
                            ? JSON.stringify(activeExtFilters)
                            : undefined
                    });
                    results = res.data ? res.data : [];
                } else {
                    const res = await contentApi.searchExtension(selectedExtension, {
                        query: searchQuery,
                        extensionFilters: Object.keys(activeExtFilters).length > 0
                            ? JSON.stringify(activeExtFilters)
                            : undefined
                    });
                    const rawResults = Array.isArray(res.results) ? res.results : [];

                    results = rawResults.map((item: any) => {
                        const cid = `ext:${selectedExtension}:${item.id}`;
                        return {
                            content: {
                                cid: cid,
                                contentType: contentType,
                                nsfw: false,
                                createdAt: Date.now(),
                                updatedAt: Date.now()
                            },
                            metadata: [{
                                cid: cid,
                                sourceName: selectedExtension,
                                title: item.title,
                                coverImage: item.image,
                                characters: [],
                                staff: [],
                                externalIds: { [selectedExtension]: item.id },
                                createdAt: Date.now(),
                                updatedAt: Date.now()
                            }],
                            trackerMappings: [],
                            extensionSources: [],
                            relations: [],
                            contentUnits: []
                        } as ContentWithMappings;
                    });
                }
            }
        } catch (error) {
            console.error("Search error:", error);
            results = [];
        } finally {
            isLoading = false;
        }
    };

    const clearFilters = () => {
        dbStatus = ""; dbGenre = ""; dbFormat = ""; dbNsfw = false;
        for (const key in extFiltersSchema) {
            if (extFiltersSchema[key].type === 'multiselect') {
                extFilterValues[key] = [];
            } else {
                extFilterValues[key] = "";
            }
        }
        performSearch();
    };
    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            'anilist': 'anilist.co',
            'mal': 'myanimelist.net',
            'kitsu': 'kitsu.io',
            'simkl': 'simkl.com'
        };

        const domain = domains[trackerName.toLowerCase()] || 'google.com';
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }
</script>

<svelte:head>
    <title>{i18n.t('search.title')}</title>
</svelte:head>

{#snippet sourceGrid(isMobile: boolean)}
    <div class="grid {isMobile ? 'grid-cols-4 sm:grid-cols-5 md:grid-cols-6' : 'grid-cols-4'} gap-3">

        <button onclick={() => selectSource('database', '', 'anilist', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchMode === 'database' && dbTracker === 'anilist' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                <img src={getTrackerFavicon('anilist')} alt="AniList" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchMode === 'database' && dbTracker === 'anilist' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">AniList</span>
        </button>

        <button onclick={() => selectSource('database', '', 'mal', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchMode === 'database' && dbTracker === 'mal' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                <img src={getTrackerFavicon('mal')} alt="MyAnimeList" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchMode === 'database' && dbTracker === 'mal' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">MAL</span>
        </button>

        <button onclick={() => selectSource('database', '', 'kitsu', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchMode === 'database' && dbTracker === 'kitsu' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                <img src={getTrackerFavicon('kitsu')} alt="Kitsu" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchMode === 'database' && dbTracker === 'kitsu' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">Kitsu</span>
        </button>

        {#each availableExtensions as ext}
            <button onclick={() => selectSource('extension', ext.id, 'anilist', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
                <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border overflow-hidden transition-all duration-300 {searchMode === 'extension' && selectedExtension === ext.id ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                    {#if ext.icon}
                        <img src={ext.icon} class="w-8 h-8 rounded-md object-contain transition-all duration-300 {searchMode === 'extension' && selectedExtension === ext.id ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" alt={ext.name} />
                    {:else}
                        <Plug class="w-6 h-6 {searchMode === 'extension' && selectedExtension === ext.id ? 'text-primary' : 'text-muted-foreground'}" />
                    {/if}
                </div>
                <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate" title={ext.name}>{ext.name}</span>
            </button>
        {/each}
    </div>
{/snippet}

{#snippet mobileHeaderAction()}
    {#if isMobileSearchActive}
        <div class="flex items-center gap-1 w-full pl-2" in:fade={{duration: 150}}>
            <form onsubmit={(e) => { e.preventDefault(); performSearch(); }} class="relative w-full group">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground focus-within:text-primary transition-colors" />
                <Input
                        id="mobile-search-input"
                        type="text"
                        placeholder={i18n.t('search.placeholder', { type: i18n.t(contentType).toLowerCase() })}
                        class="pl-9 pr-3 h-9 text-sm rounded-full border-none bg-muted/30 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-inner"
                        bind:value={searchQuery}
                />
            </form>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full shrink-0" onclick={() => {
                isMobileSearchActive = false;
                if(searchQuery.trim() === '') performSearch();
            }}>
                <X class="w-[22px] h-[22px] text-foreground" />
            </Button>
        </div>
    {:else}
        <div class="flex items-center text-foreground gap-0.5" in:fade={{duration: 150}}>

            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50" onclick={() => {
                isMobileSearchActive = true;
                setTimeout(() => document.getElementById('mobile-search-input')?.focus(), 50);
            }}>
                <Search class="w-[22px] h-[22px]" />
            </Button>

            <Drawer.Root bind:open={isDrawerOpen}>
                <Drawer.Trigger>
                    <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50 relative">
                        {#if searchMode === 'extension' || dbStatus || dbGenre}
                            <div class="absolute top-2 right-2 w-2 h-2 bg-primary rounded-full"></div>
                        {/if}
                        <ListFilter class="w-[22px] h-[22px]" />
                    </Button>
                </Drawer.Trigger>

                <Drawer.Content class="h-[85vh] rounded-t-2xl border-border/50">
                    <div class="w-full h-full flex flex-col overflow-hidden">

                        <div class="flex-1 p-6 overflow-y-auto hide-scrollbar flex flex-col gap-8 pb-6">

                            <h3 class="font-black text-2xl tracking-tight flex items-center gap-2">
                                <ListFilter class="w-5 h-5 text-primary" />
                                {i18n.t('search.search_settings')}
                            </h3>

                            <div class="space-y-3">
                                <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider">{i18n.t('search.type')}</h4>
                                <div class="bg-muted/20 p-1.5 rounded-xl grid grid-cols-3 gap-1">
                                    <button onclick={() => contentType = 'anime'} class="h-10 rounded-lg text-sm font-bold transition-all {contentType === 'anime' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.anime')}</button>
                                    <button onclick={() => contentType = 'manga'} class="h-10 rounded-lg text-sm font-bold transition-all {contentType === 'manga' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.manga')}</button>
                                    <button onclick={() => contentType = 'novel'} class="h-10 rounded-lg text-sm font-bold transition-all {contentType === 'novel' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.novel')}</button>
                                </div>
                            </div>

                            <div class="space-y-3">
                                <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider">{i18n.t('search.source')}</h4>
                                {@render sourceGrid(true)}
                            </div>

                            <div class="space-y-3">
                                <div class="flex items-center justify-between">
                                    <h4 class="text-xs font-bold text-muted-foreground uppercase tracking-wider">{i18n.t('search.filters')}</h4>
                                    <button onclick={clearFilters} class="text-xs font-semibold text-primary/80 hover:text-primary">{i18n.t('search.clear')}</button>
                                </div>
                                <SearchFilters
                                        {searchMode}
                                        {dbTracker} bind:dbStatus
                                        bind:dbGenre
                                        bind:dbFormat
                                        bind:dbNsfw
                                        {extFiltersSchema}
                                        bind:extFilterValues
                                        onClear={clearFilters}
                                />
                            </div>
                        </div>

                        <div class="shrink-0 p-4 bg-background border-t border-border/40 pb-safe z-10">
                            <Button class="w-full h-12 rounded-xl font-bold text-base shadow-sm" onclick={() => { performSearch(); isDrawerOpen = false; }}>
                                {i18n.t('search.apply_search')}
                            </Button>
                        </div>

                    </div>
                </Drawer.Content>
            </Drawer.Root>
        </div>
    {/if}
{/snippet}

<main class="min-h-screen bg-background pb-28 md:pb-10 pt-4 md:pt-12 px-4 md:px-6 lg:px-8 xl:px-10 w-full max-w-[2400px] mx-auto space-y-6 md:space-y-8">
    <section class="flex flex-col lg:flex-row gap-8 lg:gap-10 w-full items-start">

        <aside class="hidden lg:block w-64 xl:w-72 shrink-0">
            <div class="pb-6">
                <h3 class="font-black text-lg mb-6 flex items-center gap-2 text-foreground/90 tracking-tight">
                    <SlidersHorizontal class="w-5 h-5 text-primary" />
                    {i18n.t('search.filters')}
                </h3>
                <SearchFilters
                        {searchMode}
                        bind:dbStatus
                        bind:dbGenre
                        bind:dbFormat
                        bind:dbNsfw
                        {extFiltersSchema}
                        bind:extFilterValues
                        onClear={clearFilters}
                />
            </div>
        </aside>

        <div class="flex-1 min-w-0 w-full flex flex-col gap-6">

            <div class="hidden md:flex flex-col xl:flex-row gap-4 items-start xl:items-center justify-between w-full">

                <form onsubmit={(e) => { e.preventDefault(); performSearch(); }} class="relative w-full xl:flex-1 group">
                    <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <Input
                            type="text"
                            placeholder={i18n.t('search.placeholder', { type: i18n.t(contentType).toLowerCase() })}
                            class="pl-12 pr-4 h-12 text-base rounded-xl border border-border/40 bg-muted/10 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-sm"
                            bind:value={searchQuery}
                    />
                </form>

                <div class="flex flex-wrap items-center gap-2 sm:gap-3 shrink-0">

                    <Select.Root type="single" bind:value={contentType}>
                        <Select.Trigger class="w-[140px] bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold">
                            {#if contentType === "anime"}
                                <Tv class="w-4 h-4 mr-2 text-primary" />
                            {:else if contentType === "manga"}
                                <Book class="w-4 h-4 mr-2 text-primary" />
                            {:else}
                                <BookOpen class="w-4 h-4 mr-2 text-primary" />
                            {/if}
                            {i18n.t(contentType)}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="anime">{i18n.t('search.anime')}</Select.Item>
                            <Select.Item value="manga">{i18n.t('search.manga')}</Select.Item>
                            <Select.Item value="novel">{i18n.t('search.novel')}</Select.Item>
                        </Select.Content>
                    </Select.Root>

                    <Popover.Root bind:open={isSourcePopoverOpen}>
                        <Popover.Trigger>
                            {#snippet child({ props })}
                                <Button {...props} variant="secondary" class="h-11 rounded-xl text-sm font-semibold gap-2 border-none bg-muted/20 hover:bg-muted/30 px-4">
                                    {#if searchMode === "database"}
                                        <img src={getTrackerFavicon(dbTracker)} alt={dbTracker} class="w-4 h-4 rounded-sm object-contain" />
                                        {dbTracker === 'mal' ? 'MyAnimeList' : dbTracker === 'kitsu' ? 'Kitsu' : 'AniList'}
                                    {:else}
                                        {@const ext = availableExtensions.find(e => e.id === selectedExtension)}
                                        {#if ext?.icon}
                                            <img src={ext.icon} class="w-5 h-5 rounded-md object-cover" alt={ext?.name} />
                                        {:else}
                                            <Plug class="w-4 h-4 text-primary" />
                                        {/if}
                                        {ext?.name}
                                    {/if}
                                </Button>
                            {/snippet}
                        </Popover.Trigger>
                        <Popover.Content align="start" class="w-[360px] p-5 rounded-2xl border-border/50 shadow-2xl bg-card">
                            <h3 class="font-black text-xs text-muted-foreground uppercase tracking-widest mb-4 flex items-center gap-2">
                                <LayoutGrid class="w-4 h-4" /> {i18n.t('search.select_source')}
                            </h3>
                            {@render sourceGrid(false)}
                        </Popover.Content>
                    </Popover.Root>

                </div>
            </div>

            <div class="hidden md:block w-full border-t border-border/40 mt-2 mb-2"></div>

            <div class="w-full">
                {#if isLoading}
                    <div class="flex flex-col items-center justify-center w-full min-h-[50vh] text-muted-foreground space-y-4">
                        <Loader2 class="w-10 h-10 animate-spin text-primary" />
                        <p class="text-sm font-bold animate-pulse">{i18n.t('search.searching')}</p>
                    </div>

                {:else if hasSearched && results.length === 0}
                    <Empty.Root class="border border-dashed py-24 rounded-2xl bg-muted/5 min-h-[50vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon"><SearchX class="w-12 h-12" /></Empty.Media>
                            <Empty.Title class="text-2xl">{i18n.t('search.empty_title')}</Empty.Title>
                            <Empty.Description class="max-w-sm mx-auto text-base">
                                {i18n.t('search.empty_desc')}
                            </Empty.Description>
                        </Empty.Header>
                    </Empty.Root>

                {:else if results.length > 0}
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-10 md:gap-x-5 md:gap-y-12">
                        {#each results as item (item.content.cid)}
                            <div in:fade={{ duration: 300 }}>
                                <ContentCard {item} />
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </section>
</main>

<style>
    .pb-safe {
        padding-bottom: calc(env(safe-area-inset-bottom) + 1rem);
    }
</style>