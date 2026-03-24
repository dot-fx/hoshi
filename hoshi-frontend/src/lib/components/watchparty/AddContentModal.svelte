<script lang="ts">
    import * as Dialog from "@/components/ui/dialog";
    import { Input } from "@/components/ui/input";
    import { Button } from "@/components/ui/button";
    import { ScrollArea } from "@/components/ui/scroll-area";
    import { Search, ChevronLeft, ListPlus } from "lucide-svelte";
    import { contentApi } from "@/api/content/content";
    import { primaryMetadata, type ContentWithMappings } from "@/api/content/types";
    import Card from "@/components/content/Card.svelte";
    import { i18n } from "@/i18n/index.svelte.js";
    import {Spinner} from "@/components/ui/spinner";

    let { open = $bindable(false), onAdd } = $props<{
        open: boolean;
        onAdd: (item: any) => void;
    }>();

    let query = $state("");
    let results = $state<ContentWithMappings[]>([]);
    let selectedContent = $state<ContentWithMappings | null>(null);
    let loading = $state(false);

    let batchStart = $state(1);
    let batchEnd = $state(1);

    $effect(() => {
        if (!open) {
            query = "";
            results = [];
            selectedContent = null;
        }
    });

    $effect(() => {
        if (selectedContent) {
            const meta = primaryMetadata(selectedContent);
            batchStart = 1;
            batchEnd = meta?.epsOrChapters || 12;
        }
    });

    async function handleSearch() {
        if (!query.trim()) return;
        loading = true;
        try {
            const res = await contentApi.search({ query: query.trim(), type: 'anime', limit: 20 });
            results = res.data;
        } catch (err) {
            console.error("Search error:", err);
        } finally {
            loading = false;
        }
    }

    function addEpisode(unitNumber: number, closeAfter = true) {
        if (!selectedContent) return;
        const meta = primaryMetadata(selectedContent);

        const item = {
            id: crypto.randomUUID(),
            title: meta?.title || i18n.t('watchparty.content_modal.untitled'),
            thumbnail: meta?.coverImage || undefined,
            metadata: {
                contentId: selectedContent.content.cid,
                unitNumber: unitNumber,
                seriesTitle: meta?.title || ""
            }
        };
        onAdd(item);
        if (closeAfter) open = false;
    }

    function addBatch() {
        const start = Math.max(1, batchStart);
        const end = Math.max(start, batchEnd);

        for (let i = start; i <= end; i++) {
            addEpisode(i, false);
        }
        open = false; // Cerramos al terminar todo el lote
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[1000px] h-[85vh] flex flex-col p-0 overflow-hidden rounded-3xl border-border/40 shadow-2xl bg-background">

        <div class="p-6 border-b border-border/40 bg-muted/10 flex items-center gap-4">
            {#if selectedContent}
                <Button variant="ghost" size="icon" onclick={() => selectedContent = null} class="rounded-xl">
                    <ChevronLeft class="w-5 h-5" />
                </Button>
            {/if}
            <div class="flex-1">
                <Dialog.Title class="text-xl font-black">
                    {selectedContent ? primaryMetadata(selectedContent)?.title : i18n.t('watchparty.content_modal.add_content')}
                </Dialog.Title>
            </div>
        </div>

        <div class="flex-1 flex flex-col overflow-hidden p-6">
            {#if !selectedContent}
                <form onsubmit={(e) => { e.preventDefault(); handleSearch(); }} class="flex gap-2 mb-6">
                    <div class="relative flex-1">
                        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                        <Input
                                bind:value={query}
                                placeholder={i18n.t('watchparty.content_modal.search_placeholder')}
                                class="pl-10 h-12 rounded-xl bg-muted/20 border-border/50 font-medium"
                        />
                    </div>
                    <Button type="submit" class="h-12 px-6 rounded-xl font-bold" disabled={loading}>
                        {#if loading} <Spinner class="w-4 h-4 animate-spin mr-2" /> {/if}
                        {i18n.t('watchparty.content_modal.search')}
                    </Button>
                </form>

                <ScrollArea class="flex-1">
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 pr-4 pb-4">
                        {#each results as content}
                            <div
                                    role="button"
                                    tabindex="0"
                                    onclickcapture={(e) => {
                                    e.preventDefault();
                                    e.stopPropagation();
                                    selectedContent = content;
                                }}
                                    class="cursor-pointer transition-transform hover:scale-[1.02]"
                            >
                                <Card item={content} disableHover={true} />
                            </div>
                        {/each}
                    </div>
                </ScrollArea>

            {:else}
                {@const meta = primaryMetadata(selectedContent)}
                <div class="flex flex-col md:flex-row gap-6 h-full">
                    <div class="w-48 shrink-0 hidden md:block">
                        <img src={meta?.coverImage} alt={meta?.title} class="w-full aspect-[2/3] object-cover rounded-2xl shadow-lg border border-border/40" />
                    </div>

                    <div class="flex-1 flex flex-col overflow-hidden">
                        <p class="text-sm text-muted-foreground font-medium mb-4 line-clamp-3">
                            {meta?.synopsis || i18n.t('watchparty.content_modal.no_synopsis')}
                        </p>

                        <div class="flex flex-wrap items-center gap-3 bg-muted/20 p-3 rounded-xl border border-border/40 mb-4 shrink-0">
                            <span class="text-sm font-bold text-muted-foreground ml-1 flex items-center gap-1.5">
                                <ListPlus class="w-4 h-4" /> {i18n.t('watchparty.content_modal.add_batch')}
                            </span>
                            <div class="flex items-center gap-2">
                                <Input type="number" bind:value={batchStart} min="1" max={Math.max(1, batchEnd)} class="w-20 h-9 bg-background font-bold" />
                                <span class="text-sm font-medium text-muted-foreground">{i18n.t('watchparty.content_modal.to')}</span>
                                <Input type="number" bind:value={batchEnd} min={batchStart} class="w-20 h-9 bg-background font-bold" />
                            </div>
                            <Button size="sm" class="ml-auto font-bold rounded-lg shadow-sm" onclick={addBatch}>
                                {i18n.t('watchparty.content_modal.add_n_episodes', { count: Math.max(0, batchEnd - batchStart + 1) })}
                            </Button>
                        </div>

                        <ScrollArea class="flex-1 pr-4">
                            <div class="grid grid-cols-4 sm:grid-cols-5 lg:grid-cols-8 gap-2 pb-4">
                                {#each Array.from({ length: meta?.epsOrChapters || 24 }) as _, i}
                                    <Button
                                            variant="outline"
                                            class="h-12 rounded-lg font-bold border-border/50 hover:bg-primary hover:text-primary-foreground hover:border-primary transition-all"
                                            onclick={() => addEpisode(i + 1)}
                                    >
                                        {i + 1}
                                    </Button>
                                {/each}
                            </div>
                        </ScrollArea>
                    </div>
                </div>
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>