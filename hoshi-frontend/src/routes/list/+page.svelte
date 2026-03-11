<script lang="ts">
    import { listApi } from "$lib/api/list/list";
    import type { EnrichedListEntry, ListStatus } from "$lib/api/list/types";
    import type { ContentWithMappings, ContentType } from "$lib/api/content/types";

    import ContentCard from "$lib/components/home/ContentCard.svelte";
    import ListEditorModal from "$lib/components/ListEditorModal.svelte";

    import * as Tabs from "$lib/components/ui/tabs";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";

    import { Input } from "$lib/components/ui/input";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";

    import {
        Search,
        Library,
        Filter,
        MoreVertical,
        CheckCircle2,
        PlayCircle,
        Clock,
        PauseCircle,
        XCircle
    } from "lucide-svelte";

    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";

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
        entries.filter(e =>
            e.title.toLowerCase().includes(searchQuery.toLowerCase())
        )
    );

    const statusOptions = $derived([
        { value: "ALL", label: i18n.t('all') || 'All', icon: Library },
        { value: "CURRENT", label: i18n.t('current') || 'Watching', icon: PlayCircle },
        { value: "COMPLETED", label: i18n.t('done') || 'Completed', icon: CheckCircle2 },
        { value: "PLANNING", label: i18n.t('plan') || 'Plan to Watch', icon: Clock },
        { value: "PAUSED", label: i18n.t('paused_status') || 'Paused', icon: PauseCircle },
        { value: "DROPPED", label: i18n.t('dropped_status') || 'Dropped', icon: XCircle }
    ]);

    function openEdit(entry: EnrichedListEntry) {
        selectedEntry = entry;
        isModalOpen = true;
    }

    function mapToContentWithMappings(entry: EnrichedListEntry): ContentWithMappings {
        return {
            content: {
                cid: entry.cid,
                contentType: entry.contentType as ContentType,
                nsfw: entry.nsfw,
                createdAt: 0,
                updatedAt: 0
            },
            metadata: [{
                cid: entry.cid,
                sourceName: "list",
                title: entry.title,
                coverImage: entry.coverImage || undefined,
                subtype: entry.contentType === 'anime' ? 'TV' : 'MANGA',
                status: undefined,
                rating: undefined,
                epsOrChapters: entry.totalUnits || null,
                characters: [],
                staff: [],
                externalIds: entry.externalIds as any || {},
                createdAt: 0,
                updatedAt: 0
            }],
            trackerMappings: [],
            extensionSources: [],
            relations: [],
            contentUnits: []
        };
    }
</script>

<svelte:head>
    <title>{i18n.t('list')}}</title>
</svelte:head>

<div class="min-h-screen bg-background pb-28 md:pb-12">


    <div class="w-full bg-muted/10 border-b border-border/20 pt-8 pb-6 md:pt-12 md:pb-8">
        <div class="max-w-[2000px] mx-auto px-4 md:px-8 lg:px-12">
            <header class="flex flex-col md:flex-row md:items-end justify-between gap-6">

                <div class="space-y-2">
                    <h1 class="text-3xl md:text-5xl font-black tracking-tight flex items-center gap-3">
                        <Library class="h-8 w-8 md:h-10 md:w-10 text-primary" />
                        {i18n.t('my_collection') || 'My Collection'}
                    </h1>
                    <p class="text-sm md:text-base text-muted-foreground font-medium opacity-80 max-w-xl">
                        {i18n.t('my_collection_desc') || 'Manage and track all the shows and mangas you are following.'}
                    </p>
                </div>

                <!-- Buscador más elegante -->
                <div class="relative w-full md:w-80 group">
                    <Search class="absolute left-3.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                    <Input
                            placeholder={i18n.t('quick_search_list') || 'Search in your list...'}
                            class="pl-10 bg-background border border-border/40 shadow-sm h-11 rounded-xl focus-visible:ring-1 focus-visible:ring-primary/50 transition-all"
                            bind:value={searchQuery}
                    />
                </div>

            </header>
        </div>
    </div>

    <!-- CONTENIDO PRINCIPAL -->
    <main class="max-w-[2000px] mx-auto px-4 md:px-8 lg:px-12 pt-8 space-y-8">

        <!-- BARRA DE FILTROS Y TABS -->
        <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-4 w-full">

            <div class="min-w-0 w-full lg:w-auto overflow-hidden">
                <Tabs.Root bind:value={activeStatus} class="w-full">
                    <Tabs.List class="bg-transparent h-auto p-0 flex justify-start overflow-x-auto flex-nowrap hide-scrollbar gap-2 w-full border-b border-transparent">
                        {#each statusOptions as opt}
                            <Tabs.Trigger
                                    value={opt.value}
                                    class="relative px-4 py-2.5 rounded-full text-xs sm:text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-border/40 data-[state=active]:border-primary data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/30 whitespace-nowrap shrink-0 shadow-sm"
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
                        {activeType === "ALL" ? (i18n.t('all_content') || 'All Formats') : i18n.t(activeType) || activeType}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="ALL" class="font-bold">{i18n.t('all_content') || 'All Formats'}</Select.Item>
                        <Select.Item value="anime" class="font-bold">{i18n.t('anime') || 'Anime'}</Select.Item>
                        <Select.Item value="manga" class="font-bold">{i18n.t('manga') || 'Manga'}</Select.Item>
                        <Select.Item value="novel" class="font-bold">{i18n.t('novel') || 'Novel'}</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>
        </div>

        <!-- GRID DE RESULTADOS -->
        {#if isLoading}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 gap-4 md:gap-6">
                {#each Array(14) as _}
                    <div class="space-y-3">
                        <Skeleton class="aspect-[2/3] w-full rounded-xl bg-muted/20" />
                    </div>
                {/each}
            </div>

        {:else if filteredEntries.length === 0}
            <Empty.Root class="border border-dashed border-border/40 bg-muted/5 rounded-2xl py-20 min-h-[40vh] flex items-center justify-center">
                <Empty.Header>
                    <Empty.Media variant="icon" class="bg-primary/10 text-primary mb-4 p-4 rounded-full">
                        <Library class="size-8" />
                    </Empty.Media>
                    <Empty.Title class="text-xl font-bold">
                        {i18n.t('empty_list_title') || 'Nothing here yet'}
                    </Empty.Title>
                    <Empty.Description class="text-muted-foreground font-medium">
                        {i18n.t('empty_list_desc') || 'Your collection is looking a bit empty. Go find some good content!'}
                    </Empty.Description>
                </Empty.Header>
            </Empty.Root>

        {:else}
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 gap-x-4 gap-y-6 md:gap-x-5 md:gap-y-8">
                {#each filteredEntries as entry (entry.cid)}

                    <div in:fade={{ duration: 300 }} class="group relative flex flex-col w-full h-full">

                        <!-- El ContentCard que ya incluye la imagen, título, efectos hover, etc. -->
                        <div class="relative w-full h-full">

                            <!-- BARRITA DE PROGRESO FLOTANTE -->
                            <!-- Se posiciona abajo, encima de la imagen del ContentCard, al estilo Anilist -->
                            {#if entry.totalUnits && entry.progress > 0}
                                <div class="absolute bottom-1.5 left-1.5 right-1.5 z-20 h-1.5 bg-black/60 rounded-full overflow-hidden backdrop-blur-md border border-white/10 pointer-events-none">
                                    <div
                                            class="h-full bg-primary transition-all duration-700 ease-out"
                                            style="width: {Math.min((entry.progress / entry.totalUnits) * 100, 100)}%"
                                    ></div>
                                </div>
                            {/if}

                            <ContentCard item={mapToContentWithMappings(entry)} disableHover={true} />

                            <div class="absolute top-2 left-2 right-2 z-20 flex justify-between items-start opacity-100 lg:opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">

                                <span class="bg-black/80 backdrop-blur-md px-2 py-1 rounded-md text-[10px] font-black uppercase tracking-widest text-primary border border-white/10 shadow-sm">
                                    {entry.progress} / {entry.totalUnits || '?'}
                                </span>

                                <Button
                                        variant="secondary"
                                        size="icon"
                                        class="h-7 w-7 rounded-md bg-black/80 text-white border border-white/10 hover:bg-primary hover:text-primary-foreground pointer-events-auto shadow-sm"
                                        onclick={(e) => { e.preventDefault(); openEdit(entry); }}
                                        title="Edit Entry"
                                >
                                    <MoreVertical class="h-4 w-4" />
                                </Button>

                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </main>
</div>

{#if selectedEntry}
    <ListEditorModal
            bind:open={isModalOpen}
            cid={selectedEntry.cid}
            title={selectedEntry.title}
            contentType={selectedEntry.contentType}
            coverImage={selectedEntry.coverImage ?? undefined}
    />
{/if}