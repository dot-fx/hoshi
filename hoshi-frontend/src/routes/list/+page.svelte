<script lang="ts">
    import { listApi } from "$lib/api/list/list";
    import type { EnrichedListEntry, ListStatus } from "$lib/api/list/types";
    import type { CoreMetadata, ContentType } from "$lib/api/content/types";
    import ContentCard from "$lib/components/home/ContentCard.svelte";
    import ListEditorModal from "$lib/components/ListEditorModal.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Select from "$lib/components/ui/select";
    import { Input } from "$lib/components/ui/input";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import { Search, Library, Filter, MoreVertical, CheckCircle2, PlayCircle, Clock, PauseCircle, XCircle } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte"; // <-- Importamos i18n

    let activeStatus = $state<string>("ALL");
    let activeType = $state<string>("ALL");
    let searchQuery = $state("");
    let isLoading = $state(true);
    let entries = $state<EnrichedListEntry[]>([]);
    let selectedEntry = $state<EnrichedListEntry | null>(null);
    let isModalOpen = $state(false);

    async function loadList() {
        isLoading = true;
        try {
            const query = {
                status: activeStatus === "ALL" ? undefined : activeStatus as ListStatus,
                contentType: activeType === "ALL" ? undefined : activeType
            };
            const res = await listApi.getList(query);
            entries = res.results;
        } catch (error) {
            console.error("Failed to load list:", error);
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        activeStatus;
        activeType;
        loadList();
    });

    let filteredEntries = $derived(
        entries.filter(e => e.title.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    const statusOptions = $derived([
        { value: "ALL", label: i18n.t('all'), icon: Library },
        { value: "CURRENT", label: i18n.t('current'), icon: PlayCircle },
        { value: "COMPLETED", label: i18n.t('done'), icon: CheckCircle2 },
        { value: "PLANNING", label: i18n.t('plan'), icon: Clock },
        { value: "PAUSED", label: i18n.t('paused_status'), icon: PauseCircle },
        { value: "DROPPED", label: i18n.t('dropped_status'), icon: XCircle }
    ]);

    function openEdit(entry: EnrichedListEntry) {
        selectedEntry = entry;
        isModalOpen = true;
    }

    function mapToMetadata(entry: EnrichedListEntry): CoreMetadata {
        return {
            cid: entry.cid,
            title: entry.title,
            coverImage: entry.coverImage,
            contentType: entry.contentType as ContentType,
            nsfw: false,
            characters: [],
            staff: [],
            externalIds: entry.externalIds,
            createdAt: 0,
            updatedAt: 0
        };
    }
</script>
<svelte:head>
    <title>{i18n.t('profile')}</title> <!-- Asumiendo que list es parte del perfil, o añade 'list' a tu en.ts si prefieres -->
</svelte:head>

<main class="min-h-screen bg-background pb-32 pt-16 md:pt-24 px-4 md:px-8 lg:px-12 xl:px-16 w-full max-w-[2400px] mx-auto space-y-6 md:space-y-10">

    <header class="flex flex-col sm:flex-row sm:items-center justify-between gap-5">
        <div class="space-y-1">
            <h1 class="text-2xl md:text-4xl font-black tracking-tight flex items-center gap-3">
                <Library class="h-7 w-7 md:h-10 md:w-10 text-primary" />
                {i18n.t('my_collection')}
            </h1>
            <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-80">
                {i18n.t('my_collection_desc')}
            </p>
        </div>

        <div class="relative w-full sm:w-80 group">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <Input
                    placeholder={i18n.t('quick_search_list')}
                    class="pl-10 bg-muted/20 border-none h-11 rounded-2xl focus-visible:ring-1 focus-visible:ring-primary/50 transition-all"
                    bind:value={searchQuery}
            />
        </div>
    </header>

    <section class="space-y-6">
        <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4 border-b border-border/40 pb-4 w-full overflow-hidden">
            <div class="min-w-0 w-full lg:w-auto">
                <Tabs.Root bind:value={activeStatus} class="w-full">
                    <Tabs.List
                            class="bg-transparent h-auto p-0 flex justify-start overflow-x-auto flex-nowrap hide-scrollbar gap-1.5 sm:gap-2 w-full"
                    >
                        {#each statusOptions as opt}
                            <Tabs.Trigger
                                    value={opt.value}
                                    class="relative px-3 sm:px-5 py-2.5 rounded-xl text-xs sm:text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-transparent data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/20 whitespace-nowrap shrink-0"
                            >
                                <opt.icon class="h-3.5 w-3.5 sm:h-4 sm:w-4 mr-1.5 sm:mr-2 inline-block" />
                                {opt.label}
                            </Tabs.Trigger>
                        {/each}
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="flex items-center gap-3 w-full sm:w-auto shrink-0">
                <span class="text-sm font-bold text-muted-foreground hidden lg:block">{i18n.t('format_label')}</span>
                <Select.Root type="single" bind:value={activeType}>
                    <Select.Trigger class="w-full sm:w-[160px] bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold">
                        <Filter class="h-4 w-4 mr-2 opacity-60" />
                        {activeType === "ALL" ? i18n.t('all_content') : i18n.t(activeType)}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="ALL">{i18n.t('all_content')}</Select.Item>
                        <Select.Item value="anime">{i18n.t('anime')}</Select.Item>
                        <Select.Item value="manga">{i18n.t('manga')}</Select.Item>
                        <Select.Item value="novel">{i18n.t('novel')}</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>
        </div>

        {#if isLoading}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 3xl:grid-cols-10 gap-4 md:gap-6">
                {#each Array(12) as _}
                    <div class="space-y-3">
                        <Skeleton class="aspect-[2/3] w-full rounded-2xl" />
                        <Skeleton class="h-4 w-3/4 rounded-md" />
                    </div>
                {/each}
            </div>
        {:else if filteredEntries.length === 0}
        {:else}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 3xl:grid-cols-10 gap-x-4 gap-y-10 md:gap-x-5 md:gap-y-12">
                {#each filteredEntries as entry (entry.cid)}
                    <div in:fade={{ duration: 300 }} class="group relative flex flex-col">
                        <div class="relative w-full overflow-hidden rounded-2xl shadow-lg border border-border/50 group-hover:border-primary/40 transition-all duration-300">
                            <ContentCard item={mapToMetadata(entry)} />

                            {#if entry.totalUnits}
                                <div class="absolute bottom-2 left-2 right-2 h-1.5 bg-black/60 rounded-full overflow-hidden backdrop-blur-md border border-white/10">
                                    <div
                                            class="h-full bg-primary transition-all duration-700 ease-out"
                                            style="width: {(entry.progress / entry.totalUnits) * 100}%"
                                    ></div>
                                </div>
                            {/if}
                        </div>

                        <div class="mt-4 flex items-start justify-between">
                            <div class="flex flex-col min-w-0 pr-2">
                                <span class="text-[10px] font-black uppercase tracking-widest text-primary truncate mb-1">
                                    {entry.status === 'CURRENT' ? (entry.contentType === 'anime' ? i18n.t('watching_status_list') : i18n.t('reading_status_list')) : i18n.t(entry.status.toLowerCase() + '_status') || entry.status}
                                </span>
                                <span class="text-sm font-black tabular-nums text-foreground/90">
                                    {entry.progress} <span class="text-muted-foreground/60 font-bold text-xs">/ {entry.totalUnits || '?'}</span>
                                </span>
                            </div>

                            <Button
                                    variant="secondary"
                                    size="icon"
                                    class="h-9 w-9 rounded-xl bg-muted/40 md:opacity-0 group-hover:opacity-100 transition-all shrink-0 shadow-sm hover:bg-primary hover:text-primary-foreground"
                                    onclick={() => openEdit(entry)}
                            >
                                <MoreVertical class="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</main>

{#if selectedEntry}
    <ListEditorModal
            bind:open={isModalOpen}
            cid={selectedEntry.cid}
            title={selectedEntry.title}
            contentType={selectedEntry.contentType}
            coverImage={selectedEntry.coverImage ?? undefined}
    />
{/if}

<style>
    .hide-scrollbar::-webkit-scrollbar {
        display: none;
    }
    .hide-scrollbar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>