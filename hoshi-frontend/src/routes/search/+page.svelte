<script lang="ts">
    import { untrack } from "svelte";
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import type { CoreMetadata, ContentType } from "$lib/api/content/types";

    import ContentCard from "$lib/components/home/ContentCard.svelte";

    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import ContentCardSkeleton from "$lib/components/home/ContentCardSkeleton.svelte";
    import { Search, SearchX, Database, Plug, SlidersHorizontal, Tv, Book, BookOpen } from "lucide-svelte";

    // --- State Variables ---
    let searchQuery = $state("");
    let contentType = $state<ContentType>("anime");
    let searchMode = $state<"database" | "extension">("database");

    let availableExtensions = $state<string[]>([]);
    let selectedExtension = $state<string>("");

    // Filters State
    let dbStatus = $state<string>("");
    let dbGenre = $state<string>("");
    let dbFormat = $state<string>("");
    let dbNsfw = $state<boolean>(false);

    let extFiltersSchema = $state<Record<string, any>>({});
    let extFilterValues = $state<Record<string, any>>({});

    let results = $state<CoreMetadata[]>([]);
    let isLoading = $state(true);
    let hasSearched = $state(false);

    let isDrawerOpen = $state(false);

    const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
    const formatLabel = (key: string) => key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());

    $effect(() => {
        const currentType = contentType;

        untrack(async () => {
            try {
                let res;
                if (currentType === "anime") res = await extensionsApi.getAnime();
                else if (currentType === "manga") res = await extensionsApi.getManga();
                else if (currentType === "novel") res = await extensionsApi.getNovel();

                availableExtensions = res?.extensions || [];

                if (availableExtensions.length > 0 && !availableExtensions.includes(selectedExtension)) {
                    selectedExtension = availableExtensions[0];
                } else if (availableExtensions.length === 0) {
                    selectedExtension = "";
                    searchMode = "database";
                }
            } catch (error) {
                console.error("Failed to load extensions:", error);
                availableExtensions = [];
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

    // --- Search Logic ---
    const performSearch = async () => {
        isLoading = true;
        hasSearched = true;
        results = [];

        try {
            if (searchMode === "database") {
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
                results = res.data ? (res.data as unknown as CoreMetadata[]) : [];

            } else if (searchMode === "extension" && selectedExtension) {
                const res = await contentApi.searchExtension(selectedExtension, {
                    query: searchQuery,
                    extensionFilters: Object.keys(extFilterValues).length > 0 ? JSON.stringify(extFilterValues) : undefined
                });
                const rawResults = Array.isArray(res.results) ? res.results : [];
                results = rawResults.map((item: any) => ({
                    ...item,
                    contentType: contentType
                })) as CoreMetadata[];
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

        // Reset dynamic values safely
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
    <div class="space-y-6">
        {#if searchMode === "database"}
            <div class="space-y-4">
                <div class="space-y-2">
                    <Label>Status</Label>
                    <Select.Root type="single" bind:value={dbStatus}>
                        <Select.Trigger>{dbStatus || "Any Status"}</Select.Trigger>
                        <Select.Content>
                            <Select.Item value="">Any Status</Select.Item>
                            <Select.Item value="Completed">Completed</Select.Item>
                            <Select.Item value="Ongoing">Ongoing</Select.Item>
                            <Select.Item value="Planned">Planned</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="space-y-2">
                    <Label>Genre</Label>
                    <Select.Root type="single" bind:value={dbGenre}>
                        <Select.Trigger>{dbGenre || "Any Genre"}</Select.Trigger>
                        <Select.Content>
                            <Select.Item value="">Any Genre</Select.Item>
                            <Select.Item value="Action">Action</Select.Item>
                            <Select.Item value="Romance">Romance</Select.Item>
                            <Select.Item value="Fantasy">Fantasy</Select.Item>
                            <Select.Item value="Sci-Fi">Sci-Fi</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="space-y-2">
                    <Label>Format</Label>
                    <Select.Root type="single" bind:value={dbFormat}>
                        <Select.Trigger>{dbFormat || "Any Format"}</Select.Trigger>
                        <Select.Content>
                            <Select.Item value="">Any Format</Select.Item>
                            <Select.Item value="TV">TV</Select.Item>
                            <Select.Item value="MOVIE">Movie</Select.Item>
                            <Select.Item value="OVA">OVA</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
                <div class="flex items-center space-x-2 pt-2">
                    <Switch id="nsfw-mode" bind:checked={dbNsfw} />
                    <Label for="nsfw-mode">NSFW Only (18+)</Label>
                </div>
            </div>

        {:else if searchMode === "extension" && Object.keys(extFiltersSchema).length > 0}
            <div class="space-y-4">
                {#each Object.entries(extFiltersSchema) as [key, filterDef]}
                    <div class="space-y-2">
                        <Label>{filterDef.label || formatLabel(key)}</Label>

                        {#if filterDef.type === 'select'}
                            <Select.Root type="single" bind:value={extFilterValues[key]}>
                                <Select.Trigger>
                                    {filterDef.options.find((o) => o.value === extFilterValues[key])?.label || "Select..."}
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
                                            class="px-3 py-1.5 text-xs rounded-md border transition-colors
                                        {extFilterValues[key]?.includes(option.value)
                                            ? 'bg-primary text-primary-foreground border-primary'
                                            : 'bg-background hover:bg-muted'}"
                                            onclick={() => toggleMultiSelect(key, option.value)}
                                    >
                                        {option.label}
                                    </button>
                                {/each}
                            </div>

                        {:else if filterDef.type === 'boolean'}
                            <div class="flex items-center space-x-2 pt-2">
                                <Switch id={`filter-${key}`} bind:checked={extFilterValues[key]} />
                                <Label for={`filter-${key}`}>{filterDef.label || formatLabel(key)}</Label>
                            </div>

                        {:else}
                            <Input
                                    type="text"
                                    placeholder={`Enter ${filterDef.label?.toLowerCase() || formatLabel(key).toLowerCase()}...`}
                                    bind:value={extFilterValues[key]}
                            />
                        {/if}
                    </div>
                {/each}
            </div>

        {:else}
            <p class="text-muted-foreground text-sm">No specific filters available for this source.</p>
        {/if}

        <div class="pt-4 border-t">
            <Button type="button" variant="secondary" class="w-full" onclick={clearFilters}>
                Clear Filters
            </Button>
        </div>
    </div>
{/snippet}


<svelte:head>
    <title>Search</title>
</svelte:head>

<main class="min-h-screen bg-background pb-20 pt-24 px-4 md:px-8 max-w-[1600px] mx-auto">

    <div class="flex flex-col lg:flex-row gap-8">

        <aside class="hidden lg:block w-72 shrink-0">
            <div class="sticky top-24 p-6 bg-card border rounded-xl shadow-sm">
                <h3 class="font-semibold text-lg border-b pb-4 mb-4">Filters</h3>
                {@render filterFields()}
            </div>
        </aside>

        <div class="flex-1 space-y-8">
            <div class="flex items-center justify-between">
                <h1 class="text-3xl md:text-4xl font-bold tracking-tight">Discover</h1>

                <div class="lg:hidden">
                    <Drawer.Root bind:open={isDrawerOpen}>
                        <Drawer.Trigger>
                            <Button variant="outline" size="sm">
                                <SlidersHorizontal class="w-4 h-4 mr-2" />
                                Filters
                            </Button>
                        </Drawer.Trigger>
                        <Drawer.Content class="h-[85vh]">
                            <div class="p-6 overflow-y-auto">
                                <h3 class="font-semibold text-xl mb-6">Search Filters</h3>
                                {@render filterFields()}
                                <div class="mt-6 pt-6 border-t">
                                    <Button class="w-full" onclick={() => { performSearch(); isDrawerOpen = false; }}>
                                        Apply & Search
                                    </Button>
                                </div>
                            </div>
                        </Drawer.Content>
                    </Drawer.Root>
                </div>
            </div>

            <form onsubmit={(e) => { e.preventDefault(); performSearch(); }} class="space-y-6">

                <div class="relative flex items-center w-full">
                    <Search class="absolute left-4 w-5 h-5 text-muted-foreground" />
                    <Input
                            type="text"
                            placeholder={`Search for ${contentType} titles...`}
                            class="pl-12 pr-28 h-14 text-lg rounded-full shadow-sm bg-card/50 focus-visible:ring-primary"
                            bind:value={searchQuery}
                    />
                    <Button type="submit" class="absolute right-2 rounded-full px-6" disabled={isLoading}>
                        Search
                    </Button>
                </div>

                <div class="flex flex-wrap items-center gap-4 p-4 rounded-xl bg-muted/20 border">

                    <div class="flex items-center gap-2">
                        <Label class="text-muted-foreground hidden sm:block">Mode:</Label>
                        <Select.Root type="single" bind:value={contentType}>
                            <Select.Trigger class="w-[140px] bg-background">
                                {#if contentType === "anime"}
                                    <Tv class="w-4 h-4 mr-2 inline-block text-primary" />
                                {:else if contentType === "manga"}
                                    <Book class="w-4 h-4 mr-2 inline-block text-primary" />
                                {:else}
                                    <BookOpen class="w-4 h-4 mr-2 inline-block text-primary" />
                                {/if}
                                {capitalize(contentType)}
                            </Select.Trigger>
                            <Select.Content>
                                <Select.Item value="anime">Anime</Select.Item>
                                <Select.Item value="manga">Manga</Select.Item>
                                <Select.Item value="novel">Novel</Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>

                    <div class="flex items-center gap-2">
                        <Label class="text-muted-foreground hidden sm:block">Source:</Label>
                        <Select.Root type="single" bind:value={searchMode}>
                            <Select.Trigger class="w-[160px] bg-background">
                                {#if searchMode === "database"}
                                    <Database class="w-4 h-4 mr-2 inline-block" /> Database
                                {:else}
                                    <Plug class="w-4 h-4 mr-2 inline-block" /> Extension
                                {/if}
                            </Select.Trigger>
                            <Select.Content>
                                <Select.Item value="database">Database Search</Select.Item>
                                <Select.Item value="extension" disabled={availableExtensions.length === 0}>
                                    Extension Search
                                </Select.Item>
                            </Select.Content>
                        </Select.Root>
                    </div>

                    {#if searchMode === "extension" && availableExtensions.length > 0}
                        <div class="flex items-center gap-2">
                            <Label class="text-muted-foreground hidden sm:block">Provider:</Label>
                            <Select.Root type="single" bind:value={selectedExtension}>
                                <Select.Trigger class="w-[180px] bg-background">
                                    {selectedExtension || "Select Source"}
                                </Select.Trigger>
                                <Select.Content>
                                    {#each availableExtensions as ext}
                                        <Select.Item value={ext}>{ext}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        </div>
                    {/if}
                </div>
            </form>

            <hr class="border-border/40" />

            <div class="w-full">
                {#if isLoading}
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-5 2xl:grid-cols-6 gap-4 md:gap-6">
                        {#each Array(18) as _}
                            <ContentCardSkeleton />
                        {/each}
                    </div>

                {:else if hasSearched && results.length === 0}
                    <div class="mt-12">
                        <Empty.Root class="border border-dashed py-20 bg-muted/10">
                            <Empty.Header>
                                <Empty.Media variant="icon"><SearchX /></Empty.Media>
                                <Empty.Title>No results found</Empty.Title>
                                <Empty.Description class="max-w-md mx-auto">
                                    We couldn't find any matches in the selected source. Try using different keywords or changing your filters.
                                </Empty.Description>
                            </Empty.Header>
                        </Empty.Root>
                    </div>

                {:else if results.length > 0}
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-5 2xl:grid-cols-6 gap-4 md:gap-6">
                        {#each results as item (item.cid || "")}
                            <ContentCard {item} />
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </div>
</main>