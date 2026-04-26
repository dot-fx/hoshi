<script lang="ts">
    import type { ContentUnit } from "$lib/api/content/types";
    import type { AnimeProgress } from "@/api/progress/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { CheckCircle2 } from "lucide-svelte";

    let { cid, epsOrChapters, contentUnits = [], duration, progress = [] }: {
        cid: string,
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[],
        duration?: number | null,
        progress?: AnimeProgress[],
    } = $props();

    const progressMap = $derived(
        new Map(progress.map(p => [p.episode, p]))
    );

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
                thumbnail: u.thumbnailUrl.replace('_m.', '_w.'),
                enriched: true,
                duration: duration,
            }));
        }

        const totalEpisodes = epsOrChapters && epsOrChapters > 0 ? epsOrChapters : 12;
        return Array.from({ length: totalEpisodes }, (_, i) => ({
            number: i + 1,
            title: null,
            description: null,
            thumbnail: null,
            enriched: false,
            duration: null,
        }));
    });

    const resumeEpisode = $derived.by(() => {
        const inProgress = progress
            .filter(p => !p.completed && p.timestampSeconds && p.timestampSeconds > 0)
            .sort((a, b) => b.lastAccessed - a.lastAccessed)[0];
        if (inProgress) return inProgress.episode;

        const completedNums = new Set(progress.filter(p => p.completed).map(p => p.episode));
        const firstUnwatched = displayEpisodes.find(ep => !completedNums.has(ep.number));
        return firstUnwatched?.number ?? displayEpisodes[0]?.number ?? 1;
    });

    const formatDuration = (minutes: number | null) => {
        if (!minutes || minutes <= 0) return '';
        if (minutes < 60) return `${minutes}m`;
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        return m > 0 ? `${h}h ${m}m` : `${h}h`;
    };

    const formatTimestamp = (seconds: number) => {
        const m = Math.floor(seconds / 60);
        const s = seconds % 60;
        return `${m}:${s.toString().padStart(2, '0')}`;
    };

    const isRich = $derived(displayEpisodes.some(e => e.enriched));

    function scrollIfResume(node: HTMLElement, isResume: boolean) {
        if (isResume) {
            requestAnimationFrame(() => {
                node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            });
        }
        return {
            update(newIsResume: boolean) {
                if (newIsResume) {
                    requestAnimationFrame(() => {
                        node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                    });
                }
            }
        };
    }
</script>

<div class="flex flex-col h-full space-y-4">
    <div class="flex-1 overflow-y-auto pr-2 space-y-3 hide-scrollbar">
        {#each displayEpisodes as ep}
            {@const prog = progressMap.get(ep.number)}
            {@const isCompleted = prog?.completed ?? false}
            {@const isInProgress = !isCompleted && (prog?.timestampSeconds ?? 0) > 0}
            {@const isResume = ep.number === resumeEpisode}
            {@const progressPct = isInProgress && prog?.episodeDurationSeconds && prog.episodeDurationSeconds > 0
                ? Math.min(100, Math.round((prog.timestampSeconds! / prog.episodeDurationSeconds) * 100))
                : null}
            {@const href = isInProgress && prog?.timestampSeconds
                ? `/watch/${cid}/${ep.number}?t=${prog.timestampSeconds}`
                : `/watch/${cid}/${ep.number}`}

            <a
                    use:scrollIfResume={isResume}
                    {href}
                    class="group relative flex gap-4 border transition-all duration-200
                    {isCompleted
                        ? 'border-white/5 bg-white/[0.02] opacity-40 hover:opacity-70 hover:bg-white/[0.04] hover:border-white/8'
                        : isResume
                            ? 'border-primary/30 bg-primary/5 hover:bg-primary/10 hover:border-primary/50 ring-1 ring-primary/20'
                            : 'border-white/5 bg-white/[0.02] hover:bg-white/[0.05] hover:border-white/10'}"
            >
                <!-- Thumbnail -->
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

                    <!-- Duration pill -->
                    {#if ep.duration}
                        <div class="absolute bottom-1 right-1 px-1.5 py-0.5 bg-black/80 text-[10px] font-medium text-white rounded">
                            {formatDuration(ep.duration)}
                        </div>
                    {/if}

                    <!-- Completed checkmark overlay -->
                    {#if isCompleted}
                        <div class="absolute inset-0 flex items-center justify-center bg-black/30">
                            <CheckCircle2 class="w-8 h-8 text-primary/70" />
                        </div>
                    {/if}

                    <!-- Progress bar at bottom of thumbnail -->
                    {#if progressPct !== null}
                        <div class="absolute bottom-0 inset-x-0 h-1 bg-white/10">
                            <div
                                    class="h-full bg-primary transition-all duration-300"
                                    style="width: {progressPct}%"
                            ></div>
                        </div>
                    {/if}
                </div>

                <!-- Metadata -->
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

                    <!-- Status row -->
                    <div class="flex items-center gap-2 mt-2">
                        {#if isResume && !isCompleted}
                            <span class="text-[10px] font-bold uppercase tracking-widest text-primary/80 bg-primary/10 px-2 py-0.5 rounded-sm">
                                {isInProgress ? i18n.t('content.resume') : i18n.t('home.hero.watch')}
                            </span>
                        {/if}
                        {#if isInProgress && prog?.timestampSeconds}
                            <span class="text-[10px] text-muted-foreground/40 font-mono tabular-nums">
                                {formatTimestamp(prog.timestampSeconds)}
                                {#if progressPct !== null}· {progressPct}%{/if}
                            </span>
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