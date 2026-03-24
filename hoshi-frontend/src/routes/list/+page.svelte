<script lang="ts">
    import { listApi } from "$lib/api/list/list";
    import { auth } from "$lib/auth.svelte";
    import type { EnrichedListEntry, ListStatus, UserStats } from "$lib/api/list/types";
    import type { ContentWithMappings, ContentType } from "$lib/api/content/types";

    import ContentCard from "@/components/content/Card.svelte";
    import ListEditor from "@/components/modals/ListEditor.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Avatar from "$lib/components/ui/avatar";
    import { Input } from "$lib/components/ui/input";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import {
        Search, List, Filter, MoreVertical, CheckCircle2,
        PlayCircle, Clock, PauseCircle, XCircle, Monitor, Library
    } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";
    import { layoutState } from '@/layout.svelte.js';

    $effect(() => {
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    let activeStatus = $state<string>("ALL");
    let activeType = $state<string>("ALL");
    let searchQuery = $state("");
    let isLoading = $state(true);
    let entries = $state<EnrichedListEntry[]>([]);
    let stats = $state<UserStats | null>(null);

    let selectedEntry = $state<EnrichedListEntry | null>(null);
    let isModalOpen = $state(false);

    async function loadData() {
        isLoading = true;
        try {
            const query = {
                status: activeStatus === "ALL" ? undefined : activeStatus as ListStatus,
                contentType: activeType === "ALL" ? undefined : activeType
            };

            const [listRes, statsRes] = await Promise.all([
                listApi.getList(query),
                listApi.getStats()
            ]);

            entries = listRes.results;
            stats = statsRes;
        } catch (error) {
            console.error("Failed to load collection data:", error);
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        activeStatus;
        activeType;
        loadData();
    });

    let filteredEntries = $derived(
        entries.filter(e => e.title.toLowerCase().includes(searchQuery.toLowerCase()))
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

    function mapToContentWithMappings(entry: EnrichedListEntry): ContentWithMappings {
        return {
            content: { cid: entry.cid, contentType: entry.contentType as ContentType, nsfw: entry.nsfw, createdAt: 0, updatedAt: 0 },
            metadata: [{
                cid: entry.cid,
                sourceName: "list",
                title: entry.title,
                coverImage: entry.coverImage || undefined,
                subtype: entry.contentType === 'anime' ? 'TV' : 'MANGA',
                epsOrChapters: entry.totalUnits || null,
                characters: [], staff: [], externalIds: entry.externalIds as any || {},
                createdAt: 0, updatedAt: 0
            }],
            trackerMappings: [], extensionSources: [], relations: [], contentUnits: []
        };
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
                    <Library class="size-3.5 text-primary" /> {stats?.totalEntries === 1
                    ? i18n.t('list.single_entry', { count: stats?.totalEntries })
                    : i18n.t('list.multiple_entries', { count: stats?.totalEntries || 0 })}
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

    {#if stats}
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4" in:fade>
            {@render statCard('watching', stats.watching, PlayCircle, 'text-primary')}
            {@render statCard('completed', stats.completed, CheckCircle2, 'text-green-500')}
            {@render statCard('planning', stats.planning, Clock, 'text-blue-500')}
            {@render statCard('paused_status', stats.paused, PauseCircle, 'text-yellow-500')}
            {@render statCard('dropped_status', stats.dropped, XCircle, 'text-destructive')}
            {@render statCard('episodes', stats.totalEpisodes, Monitor, 'text-purple-500')}
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
        {#if isLoading}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 gap-4 md:gap-6">
                {#each Array(14) as _}
                    <Skeleton class="aspect-[2/3] w-full rounded-xl bg-muted/20" />
                {/each}
            </div>
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
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 gap-x-4 gap-y-10 md:gap-x-5 md:gap-y-12">
                {#each filteredEntries as entry (entry.cid)}
                    <div in:fade={{ duration: 300 }} class="group relative flex flex-col w-full h-full">
                        <div class="relative w-full h-full">
                            {#if entry.totalUnits && entry.progress > 0}
                                <div class="absolute bottom-1.5 left-1.5 right-1.5 z-20 h-1.5 bg-black/60 rounded-full overflow-hidden backdrop-blur-md border border-white/10 pointer-events-none">
                                    <div class="h-full bg-primary transition-all duration-700 ease-out" style="width: {Math.min((entry.progress / entry.totalUnits) * 100, 100)}%"></div>
                                </div>
                            {/if}
                            <ContentCard item={mapToContentWithMappings(entry)} disableHover={true} />
                            <div class="absolute top-2 left-2 right-2 z-20 flex justify-between items-start opacity-100 lg:opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                                <span class="bg-black/80 backdrop-blur-md px-2 py-1 rounded-md text-[10px] font-black uppercase tracking-widest text-primary border border-white/10 shadow-sm">
                                    {entry.progress} / {entry.totalUnits || '?'}
                                </span>
                                <Button variant="secondary" size="icon" class="h-7 w-7 rounded-md bg-black/80 text-white border border-white/10 hover:bg-primary hover:text-primary-foreground pointer-events-auto" onclick={(e) => { e.preventDefault(); openEdit(entry); }}>
                                    <MoreVertical class="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</main>

{#if selectedEntry}
    <ListEditor bind:open={isModalOpen} cid={selectedEntry.cid} title={selectedEntry.title} contentType={selectedEntry.contentType} coverImage={selectedEntry.coverImage ?? undefined} />
{/if}