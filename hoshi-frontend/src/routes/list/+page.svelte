<script lang="ts">
    import { auth } from "@/stores/auth.svelte.js";
    import { listStore } from "@/stores/list.svelte.js";
    import type { EnrichedListEntry } from "$lib/api/list/types";
    import ContentCard from "@/components/content/Card.svelte";
    import ListEditor from "@/components/modals/ListEditor.svelte";
    import * as Empty from "$lib/components/ui/empty";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Pagination from "$lib/components/ui/pagination";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import {
        Search, List, MoreVertical, CheckCircle2,
        PlayCircle, Clock, PauseCircle, XCircle, Monitor, Library, AlertCircle, SlidersHorizontal, X
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { layoutState } from '@/stores/layout.svelte.js';
    import { appConfig } from "@/stores/config.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let activeStatus = $state<string>("ALL");
    let activeType = $state<string>("ALL");
    let searchQuery = $state("");
    let activeSort = $state<string>("TITLE_ASC");

    let isMobileSearchActive = $state(false);
    let isDrawerOpen = $state(false);

    let selectedEntry = $state<EnrichedListEntry | null>(null);
    let isModalOpen = $state(false);
    let currentTitleLanguage = $derived(appConfig.data?.ui?.titleLanguage || 'romaji');
    let currentPage = $state(1);
    const itemsPerPage = 49;

    $effect(() => {
        layoutState.title = isMobileSearchActive ? "" : i18n.t('list.title');
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileTopbar;
        listStore.loadData();
    });

    $effect(() => {
        activeStatus;
        activeType;
        searchQuery;
        activeSort;
        currentPage = 1;
    });

    function getDisplayTitle(entry: EnrichedListEntry): string {
        const i18nTitles = (entry as any).titleI18n;
        if (i18nTitles && i18nTitles[currentTitleLanguage]) {
            return i18nTitles[currentTitleLanguage];
        }
        return entry.title || "";
    }

    function resetFilters() {
        activeStatus = "ALL";
        activeType = "ALL";
        searchQuery = "";
        activeSort = "TITLE_ASC";
    }

    let mappedEntries = $derived(
        listStore.entries.map(entry => {
            const displayTitle = getDisplayTitle(entry).toLowerCase();
            const baseTitle = (entry.title || "").toLowerCase();
            return {
                original: entry,
                searchString: displayTitle + " " + baseTitle,
                mappedContent: entry
            };
        })
    );

    let filteredEntries = $derived(
        mappedEntries.filter(item => {
            const matchesStatus = activeStatus === "ALL" || item.original.status === activeStatus;
            const matchesType = activeType === "ALL" || item.original.contentType === activeType;
            const matchesSearch = searchQuery === "" || item.searchString.includes(searchQuery.toLowerCase());
            return matchesStatus && matchesType && matchesSearch;
        })
    );

    let sortedEntries = $derived(
        [...filteredEntries].sort((a, b) => {
            const titleA = a.original.title || "";
            const titleB = b.original.title || "";
            switch (activeSort) {
                case "TITLE_ASC": return titleA.localeCompare(titleB);
                case "TITLE_DESC": return titleB.localeCompare(titleA);
                case "PROGRESS_DESC": return (b.original.progress || 0) - (a.original.progress || 0);
                case "PROGRESS_ASC": return (a.original.progress || 0) - (b.original.progress || 0);
                default: return 0;
            }
        })
    );

    let paginatedEntries = $derived(
        sortedEntries.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)
    );

    const statusOptions = $derived([
        { value: "ALL", label: i18n.t('list.all'), icon: List },
        { value: "CURRENT", label: i18n.t('list.current'), icon: PlayCircle },
        { value: "COMPLETED", label: i18n.t('list.completed'), icon: CheckCircle2 },
        { value: "PLANNING", label: i18n.t('list.planning'), icon: Clock },
        { value: "PAUSED", label: i18n.t('list.paused'), icon: PauseCircle },
        { value: "DROPPED", label: i18n.t('list.dropped'), icon: XCircle }
    ]);

    function openEdit(entry: EnrichedListEntry) {
        selectedEntry = entry;
        isModalOpen = true;
    }
</script>

{#snippet statusSelect()}
    <ResponsiveSelect
            bind:value={activeStatus}
            items={statusOptions}
            class="h-11 rounded-xl font-bold bg-card border border-border/40 shadow-sm"
    />
{/snippet}

{#snippet sortSelect()}
    <ResponsiveSelect
            bind:value={activeSort}
            items={[
            { value: "TITLE_ASC", label: "A-Z" },
            { value: "TITLE_DESC", label: "Z-A" },
            { value: "PROGRESS_DESC", label: i18n.t('list.sort_progress_desc')},
            { value: "PROGRESS_ASC", label: i18n.t('list.sort_progress_asc') },
        ]}
            class="h-11 rounded-xl font-bold bg-card border border-border/40 shadow-sm"
    />
{/snippet}

{#snippet searchBar()}
    <div class="relative w-full group">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
        <Input
                placeholder={i18n.t('list.search_placeholder')}
                class="pl-9 pr-3 h-9 text-sm rounded-full border-none bg-muted/30 focus-visible:ring-1 focus-visible:ring-primary/50 w-full shadow-inner"
                bind:value={searchQuery}
        />
    </div>
{/snippet}

{#snippet typeSelect()}
    <ResponsiveSelect
            bind:value={activeType}
            items={[
            { value: "ALL", label: i18n.t('list.all_content') },
            { value: "anime", label: i18n.t('list.anime') },
            { value: "manga", label: i18n.t('list.manga') },
            { value: "novel", label: i18n.t('list.novel') }
        ]}
            class="h-11 rounded-xl font-bold bg-card border border-border/40 shadow-sm"
    />
{/snippet}

{#snippet mobileTopbar()}
    {#if isMobileSearchActive}
        <div class="flex items-center gap-1 w-full pl-2" in:fade={{ duration: 150 }}>
            <div class="flex-1 min-w-0">{@render searchBar()}</div>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full shrink-0" onclick={() => isMobileSearchActive = false}>
                <X class="w-[22px] h-[22px]" />
            </Button>
        </div>
    {:else}
        <div class="flex items-center gap-0.5 w-full justify-end" in:fade={{ duration: 150 }}>
            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50" onclick={() => isMobileSearchActive = true}>
                <Search class="w-[22px] h-[22px]" />
            </Button>

            <Drawer.Root bind:open={isDrawerOpen}>
                <Drawer.Trigger>
                    <Button variant="ghost" size="icon" class="h-10 w-10 rounded-full hover:bg-muted/50 relative">
                        <SlidersHorizontal class="w-[22px] h-[22px]" />
                        {#if activeStatus !== 'ALL' || activeType !== 'ALL' || activeSort !== 'TITLE_ASC'}
                            <span class="absolute top-2 right-2 w-2 h-2 bg-primary rounded-full border border-background"></span>
                        {/if}
                    </Button>
                </Drawer.Trigger>

                <Drawer.Content class="max-h-[85vh] rounded-t-3xl border-border/50">
                    <div class="w-full h-full flex flex-col overflow-hidden">
                        <div class="flex items-center justify-between p-6 pb-2">
                            <h3 class="font-black text-2xl tracking-tight">{i18n.t("search.filters")}</h3>
                            <Button variant="ghost" size="sm" class="text-xs font-bold text-muted-foreground hover:text-primary" onclick={resetFilters}>
                                {i18n.t("list.clear_all")}
                            </Button>
                        </div>

                        <div class="flex-1 p-6 pt-2 overflow-y-auto hide-scrollbar space-y-6">
                            <div class="space-y-2.5">
                                <Label class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground ml-1">{i18n.t("list.sort_by")}</Label>
                                {@render sortSelect()}
                            </div>
                            <div class="space-y-2.5">
                                <Label class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground ml-1">{i18n.t("list.content_type")}</Label>
                                {@render typeSelect()}
                            </div>
                            <div class="space-y-2.5">
                                <Label class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground ml-1">{i18n.t("list.status")}</Label>
                                {@render statusSelect()}
                            </div>
                        </div>

                        <div class="shrink-0 p-4 bg-background border-t border-border/40 pb-8">
                            <Button class="w-full h-12 rounded-xl font-bold text-base shadow-sm" onclick={() => isDrawerOpen = false}>
                                {i18n.t("search.apply_search")}
                            </Button>
                        </div>
                    </div>
                </Drawer.Content>
            </Drawer.Root>
        </div>
    {/if}
{/snippet}

<svelte:head>
    <title>{i18n.t('list.title')}</title>
</svelte:head>

<main class="bg-background px-4 md:px-8 lg:pl-32 lg:pr-12 lg:pt-20 w-full max-w-[2000px] mx-auto space-y-10 pt-5">
    <header class="hidden lg:flex lg:flex-row lg:items-start justify-between gap-6 border-b border-border/40 pb-8 w-full">
        <div class="flex items-start gap-5 w-full">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm shrink-0">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>
            <div class="flex flex-col gap-2 w-full">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight leading-none">
                    {i18n.t('list.header_title', { name: auth.user?.username || i18n.t('list.default_user')})}
                </h1>
                {#if listStore.stats}
                    <div class="flex flex-wrap items-center gap-2 mt-1">
                        <div class="flex items-center gap-1.5 bg-primary/10 text-primary px-2.5 py-1 rounded-md border border-primary/20">
                            <Library class="size-3.5" />
                            <span class="text-xs font-bold">{listStore.stats.totalEntries}</span>
                            <span class="text-[10px] uppercase font-bold tracking-wider opacity-80">Entries</span>
                        </div>
                        {#snippet statBadge(value, icon, colorClass)}
                            {#if value > 0}
                                <div class="flex items-center gap-1.5 bg-muted/40 px-2 py-1 rounded-md border border-border/40 text-muted-foreground">
                                    <svelte:component this={icon} class="size-3.5 {colorClass}" />
                                    <span class="text-xs font-bold text-foreground">{value}</span>
                                </div>
                            {/if}
                        {/snippet}
                        {@render statBadge(listStore.stats.watching, PlayCircle, 'text-primary')}
                        {@render statBadge(listStore.stats.completed, CheckCircle2, 'text-green-500')}
                        {@render statBadge(listStore.stats.planning, Clock, 'text-blue-500')}
                        {@render statBadge(listStore.stats.paused, PauseCircle, 'text-yellow-500')}
                        {@render statBadge(listStore.stats.dropped, XCircle, 'text-destructive')}
                        {@render statBadge(listStore.stats.totalEpisodes, Monitor, 'text-purple-500')}
                    </div>
                {/if}
            </div>
        </div>
    </header>

    <div class="flex items-start gap-8 w-full pt-4">
        <aside class="hidden lg:flex flex-col gap-8 w-64 shrink-0 sticky top-16 h-fit">
            <div class="space-y-3">{@render searchBar()}</div>
            <div class="space-y-6">
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.sort_by")}</h3>
                    {@render sortSelect()}
                </div>
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.content_type")}</h3>
                    {@render typeSelect()}
                </div>
                <div class="space-y-2.5">
                    <h3 class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-1">{i18n.t("list.status")}</h3>
                    {@render statusSelect()}
                </div>
            </div>
        </aside>

        <section class="flex-1 min-w-0">
            {#if listStore.isLoading}
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-7 gap-4 md:gap-6">
                    {#each Array(14) as _}
                        <Skeleton class="aspect-[2/3] w-full rounded-xl bg-muted/20" />
                    {/each}
                </div>
            {:else if listStore.error}
                <Empty.Root class="border border-dashed border-destructive/40 bg-destructive/5 rounded-2xl py-24 min-h-[40vh] flex flex-col items-center justify-center text-center px-4">
                    <Empty.Header>
                        <Empty.Media variant="icon" class="bg-destructive/10 text-destructive mb-4 p-4 rounded-full">
                            <AlertCircle class="size-8" />
                        </Empty.Media>
                        <Empty.Title class="text-xl font-bold text-destructive">{i18n.t(listStore.error.key)}</Empty.Title>
                        <Button variant="outline" class="mt-6 border-destructive/20 hover:bg-destructive/10 text-destructive" onclick={() => listStore.refresh()}>{i18n.t("content.retry")}</Button>
                    </Empty.Header>
                </Empty.Root>
            {:else if filteredEntries.length === 0}
                <Empty.Root class="border border-dashed border-border/40 bg-muted/5 rounded-2xl py-24 min-h-[40vh] flex items-center justify-center">
                    <Empty.Header>
                        <Empty.Media variant="icon" class="bg-primary/10 text-primary mb-4 p-4 rounded-full"><List class="size-8" /></Empty.Media>
                        <Empty.Title class="text-xl font-bold">{i18n.t('list.empty_title')}</Empty.Title>
                        <Empty.Description class="text-muted-foreground font-medium">{i18n.t('list.empty_desc')}</Empty.Description>
                    </Empty.Header>
                </Empty.Root>
            {:else}
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-7 gap-x-4 gap-y-10 md:gap-x-5 md:gap-y-12 mb-10">
                    {#each paginatedEntries as item (item.original.cid)}
                        <div in:fade={{ duration: 300 }} class="group relative flex flex-col w-full h-full">
                            <ContentCard item={item.mappedContent} disableHover={true} />
                            <div class="absolute top-2 left-2 right-2 z-20 flex justify-between items-start opacity-100 lg:opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                                <span class="bg-black/80 backdrop-blur-md px-2 py-1 rounded-md text-[10px] font-black uppercase tracking-widest text-primary border border-white/10 shadow-sm">
                                    {item.original.progress} / {item.original.totalUnits || '?'}
                                </span>
                                <Button variant="secondary" size="icon" class="h-7 w-7 rounded-md bg-black/80 text-white border border-white/10 hover:bg-primary hover:text-primary-foreground pointer-events-auto" onclick={(e) => { e.preventDefault(); openEdit(item.original); }}>
                                    <MoreVertical class="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                    {/each}
                </div>

                {#if filteredEntries.length > itemsPerPage}
                    <div class="flex justify-center w-full mt-10">
                        <Pagination.Root count={filteredEntries.length} perPage={itemsPerPage} bind:page={currentPage}>
                            {#snippet children({ pages, currentPage })}
                                <Pagination.Content>
                                    <Pagination.Item><Pagination.PrevButton /></Pagination.Item>
                                    {#each pages as page (page.key)}
                                        {#if page.type === "ellipsis"}
                                            <Pagination.Item><Pagination.Ellipsis /></Pagination.Item>
                                        {:else}
                                            <Pagination.Item>
                                                <Pagination.Link {page} isActive={currentPage === page.value}>{page.value}</Pagination.Link>
                                            </Pagination.Item>
                                        {/if}
                                    {/each}
                                    <Pagination.Item><Pagination.NextButton /></Pagination.Item>
                                </Pagination.Content>
                            {/snippet}
                        </Pagination.Root>
                    </div>
                {/if}
            {/if}
        </section>
    </div>
</main>

{#if selectedEntry}
    <ListEditor
            bind:open={isModalOpen}
            cid={selectedEntry.cid}
            title={getDisplayTitle(selectedEntry)}
            contentType={selectedEntry.contentType}
            coverImage={selectedEntry.coverImage ?? undefined}
    />
{/if}