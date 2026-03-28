<script lang="ts">
    import { auth } from "$lib/auth.svelte";
    import { listStore } from "@/list.svelte.js";
    import type { EnrichedListEntry } from "$lib/api/list/types";
    import type { ContentWithMappings, ContentType } from "$lib/api/content/types";
    import ContentCard from "@/components/content/Card.svelte";
    import ListEditor from "@/components/modals/ListEditor.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Pagination from "$lib/components/ui/pagination";
    import { Input } from "$lib/components/ui/input";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import {
        Search, List, Filter, MoreVertical, CheckCircle2,
        PlayCircle, Clock, PauseCircle, XCircle, Monitor, Library, AlertCircle
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";
    import { layoutState } from '@/layout.svelte.js';
    import { appConfig } from "@/config.svelte.js";

    $effect(() => {
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
        listStore.loadData();
    });

    let activeStatus = $state<string>("ALL");
    let activeType = $state<string>("ALL");
    let searchQuery = $state("");

    let selectedEntry = $state<EnrichedListEntry | null>(null);
    let isModalOpen = $state(false);
    let currentTitleLanguage = $derived(appConfig.data?.ui?.titleLanguage || 'romaji');

    let currentPage = $state(1);
    const itemsPerPage = 49;

    $effect(() => {
        activeStatus;
        activeType;
        searchQuery;
        currentPage = 1;
    });

    function getDisplayTitle(entry: EnrichedListEntry): string {
        const i18nTitles = (entry as any).titleI18n;
        if (i18nTitles && i18nTitles[currentTitleLanguage]) {
            return i18nTitles[currentTitleLanguage];
        }
        return entry.title || "";
    }

    function mapToContentWithMappings(entry: EnrichedListEntry): ContentWithMappings {
        return {
            _isPartialMock: true,
            content: { cid: entry.cid, contentType: entry.contentType as ContentType, nsfw: entry.nsfw, createdAt: 0, updatedAt: 0 },
            metadata: [{
                cid: entry.cid,
                sourceName: "list",
                title: entry.title,
                titleI18n: (entry as any).titleI18n,
                coverImage: entry.coverImage || undefined,
                subtype: entry.contentType === 'anime' ? 'TV' : 'MANGA',
                epsOrChapters: entry.totalUnits || null,
                characters: [], staff: [], externalIds: entry.externalIds as any || {},
                createdAt: 0, updatedAt: 0
            }],
            trackerMappings: [], extensionSources: [], relations: [], contentUnits: []
        };
    }

    let mappedEntries = $derived(
        listStore.entries.map(entry => {
            const displayTitle = getDisplayTitle(entry).toLowerCase();
            const baseTitle = (entry.title || "").toLowerCase();
            return {
                original: entry,
                searchString: displayTitle + " " + baseTitle,
                mappedContent: mapToContentWithMappings(entry)
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

    let paginatedEntries = $derived(
        filteredEntries.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)
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

{#snippet statCard(label, value, icon, colorClass)}
    <div class="bg-card/40 backdrop-blur-sm border border-border/40 p-4 rounded-2xl shadow-sm space-y-2">
        <div class="flex items-center justify-between">
            <span class="text-[10px] font-black uppercase tracking-widest text-muted-foreground">{i18n.t(label)}</span>
            <svelte:component this={icon} class="size-4 {colorClass}" />
        </div>
        <p class="text-2xl font-black tracking-tight">{value}</p>
    </div>
{/snippet}

<svelte:head>
    <title>{i18n.t('list.title')}</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-12 pt-8 md:pt-12 px-4 md:px-8 lg:px-12 w-full max-w-[2000px] mx-auto space-y-10">

    <header class="flex flex-col md:flex-row md:items-center justify-between gap-6 border-b border-border/40 pb-8 w-full">
        <div class="flex items-center gap-5">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>

            <div class="space-y-0.5">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight">
                    {i18n.t('list.header_title', { name: auth.user?.username || i18n.t('list.default_user')})}
                </h1>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-70 uppercase tracking-wider flex items-center gap-2">
                    <Library class="size-3.5 text-primary" /> {listStore.stats?.totalEntries === 1
                    ? i18n.t('list.single_entry', { count: listStore.stats?.totalEntries })
                    : i18n.t('list.multiple_entries', { count: listStore.stats?.totalEntries || 0 })}
                </p>
            </div>
        </div>

        <div class="relative w-full md:w-80 group">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <Input
                    placeholder={i18n.t('list.search_placeholder')}
                    class="pl-11 bg-muted/10 border-none shadow-sm h-11 rounded-xl focus-visible:ring-2 focus-visible:ring-primary/40 transition-all text-sm font-medium"
                    bind:value={searchQuery}
            />
        </div>
    </header>

    {#if listStore.stats}
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4" in:fade>
            {@render statCard('watching', listStore.stats.watching, PlayCircle, 'text-primary')}
            {@render statCard('completed', listStore.stats.completed, CheckCircle2, 'text-green-500')}
            {@render statCard('planning', listStore.stats.planning, Clock, 'text-blue-500')}
            {@render statCard('paused_status', listStore.stats.paused, PauseCircle, 'text-yellow-500')}
            {@render statCard('dropped_status', listStore.stats.dropped, XCircle, 'text-destructive')}
            {@render statCard('episodes', listStore.stats.totalEpisodes, Monitor, 'text-purple-500')}
        </div>
    {/if}

    <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4 w-full pt-4">
        <div class="min-w-0 w-full lg:w-auto overflow-hidden">
            <Tabs.Root bind:value={activeStatus} class="w-full">
                <Tabs.List class="bg-transparent h-auto p-0 flex justify-start overflow-x-auto flex-nowrap hide-scrollbar gap-2 w-full border-b border-transparent">
                    {#each statusOptions as opt}
                        <Tabs.Trigger
                                value={opt.value}
                                class="relative px-4 py-2.5 rounded-full text-xs sm:text-sm font-bold transition-all
                                   data-[state=active]:bg-primary/10 data-[state=active]:text-primary
                                   data-[state=active]:border-primary border border-border/40
                                   data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/30 whitespace-nowrap shrink-0 shadow-sm"
                        >
                            <opt.icon class="h-3.5 w-3.5 sm:h-4 sm:w-4 mr-2 inline-block" />
                            {opt.label}
                        </Tabs.Trigger>
                    {/each}
                </Tabs.List>
            </Tabs.Root>
        </div>

        <div class="flex items-center gap-3 w-full sm:w-auto shrink-0 lg:justify-end">
            <Select.Root type="single" bind:value={activeType}>
                <Select.Trigger class="w-full sm:w-[140px] bg-card border border-border/40 shadow-sm h-10 rounded-full text-sm font-bold hover:bg-muted/50 transition-colors">
                    <Filter class="h-4 w-4 mr-2 opacity-60 text-primary" />
                    {activeType === "ALL" ? (i18n.t('list.all_content')) : i18n.t(activeType)}
                </Select.Trigger>
                <Select.Content>
                    <Select.Item value="ALL" class="font-bold">{i18n.t('list.all_content')}</Select.Item>
                    <Select.Item value="anime" class="font-bold">{i18n.t('list.anime')}</Select.Item>
                    <Select.Item value="manga" class="font-bold">{i18n.t('list.manga')}</Select.Item>
                    <Select.Item value="novel" class="font-bold">{i18n.t('list.novel')}</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
    </div>

    <section>
        {#if listStore.isLoading}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 gap-4 md:gap-6">
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
                    <Empty.Title class="text-xl font-bold text-destructive">
                        {i18n.t(listStore.error.key || 'Error')}
                    </Empty.Title>
                    <Button variant="outline" class="mt-6 border-destructive/20 hover:bg-destructive/10 text-destructive" onclick={() => listStore.refresh()}>
                        {i18n.t("content.retry")}</Button>
                </Empty.Header>
            </Empty.Root>
        {:else if filteredEntries.length === 0}
            <Empty.Root class="border border-dashed border-border/40 bg-muted/5 rounded-2xl py-24 min-h-[40vh] flex items-center justify-center">
                <Empty.Header>
                    <Empty.Media variant="icon" class="bg-primary/10 text-primary mb-4 p-4 rounded-full">
                        <List class="size-8" />
                    </Empty.Media>
                    <Empty.Title class="text-xl font-bold">{i18n.t('list.empty_title')}</Empty.Title>
                    <Empty.Description class="text-muted-foreground font-medium">{i18n.t('list.empty_desc')}</Empty.Description>
                </Empty.Header>
            </Empty.Root>
        {:else}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 gap-x-4 gap-y-10 md:gap-x-5 md:gap-y-12 mb-10">
                {#each paginatedEntries as item (item.original.cid)}
                    <div in:fade={{ duration: 300 }} class="group relative flex flex-col w-full h-full">
                        <div class="relative w-full h-full">

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
                    </div>
                {/each}
            </div>

            {#if filteredEntries.length > itemsPerPage}
                <div class="flex justify-center w-full mt-10">
                    <Pagination.Root count={filteredEntries.length} perPage={itemsPerPage} bind:page={currentPage}>
                        {#snippet children({ pages, currentPage })}
                            <Pagination.Content>
                                <Pagination.Item>
                                    <Pagination.PrevButton />
                                </Pagination.Item>
                                {#each pages as page (page.key)}
                                    {#if page.type === "ellipsis"}
                                        <Pagination.Item>
                                            <Pagination.Ellipsis />
                                        </Pagination.Item>
                                    {:else}
                                        <Pagination.Item>
                                            <Pagination.Link {page} isActive={currentPage === page.value}>
                                                {page.value}
                                            </Pagination.Link>
                                        </Pagination.Item>
                                    {/if}
                                {/each}
                                <Pagination.Item>
                                    <Pagination.NextButton />
                                </Pagination.Item>
                            </Pagination.Content>
                        {/snippet}
                    </Pagination.Root>
                </div>
            {/if}
        {/if}
    </section>
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