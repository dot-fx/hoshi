<script lang="ts">
    import { untrack } from "svelte";
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { extensions } from "$lib/extensions.svelte"; // <-- IMPORTAMOS EL STORE GLOBAL
    import type { ContentWithMappings, ContentType, HomeMediaItem } from "$lib/api/content/types";
    import { i18n } from "$lib/i18n/index.svelte";

    import ContentCard from "$lib/components/home/ContentCard.svelte";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";

    import { Search, SearchX, Database, Plug, SlidersHorizontal, Tv, Book, BookOpen, Loader2 } from "lucide-svelte";
    import { fade } from "svelte/transition";

    // --- State Variables ---
    let searchQuery = $state("");
    let contentType = $state<ContentType>("anime");
    let searchMode = $state<"database" | "extension">("database");

    // Derivamos las extensiones disponibles directamente del STORE GLOBAL según el tipo de contenido
    let availableExtensions = $derived(
        contentType === "anime" ? extensions.anime.map(e => e.id) :
            contentType === "manga" ? extensions.manga.map(e => e.id) :
                contentType === "novel" ? extensions.novel.map(e => e.id) : []
    );

    let selectedExtension = $state<string>("");

    // Filters State
    let dbStatus = $state<string>("");
    let dbGenre = $state<string>("");
    let dbFormat = $state<string>("");
    let dbNsfw = $state<boolean>(false);

    let extFiltersSchema = $state<Record<string, any>>({});
    let extFilterValues = $state<Record<string, any>>({});

    let results = $state<ContentWithMappings[]>([]);
    let isLoading = $state(true);
    let hasSearched = $state(false);

    let isDrawerOpen = $state(false);

    const formatLabel = (key: string) => key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());

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
            if (currentExts.length > 0 && !currentExts.includes(selectedExtension)) {
                selectedExtension = currentExts[0];
            } else if (currentExts.length === 0) {
                selectedExtension = "";
                searchMode = "database";
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

    const toggleMultiSelect = (key: string, value: string) => {
        if (!extFilterValues[key]) extFilterValues[key] = [];
        const index = extFilterValues[key].indexOf(value);
        if (index > -1) {
            extFilterValues[key] = extFilterValues[key].filter((v: string) => v !== value);
        } else {
            extFilterValues[key] = [...extFilterValues[key], value];
        }
    };

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
</script>

{#snippet filterFields()}
    <div class="space-y-6 w-full">
        {#if searchMode === "database"}
            <div class="space-y-5">
                <div class="space-y-2.5">
                    <Label class="text-sm font-bold text-foreground/90">{i18n.t('status')}</Label>
                    <Select.Root type="single" bind:value={dbStatus}>
                        <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                            {dbStatus ? i18n.t(dbStatus.toLowerCase()) || dbStatus : i18n.t('any_status')}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="">{i18n.t('any_status')}</Select.Item>
                            <Select.Item value="Completed">{i18n.t('completed')}</Select.Item>
                            <Select.Item value="Ongoing">{i18n.t('ongoing')}</Select.Item>
                            <Select.Item value="Planned">{i18n.t('planned')}</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="space-y-2.5">
                    <Label class="text-sm font-bold text-foreground/90">{i18n.t('genre')}</Label>
                    <Select.Root type="single" bind:value={dbGenre}>
                        <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                            {dbGenre ? i18n.t(dbGenre.toLowerCase().replace('-', '_')) || dbGenre : i18n.t('any_genre')}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="">{i18n.t('any_genre')}</Select.Item>
                            <Select.Item value="Action">{i18n.t('action')}</Select.Item>
                            <Select.Item value="Romance">{i18n.t('romance')}</Select.Item>
                            <Select.Item value="Fantasy">{i18n.t('fantasy')}</Select.Item>
                            <Select.Item value="Sci-Fi">{i18n.t('sci_fi')}</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="space-y-2.5">
                    <Label class="text-sm font-bold text-foreground/90">{i18n.t('format')}</Label>
                    <Select.Root type="single" bind:value={dbFormat}>
                        <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                            {dbFormat ? i18n.t(dbFormat.toLowerCase()) || dbFormat : i18n.t('any_format')}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="">{i18n.t('any_format')}</Select.Item>
                            <Select.Item value="TV">{i18n.t('tv')}</Select.Item>
                            <Select.Item value="MOVIE">{i18n.t('movie')}</Select.Item>
                            <Select.Item value="OVA">{i18n.t('ova')}</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="flex items-center space-x-3 pt-2">
                    <Switch id="nsfw-mode" bind:checked={dbNsfw} />
                    <Label for="nsfw-mode" class="text-sm font-bold text-foreground/90">{i18n.t('nsfw_only')}</Label>
                </div>
            </div>

        {:else if searchMode === "extension" && Object.keys(extFiltersSchema).length > 0}
            <div class="space-y-5">
                {#each Object.entries(extFiltersSchema) as [key, filterDef]}
                    <div class="space-y-2.5">
                        <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>

                        {#if filterDef.type === 'select'}
                            <Select.Root type="single" bind:value={extFilterValues[key]}>
                                <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                                    {filterDef.options.find((o) => o.value === extFilterValues[key])?.label || i18n.t('select')}
                                </Select.Trigger>
                                <Select.Content class="max-h-[300px]">
                                    {#each filterDef.options as option}
                                        <Select.Item value={String(option.value)}>{option.label}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>

                        {:else if filterDef.type === 'multiselect'}
                            <div class="flex flex-wrap gap-2 pt-1">
                                {#each filterDef.options as option}
                                    <button
                                            type="button"
                                            class="px-3.5 py-1.5 text-xs font-bold rounded-lg border transition-colors shadow-sm
                                        {extFilterValues[key]?.includes(option.value)
                                            ? 'bg-primary text-primary-foreground border-primary'
                                            : 'bg-background hover:bg-muted border-border/60'}"
                                            onclick={() => toggleMultiSelect(key, option.value)}
                                    >
                                        {option.label}
                                    </button>
                                {/each}
                            </div>

                        {:else if filterDef.type === 'boolean'}
                            <div class="flex items-center space-x-3 pt-2">
                                <Switch id={`filter-${key}`} bind:checked={extFilterValues[key]} />
                                <Label for={`filter-${key}`} class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                            </div>

                        {:else}
                            <Input
                                    type="text"
                                    placeholder={`${i18n.t('enter')} ${filterDef.label?.toLowerCase() || formatLabel(key).toLowerCase()}...`}
                                    class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50"
                                    bind:value={extFilterValues[key]}
                            />
                        {/if}
                    </div>
                {/each}
            </div>

        {:else}
            <div class="py-8 text-center bg-muted/5 rounded-xl border border-dashed border-border/50">
                <p class="text-muted-foreground text-sm font-medium">{i18n.t('no_specific_filters')}</p>
            </div>
        {/if}

        <div class="pt-6 border-t border-border/40">
            <Button type="button" variant="secondary" class="w-full h-11 rounded-xl font-bold hover:bg-destructive hover:text-destructive-foreground transition-colors" onclick={clearFilters}>
                {i18n.t('clear_filters')}
            </Button>
        </div>
    </div>
{/snippet}

<svelte:head>
    <title>{i18n.t('search')}</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-10 pt-6 md:pt-8 px-4 md:px-6 lg:px-8 xl:px-10 w-full max-w-[2400px] mx-auto space-y-6 md:space-y-8">

    <header class="flex flex-col md:flex-row md:items-center justify-between gap-5 md:gap-6 border-b border-border/40 pb-6 w-full">
        <div class="space-y-1">
            <h1 class="text-3xl md:text-4xl font-black tracking-tight flex items-center gap-3">
                <Search class="h-8 w-8 md:h-10 md:w-10 text-primary" />
                {i18n.t('discover')}
            </h1>
            <p class="text-sm md:text-base text-muted-foreground font-medium opacity-80">
                {i18n.t('search_for')} {i18n.t(contentType).toLowerCase()} {i18n.t('titles')}
            </p>
        </div>
    </header>

    <section class="flex flex-col lg:flex-row gap-8 lg:gap-10 w-full items-start">

        <!-- Sidebar Desktop -->
        <aside class="hidden lg:block w-64 xl:w-72 shrink-0">
            <div class="pb-6">
                <h3 class="font-black text-lg mb-6 flex items-center gap-2 text-foreground/90 tracking-tight">
                    <SlidersHorizontal class="w-5 h-5 text-primary" />
                    {i18n.t('filters')}
                </h3>
                {@render filterFields()}
            </div>
        </aside>

        <!-- Zona de Contenido Principal -->
        <div class="flex-1 min-w-0 w-full flex flex-col gap-6">

            <div class="flex flex-col 2xl:flex-row gap-4 items-start 2xl:items-center justify-between w-full">

                <form onsubmit={(e) => { e.preventDefault(); performSearch(); }} class="relative w-full 2xl:max-w-md group">
                    <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <Input
                            type="text"
                            placeholder={`${i18n.t('search_for')} ${i18n.t(contentType).toLowerCase()}...`}
                            class="pl-12 pr-28 h-12 text-base rounded-xl border border-border/40 bg-muted/10 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-sm"
                            bind:value={searchQuery}
                    />
                    <Button type="submit" class="absolute right-1.5 top-1/2 -translate-y-1/2 h-9 rounded-lg px-5 font-bold shadow-sm" disabled={isLoading}>
                        {i18n.t('search')}
                    </Button>
                </form>

                <div class="flex flex-wrap items-center gap-2 sm:gap-3 w-full 2xl:w-auto">

                    <Select.Root type="single" bind:value={contentType}>
                        <Select.Trigger class="w-[130px] sm:w-[140px] bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold">
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
                            <Select.Item value="anime">{i18n.t('anime')}</Select.Item>
                            <Select.Item value="manga">{i18n.t('manga')}</Select.Item>
                            <Select.Item value="novel">{i18n.t('novel')}</Select.Item>
                        </Select.Content>
                    </Select.Root>

                    <Select.Root type="single" bind:value={searchMode}>
                        <Select.Trigger class="w-[140px] sm:w-[160px] bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold">
                            {#if searchMode === "database"}
                                <Database class="w-4 h-4 mr-2 text-primary/70" /> {i18n.t('database')}
                            {:else}
                                <Plug class="w-4 h-4 mr-2 text-primary/70" /> {i18n.t('extension')}
                            {/if}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="database">{i18n.t('database_search')}</Select.Item>
                            <Select.Item value="extension" disabled={availableExtensions.length === 0}>
                                {i18n.t('extension_search')}
                            </Select.Item>
                        </Select.Content>
                    </Select.Root>

                    {#if searchMode === "extension" && availableExtensions.length > 0}
                        <Select.Root type="single" bind:value={selectedExtension}>
                            <Select.Trigger class="w-[140px] sm:w-[180px] bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold">
                                <!-- En tu objeto 'Extension' del store el id se asume como string único. Si en el futuro quieres mostrar extension.name en lugar de id, puedes buscarlo en el array -->
                                {selectedExtension || i18n.t('select_source')}
                            </Select.Trigger>
                            <Select.Content>
                                {#each availableExtensions as ext}
                                    <Select.Item value={ext}>{ext}</Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    {/if}

                    <div class="lg:hidden ml-auto">
                        <Drawer.Root bind:open={isDrawerOpen}>
                            <Drawer.Trigger>
                                <Button variant="secondary" class="h-11 border-border/50 rounded-xl font-bold shadow-sm">
                                    <SlidersHorizontal class="w-4 h-4 sm:mr-2" />
                                    <span class="hidden sm:inline">{i18n.t('filters')}</span>
                                </Button>
                            </Drawer.Trigger>
                            <Drawer.Content class="h-[85vh] rounded-t-2xl border-border/50">
                                <div class="p-6 overflow-y-auto hide-scrollbar">
                                    <h3 class="font-black text-2xl mb-6 tracking-tight flex items-center gap-2">
                                        <SlidersHorizontal class="w-5 h-5 text-primary" />
                                        {i18n.t('search_filters')}
                                    </h3>
                                    {@render filterFields()}
                                    <div class="mt-8 pt-6 border-t border-border/40">
                                        <Button class="w-full h-12 rounded-xl font-bold text-base shadow-sm" onclick={() => { performSearch(); isDrawerOpen = false; }}>
                                            {i18n.t('apply_search')}
                                        </Button>
                                    </div>
                                </div>
                            </Drawer.Content>
                        </Drawer.Root>
                    </div>

                </div>
            </div>

            <div class="w-full border-t border-border/40 pt-6">
                {#if isLoading}
                    <div class="flex flex-col items-center justify-center w-full min-h-[50vh] text-muted-foreground space-y-4">
                        <Loader2 class="w-10 h-10 animate-spin text-primary" />
                        <p class="text-sm font-bold animate-pulse">{i18n.t('searching_results')}</p>
                    </div>

                {:else if hasSearched && results.length === 0}
                    <Empty.Root class="border border-dashed py-24 rounded-2xl bg-muted/5 min-h-[50vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon"><SearchX class="w-12 h-12" /></Empty.Media>
                            <Empty.Title class="text-2xl">{i18n.t('no_results_found')}</Empty.Title>
                            <Empty.Description class="max-w-sm mx-auto text-base">
                                {i18n.t('no_matches_found')}
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