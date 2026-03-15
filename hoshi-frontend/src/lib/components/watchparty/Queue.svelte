<script lang="ts">
    import { Button } from '@/components/ui/button';
    import { Play, Trash2, ListVideo, Image as ImageIcon, ChevronDown, ChevronUp } from 'lucide-svelte';
    import type { PlaylistItem } from '@/api/watchparty/types';
    import { i18n } from "@/i18n/index.svelte.js";

    let props = $props<{
        queue?: PlaylistItem[];
        currentItem?: PlaylistItem | null;
        isHost?: boolean;
        onPlayItem: (itemId: string) => void;
        onRemoveItem: (itemId: string) => void;
        onReorder?: (orderedIds: string[]) => void;
    }>();

    let expandedGroups = $state<Record<string, boolean>>({});

    let groupedQueue = $derived.by(() => {
        const groups: any[] = [];
        for (const item of props.queue || []) {
            const lastGroup = groups[groups.length - 1];
            const cid = item.metadata?.contentId;

            if (lastGroup && cid && lastGroup.contentId === cid) {
                lastGroup.items.push(item);
            } else {
                groups.push({
                    id: item.id,
                    contentId: cid,
                    seriesTitle: item.metadata?.seriesTitle || item.title,
                    thumbnail: item.thumbnail || item.metadata?.coverImage,
                    items: [item]
                });
            }
        }
        return groups;
    });

    $effect(() => {
        if (props.currentItem) {
            for (const group of groupedQueue) {
                if (group.items.length > 1 && group.items.some((i: any) => i.id === props.currentItem!.id)) {
                    expandedGroups[group.id] = true;
                }
            }
        }
    });

    function moveItem(itemId: string, direction: -1 | 1) {
        const q = props.queue || [];
        const idx = q.findIndex(i => i.id === itemId);
        if (idx < 0) return;

        const newIdx = idx + direction;
        if (newIdx < 0 || newIdx >= q.length) return;

        const newIds = q.map(i => i.id);
        const temp = newIds[idx];
        newIds[idx] = newIds[newIdx];
        newIds[newIdx] = temp;

        props.onReorder?.(newIds);
    }
</script>

{#snippet episodeRow(item: PlaylistItem, isCompact: boolean)}
    {@const isPlaying = props.currentItem?.id === item.id}
    <div class="flex items-center gap-3 p-2 rounded-xl border transition-colors relative group {isPlaying ? 'border-primary bg-primary/10' : 'border-border/40 bg-muted/10 hover:bg-muted/20'}">

        <div class="{isCompact ? 'w-16 h-10' : 'w-24 h-14'} bg-black/20 rounded-lg overflow-hidden shrink-0 relative flex items-center justify-center">
            {#if item.thumbnail || item.metadata?.coverImage}
                <img src={item.thumbnail || item.metadata?.coverImage} alt={item.title} class="w-full h-full object-cover" />
            {:else}
                <ImageIcon class="w-6 h-6 text-muted-foreground/50" />
            {/if}

            {#if isPlaying}
                <div class="absolute inset-0 bg-primary/20 flex items-center justify-center backdrop-blur-[1px]">
                    <div class="w-3 h-3 rounded-full bg-primary animate-ping"></div>
                </div>
            {/if}
        </div>

        <div class="flex-1 min-w-0">
            <h4 class="font-bold {isCompact ? 'text-xs' : 'text-sm'} text-foreground line-clamp-1">{item.title}</h4>
            {#if item.metadata?.seriesTitle}
                <p class="text-[10px] text-muted-foreground line-clamp-1 font-medium mt-0.5">
                    {!isCompact ? `${item.metadata.seriesTitle} - ` : ''}{i18n.t('watchparty.episode')} {item.metadata.unitNumber}
                </p>
            {/if}
        </div>

        {#if props.isHost}
            <div class="flex opacity-0 group-hover:opacity-100 transition-opacity gap-0.5 shrink-0 pr-1">
                <div class="flex flex-col gap-0.5 mr-1">
                    <button type="button" onclick={(e) => { e.stopPropagation(); moveItem(item.id, -1); }} class="text-muted-foreground hover:text-foreground transition-colors p-0.5" title={i18n.t('watchparty.move_up')}>
                        <ChevronUp class="w-3.5 h-3.5" />
                    </button>
                    <button type="button" onclick={(e) => { e.stopPropagation(); moveItem(item.id, 1); }} class="text-muted-foreground hover:text-foreground transition-colors p-0.5" title={i18n.t('watchparty.move_down')}>
                        <ChevronDown class="w-3.5 h-3.5" />
                    </button>
                </div>

                {#if !isPlaying}
                    <Button type="button" variant="ghost" size="icon" class="h-8 w-8 rounded-lg hover:bg-primary hover:text-primary-foreground" onclick={(e) => { e.stopPropagation(); props.onPlayItem(item.id); }}>
                        <Play class="w-4 h-4" />
                    </Button>
                {/if}

                <Button type="button" variant="ghost" size="icon" class="h-8 w-8 rounded-lg hover:bg-destructive hover:text-destructive-foreground" onclick={(e) => { e.stopPropagation(); props.onRemoveItem(item.id); }}>
                    <Trash2 class="w-4 h-4" />
                </Button>
            </div>
        {/if}
    </div>
{/snippet}

<div class="flex flex-col h-full w-full bg-card overflow-hidden">
    <div class="flex-1 overflow-y-auto p-3 flex flex-col gap-2 scroll-smooth">
        {#if groupedQueue.length === 0}
            <div class="h-full flex flex-col items-center justify-center text-muted-foreground text-sm font-medium gap-3 opacity-60">
                <ListVideo class="w-12 h-12" />
                <p>{i18n.t('watchparty.empty_queue')}</p>
            </div>
        {:else}
            {#each groupedQueue as group (group.id)}
                {#if group.items.length === 1}
                    {@render episodeRow(group.items[0], false)}
                {:else}
                    <div class="flex flex-col border border-border/40 bg-muted/5 rounded-xl overflow-hidden">
                        <button
                                type="button"
                                onclick={() => expandedGroups[group.id] = !expandedGroups[group.id]}
                                class="flex items-center gap-3 p-2 hover:bg-muted/10 transition-colors text-left w-full"
                        >
                            <div class="w-16 h-10 bg-black/20 rounded-md overflow-hidden shrink-0">
                                {#if group.thumbnail}
                                    <img src={group.thumbnail} alt={group.seriesTitle} class="w-full h-full object-cover" />
                                {:else}
                                    <ImageIcon class="w-4 h-4 text-muted-foreground/50 m-auto mt-3" />
                                {/if}
                            </div>
                            <div class="flex-1">
                                <h4 class="font-bold text-sm line-clamp-1">{group.seriesTitle}</h4>
                                <p class="text-xs text-primary font-semibold">{group.items.length} {i18n.t('watchparty.episodes').toLowerCase()}</p>
                            </div>
                            <ChevronDown class="w-5 h-5 text-muted-foreground transition-transform duration-200 {expandedGroups[group.id] ? 'rotate-180' : ''}" />
                        </button>

                        {#if expandedGroups[group.id]}
                            <div class="flex flex-col gap-1 p-2 border-t border-border/40 bg-background/50">
                                {#each group.items as item}
                                    {@render episodeRow(item, true)}
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            {/each}
        {/if}
    </div>
</div>