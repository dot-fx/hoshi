<script lang="ts">
    import { untrack } from "svelte";
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { extensions } from "$lib/extensions.svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import SearchFilters from "$lib/components/search/SearchFilters.svelte";
    import ContentCard from "@/components/content/Card.svelte";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Drawer from "$lib/components/ui/drawer";
    import * as Popover from "$lib/components/ui/popover";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Search, SearchX, Plug, SlidersHorizontal, Tv, Book, BookOpen, LayoutGrid, ListFilter, X, AlertCircle } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import { fade } from "svelte/transition";
    import { layoutState } from '$lib/layout.svelte';
    import { searchState } from '@/search.svelte.js';
    import type { CoreError } from "@/api/client";

    let isLoading = $state(false);
    let isSourcePopoverOpen = $state(false);
    let isDrawerOpen = $state(false);
    let isMobileSearchActive = $state(false);
    let extFiltersSchema = $state<Record<string, any>>({});

    let error = $state<CoreError | null>(null);

    let availableExtensions = $derived(
        searchState.contentType === "anime" ? extensions.anime :
            searchState.contentType === "manga" ? extensions.manga :
                searchState.contentType === "novel" ? extensions.novel : []
    );

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

    $effect(() => {
        layoutState.title = isMobileSearchActive ? "" : i18n.t('search.title');
        layoutState.showBack = false;
        layoutState.headerAction = mobileHeaderAction;
        return () => { layoutState.headerAction = undefined; };
    });

    $effect(() => {
        const currentExts = availableExtensions;
        untrack(() => {
            if (searchState.searchMode === "extension" && (!searchState.selectedExtension || !currentExts.find(e => e.id === searchState.selectedExtension))) {
                if (currentExts.length > 0) {
                    searchState.selectedExtension = currentExts[0].id;
                } else {
                    searchState.selectedExtension = "";
                    searchState.searchMode = "database";
                }
            }
            performSearch();
        });
    });

    $effect(() => {
        if (searchState.searchMode === "extension" && searchState.selectedExtension) {
            extensionsApi.getFilters(searchState.selectedExtension)
                .then(res => { extFiltersSchema = res.filters || {}; })
                .catch(() => { extFiltersSchema = {}; });
        } else {
            extFiltersSchema = {};
        }
    });

    const performSearch = async () => {
        isLoading = true;
        searchState.hasSearched = true;
        error = null;

        try {
            if (searchState.searchMode === "database") {
                const isSearchEmpty = !searchState.query.trim() && !searchState.dbStatus && !searchState.dbGenre && !searchState.dbFormat && !searchState.dbNsfw;
                if (isSearchEmpty) {
                    const res = await contentApi.getTrending(searchState.contentType);
                    searchState.results = res || [];
                } else {
                    let reqFormat = searchState.dbFormat;
                    if (!reqFormat) {
                        if (searchState.contentType === "novel") reqFormat = "NOVEL";
                        else if (searchState.contentType === "manga") reqFormat = "MANGA";
                    }
                    const res = await contentApi.search({
                        query: searchState.query,
                        type: searchState.contentType,
                        tracker: searchState.dbTracker,
                        ...(searchState.dbStatus && { status: searchState.dbStatus }),
                        ...(searchState.dbGenre && { genre: searchState.dbGenre }),
                        ...(reqFormat && { format: reqFormat }),
                        nsfw: searchState.dbNsfw
                    });
                    searchState.results = res.data || [];
                }
            } else if (searchState.searchMode === "extension" && searchState.selectedExtension) {
                const activeExtFilters = Object.fromEntries(
                    Object.entries(searchState.extFilterValues).filter(([_, v]) => {
                        if (Array.isArray(v)) return v.length > 0;
                        if (typeof v === 'string') return v.trim() !== "";
                        return v !== null && v !== undefined && v !== false;
                    })
                );

                const res = await contentApi.searchExtension(searchState.selectedExtension, {
                    query: searchState.query,
                    extensionFilters: Object.keys(activeExtFilters).length > 0 ? JSON.stringify(activeExtFilters) : undefined
                });

                searchState.results = res.results || res.data || [];
            }
        } catch (err) {
            console.error("Search error:", err);
            error = err as CoreError;
            if (!searchState.hasData) searchState.results = [];
        } finally {
            isLoading = false;
        }
    };

    function selectSource(mode: "database" | "extension", extId: string = "", tracker = "anilist", isMobile = false) {
        searchState.searchMode = mode;
        if (mode === "extension") searchState.selectedExtension = extId;
        else searchState.dbTracker = tracker;

        if (!isMobile) {
            isSourcePopoverOpen = false;
            performSearch();
        }
    }

    const clearFilters = () => {
        searchState.dbStatus = "";
        searchState.dbGenre = ""; searchState.dbFormat = ""; searchState.dbNsfw = false;
        searchState.extFilterValues = {};
        performSearch();
    };
</script>

<svelte:head>
    <title>{i18n.t('search.title')}</title>
</svelte:head>

{#snippet sourceGrid(isMobile: boolean)}
    <div class="grid {isMobile ? 'grid-cols-4 sm:grid-cols-5 md:grid-cols-6' : 'grid-cols-4'} gap-3">
        <button onclick={() => selectSource('database', '', 'anilist', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchState.searchMode === 'database' && searchState.dbTracker === 'anilist' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                <img src={getTrackerFavicon('anilist')} alt="AniList" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchState.searchMode === 'database' && searchState.dbTracker === 'anilist' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">AniList</span>
        </button>

        <button onclick={() => selectSource('database', '', 'mal', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchState.searchMode === 'database' && searchState.dbTracker === 'mal' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                <img src={getTrackerFavicon('mal')} alt="MyAnimeList" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchState.searchMode === 'database' && searchState.dbTracker === 'mal' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">MAL</span>
        </button>

        <button onclick={() => selectSource('database', '', 'kitsu', isMobile)} class="flex flex-col items-center gap-2 group) outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchState.searchMode === 'database' && searchState.dbTracker === 'kitsu' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                <img src={getTrackerFavicon('kitsu')} alt="Kitsu" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchState.searchMode === 'database' && searchState.dbTracker === 'kitsu' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">Kitsu</span>
        </button>

        {#each availableExtensions as ext}
            <button onclick={() => selectSource('extension', ext.id, 'anilist', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
                <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border overflow-hidden transition-all duration-300 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                    {#if ext.icon}
                        <img src={ext.icon} class="w-8 h-8 rounded-md object-contain transition-all duration-300 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" alt={ext.name} />
                    {:else}
                        <Plug class="w-6 h-6 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? 'text-primary' : 'text-muted-foreground'}" />
                    {/if}
                </div>
                <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">{ext.name}</span>
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
                        placeholder={i18n.t('search.placeholder', { type: i18n.t(searchState.contentType).toLowerCase() })}
                        class="pl-9 pr-3 h-9 text-sm rounded-full border-none bg-muted/30 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-inner"
                        bind:value={searchState.query}
                />
            </form>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full shrink-0" onclick={() => {
                isMobileSearchActive = false;
                if(searchState.query.trim() === '') performSearch();
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
                        {#if searchState.searchMode === 'extension' || searchState.dbStatus || searchState.dbGenre}
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
                                    <button onclick={() => searchState.contentType = 'anime'} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'anime' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.anime')}</button>
                                    <button onclick={() => searchState.contentType = 'manga'} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'manga' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.manga')}</button>
                                    <button onclick={() => searchState.contentType = 'novel'} class="h-10 rounded-lg text-sm font-bold transition-all {searchState.contentType === 'novel' ? 'bg-background shadow-sm text-primary' : 'text-muted-foreground hover:text-foreground'}">{i18n.t('search.novel')}</button>
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
                                        searchMode={searchState.searchMode}
                                        dbTracker={searchState.dbTracker}
                                        bind:dbStatus={searchState.dbStatus}
                                        bind:dbGenre={searchState.dbGenre}
                                        bind:dbFormat={searchState.dbFormat}
                                        bind:dbNsfw={searchState.dbNsfw}
                                        {extFiltersSchema}
                                        bind:extFilterValues={searchState.extFilterValues}
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

<div class="min-h-screen bg-background pb-28 md:pb-12 pt-16 md:pt-20 px-4 md:px-8 lg:pl-32 lg:pr-12 w-full max-w-[2000px] mx-auto space-y-10">
    <section class="flex flex-col lg:flex-row gap-8 lg:gap-10 w-full items-start">
        <aside class="hidden lg:block w-[260px] shrink-0 sticky top-24 h-fit">
            <div class="pb-6">
                <h3 class="font-black text-lg mb-6 flex items-center gap-2 text-foreground/90 tracking-tight">
                    <SlidersHorizontal class="w-5 h-5 text-primary" />
                    {i18n.t('search.filters')}
                </h3>
                <SearchFilters
                        searchMode={searchState.searchMode}
                        dbTracker={searchState.dbTracker}
                        bind:dbStatus={searchState.dbStatus}
                        bind:dbGenre={searchState.dbGenre}
                        bind:dbFormat={searchState.dbFormat}
                        bind:dbNsfw={searchState.dbNsfw}
                        {extFiltersSchema}
                        bind:extFilterValues={searchState.extFilterValues}
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
                            placeholder={i18n.t('search.placeholder', { type: i18n.t(searchState.contentType).toLowerCase() })}
                            class="pl-12 pr-4 h-12 text-base rounded-xl border border-border/40 bg-muted/10 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-sm"
                            bind:value={searchState.query}
                    />
                </form>

                <div class="flex flex-wrap items-center gap-2 sm:gap-3 shrink-0">
                    <Select.Root type="single" bind:value={searchState.contentType}>
                        <Select.Trigger class="w-[140px] bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold">
                            {#if searchState.contentType === "anime"}
                                <Tv class="w-4 h-4 mr-2 text-primary" />
                            {:else if searchState.contentType === "manga"}
                                <Book class="w-4 h-4 mr-2 text-primary" />
                            {:else}
                                <BookOpen class="w-4 h-4 mr-2 text-primary" />
                            {/if}
                            {i18n.t(searchState.contentType)}
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
                                    {#if searchState.searchMode === "database"}
                                        <img src={getTrackerFavicon(searchState.dbTracker)} alt={searchState.dbTracker} class="w-4 h-4 rounded-sm object-contain" />
                                        {searchState.dbTracker === 'mal' ? 'MyAnimeList' : searchState.dbTracker === 'kitsu' ? 'Kitsu' : 'AniList'}
                                    {:else}
                                        {@const ext = availableExtensions.find(e => e.id === searchState.selectedExtension)}
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
                        <Spinner class="w-10 h-10 animate-spin text-primary" />
                        <p class="text-sm font-bold animate-pulse">{i18n.t('search.searching')}</p>
                    </div>
                {:else if error}
                    <Empty.Root class="border border-dashed border-destructive/40 py-24 rounded-2xl bg-destructive/5 min-h-[50vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon" class="bg-destructive/10 text-destructive mb-4 p-4 rounded-full">
                                <AlertCircle class="w-10 h-10" />
                            </Empty.Media>
                            <Empty.Title class="text-xl font-bold text-destructive">
                                {i18n.t(error.key)}
                            </Empty.Title>
                            <Button variant="outline" class="mt-6 border-destructive/20 hover:bg-destructive/10 text-destructive" onclick={performSearch}>
                                {i18n.t("c.retry")}
                            </Button>
                        </Empty.Header>
                    </Empty.Root>
                {:else if searchState.hasSearched && searchState.results.length === 0}
                    <Empty.Root class="border border-dashed py-24 rounded-2xl bg-muted/5 min-h-[50vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon"><SearchX class="w-12 h-12" /></Empty.Media>
                            <Empty.Title class="text-2xl">{i18n.t('search.empty_title')}</Empty.Title>
                            <Empty.Description class="max-w-sm mx-auto text-base">
                                {i18n.t('search.empty_desc')}
                            </Empty.Description>
                        </Empty.Header>
                    </Empty.Root>
                    {:else if searchState.results.length > 0}
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 gap-x-4 gap-y-10 md:gap-x-5 md:gap-y-12">
                        {#each searchState.results as item (item.trackerId)}
                            <div in:fade={{ duration: 300 }}>
                                <ContentCard
                                        {item}
                                        source={searchState.searchMode === 'extension' ? searchState.selectedExtension : searchState.dbTracker}
                                />
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </section>
</div>