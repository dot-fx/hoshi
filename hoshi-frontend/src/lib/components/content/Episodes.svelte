<script lang="ts">
    import type { ContentUnit } from "$lib/api/content/types";
    import { i18n } from "@/stores/i18n.svelte.js";

    let { cid, epsOrChapters, contentUnits = [], duration }: {
        cid: string,
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[],
        duration?: number | null,
    } = $props();

    const displayEpisodes = $derived.by(() => {
        const enrichedEpisodes = (contentUnits ?? [])
            .filter(u =>
                u.contentType === 'episode' &&
                u.title &&
                u.thumbnailUrl
            )
            .sort((a, b) => a.unitNumber - b.unitNumber);

        if (enrichedEpisodes.length > 0) {
            return enrichedEpisodes.map(u => ({
                number: u.unitNumber,
                title: u.title,
                description: u.description || null,
                thumbnail: u.thumbnailUrl.replace('_m.', '_w.') ,
                isWatched: false,
                enriched: true,
                duration: duration
            }));
        }

        const totalEpisodes = epsOrChapters && epsOrChapters > 0 ? epsOrChapters : 12;
        return Array.from({ length: totalEpisodes }, (_, i) => ({
            number: i + 1,
            title: null,
            description: null,
            thumbnail: null,
            isWatched: false,
            enriched: false,
            duration: null,
        }));
    });

    const formatDuration = (minutes: number | null) => {
        if (!minutes || minutes <= 0) return '';
        if (minutes < 60) return `${minutes}m`;
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        return m > 0 ? `${h}h ${m}m` : `${h}h`;
    };

    const isRich = $derived(displayEpisodes.some(e => e.enriched));
</script>

<div class="flex flex-col h-full space-y-4">
    <div class="flex-1 overflow-y-auto pr-2 space-y-3 hide-scrollbar">
        {#each displayEpisodes as ep}
            <a
                    href={`/watch/${cid}/${ep.number}`}
                    class="group flex gap-4 border border-white/5 bg-white/[0.02] hover:bg-white/[0.05] hover:border-white/10 transition-all duration-200 {ep.isWatched ? 'opacity-40' : ''}"
            >
                <div class="relative shrink-0 w-48 aspect-video overflow-hidden bg-muted/20">
                    {#if ep.thumbnail}
                        <img
                                src={ep.thumbnail}
                                alt={ep.title ?? `EP ${ep.number}`}
                                class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                        />
                    {:else}
                        <div class="w-full h-full flex items-center justify-center">
                            <span class="text-4xl font-black text-white/5">{ep.number}</span>
                        </div>
                    {/if}

                    {#if ep.duration}
                        <div class="absolute bottom-1 right-1 px-1.5 py-0.5 bg-black/80 text-[10px] font-medium text-white rounded">
                            {formatDuration(ep.duration)}
                        </div>
                    {/if}
                </div>

                <div class="flex-1 min-w-0 py-3 pr-4 flex flex-col justify-between">
                    <div class="space-y-1">
                        <div class="flex justify-between items-start gap-2">
                            <p class="font-bold text-[15px] leading-tight line-clamp-1 group-hover:text-primary transition-colors">
                                {#if isRich && ep.title}
                                    {ep.number}. {ep.title}
                                {:else}
                                    {i18n.t('content.episode_title', { num: ep.number })}
                                {/if}
                            </p>
                        </div>
                        {#if ep.description}
                            <p class="text-[11px] text-muted-foreground/60 line-clamp-2 leading-relaxed">
                                {ep.description}
                            </p>
                        {/if}
                    </div>

                </div>
            </a>
        {/each}
    </div>
</div>

<style>
    .hide-scrollbar::-webkit-scrollbar { display: none; }
    .hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>