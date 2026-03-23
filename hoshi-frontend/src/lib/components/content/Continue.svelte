<script lang="ts">
    import { Progress } from "$lib/components/ui/progress";
    import { PlayCircle, FileText } from "lucide-svelte";
    import type { ContinueItem } from '@/api/progress/types';
    import type { ContentType } from '@/api/content/types';
    import {i18n} from "@/i18n/index.svelte";

    let {
        items,
        mode
    }: {
        items: ContinueItem[];
        mode: ContentType;
    } = $props();

    function getContinueUrl(item: ContinueItem) {
        if (item.contentType === 'anime' && item.episode) {
            const ratio = (item.episodeDurationSeconds && item.timestampSeconds)
                ? item.timestampSeconds / item.episodeDurationSeconds
                : 0;

            if (ratio >= 0.95) {
                return `/watch/${item.cid}/${item.episode + 1}`;
            } else if (item.timestampSeconds && item.timestampSeconds > 0) {
                return `/watch/${item.cid}/${item.episode}?t=${item.timestampSeconds}`;
            } else {
                return `/watch/${item.cid}/${item.episode}`;
            }
        }
        return `/content/${item.cid}`;
    }
</script>

<div class="space-y-4">
    <h2 class="text-xl md:text-2xl font-black tracking-tight flex items-center gap-2 text-foreground">
        {mode === 'anime' ? i18n.t("home.continue.continue_watching") : i18n.t("home.continue.continue_reading")}
    </h2>
    <div class="flex overflow-x-auto gap-4 pb-4 custom-scrollbar snap-x">
        {#each items as item}
            <a
                    href={getContinueUrl(item)}
                    class="group flex items-center gap-4 p-3 bg-card/90 backdrop-blur-md border border-border/60 hover:border-primary/50 rounded-2xl min-w-[280px] max-w-[320px] snap-start transition-all hover:shadow-md"
            >
                <div class="relative h-24 w-16 shrink-0 rounded-lg overflow-hidden bg-muted/50 border border-border/50">
                    {#if item.coverImage}
                        <img src={item.coverImage} alt={item.title} class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                    {:else}
                        <div class="w-full h-full flex items-center justify-center opacity-30 text-muted-foreground">
                            {#if mode === 'anime'}
                                <PlayCircle class="size-6" />
                            {:else}
                                <FileText class="size-6" />
                            {/if}
                        </div>
                    {/if}
                </div>

                <div class="flex flex-col flex-1 min-w-0 h-full py-0.5">
                    <h3 class="font-bold text-sm leading-tight line-clamp-2 mb-1 group-hover:text-primary transition-colors" title={item.title}>{item.title}</h3>
                    <div class="mt-auto">
                        {#if mode === 'anime' && item.episode}
                            <div class="flex items-center justify-between mb-2">
                                <span class="text-xs font-bold text-muted-foreground">{i18n.t('home.continue.episodes', { num: item.episode })}</span>
                            </div>
                            {#if item.episodeDurationSeconds && item.timestampSeconds}
                                <Progress value={(item.timestampSeconds / item.episodeDurationSeconds) * 100} max={100} class="h-1.5 w-full bg-muted" />
                            {/if}
                        {:else if item.chapter}
                            <span class="text-xs font-bold text-muted-foreground bg-foreground/5 px-2 py-1 rounded-md">{i18n.t('home.continue.chapters', { num: item.chapter })}</span>
                        {/if}
                    </div>
                </div>
            </a>
        {/each}
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { height: 6px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150,150,150,0.3); border-radius: 10px; }
</style>