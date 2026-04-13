<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import * as Pagination from "$lib/components/ui/pagination";
    import { PlayCircle } from "lucide-svelte";
    import type { ContentUnit } from "$lib/api/content/types";
    import { i18n } from "$lib/i18n/index.svelte";

    let { cid, epsOrChapters, contentUnits = [] }: {
        cid: string,
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[]
    } = $props();

    let currentPage = $state(1);
    const pageSize = 24;

    const displayEpisodes = $derived.by(() => {
        if (contentUnits && contentUnits.length > 0) {
            const regularEpisodes = contentUnits
                .filter(u => u.contentType === 'episode')
                .sort((a, b) => a.unitNumber - b.unitNumber);

            if (regularEpisodes.length > 0) {
                return regularEpisodes.map(u => ({
                    number: u.unitNumber,
                    title: u.title || i18n.t('content.episode_number', { num: u.unitNumber }),
                    description: u.description,
                    thumbnail: u.thumbnailUrl ? u.thumbnailUrl.replace('_m.', '_w.') : null,
                    isWatched: false
                }));
            }
        }

        const totalEpisodes = epsOrChapters && epsOrChapters > 0 ? epsOrChapters : 12;
        return Array.from({ length: totalEpisodes }, (_, i) => ({
            number: i + 1,
            title: i18n.t('content.episode_title', {num: i + 1}),
            description: null,
            thumbnail: null,
            isWatched: false
        }));
    });

    const paginatedEpisodes = $derived(
        displayEpisodes.slice((currentPage - 1) * pageSize, currentPage * pageSize)
    );

    const isRichMode = $derived(displayEpisodes.length > 0 && displayEpisodes[0]?.thumbnail);
</script>

<div class="space-y-6">
    <h2 class="text-xl md:text-2xl font-bold tracking-tight">{i18n.t('content.episodes_title')}</h2>

    {#if isRichMode}
        <div class="w-full">
            <div class="flex flex-col gap-5 sm:hidden w-full">
                {#each paginatedEpisodes as ep}
                    <a href={`/watch/${cid}/${ep.number}`} class="group/ep cursor-pointer flex gap-4 transition-colors">
                        <div class="relative w-36 shrink-0 aspect-video bg-muted rounded-xl overflow-hidden border border-border/40 shadow-sm">
                            {#if ep.thumbnail}
                                <img src={ep.thumbnail} alt={ep.title} class="h-full w-full object-cover group-hover/ep:scale-105 transition-transform duration-300" />
                            {:else}
                                <div class="h-full w-full flex items-center justify-center bg-muted/80">
                                    <span class="text-2xl font-black text-muted-foreground/30">{ep.number}</span>
                                </div>
                            {/if}
                            <div class="absolute inset-0 bg-black/40 opacity-0 group-hover/ep:opacity-100 transition-opacity flex items-center justify-center">
                                <PlayCircle class="h-8 w-8 text-white drop-shadow-lg" />
                            </div>
                            <div class="absolute bottom-1 left-1 bg-background/90 backdrop-blur-md px-1.5 py-0.5 rounded text-[10px] font-bold shadow-sm">
                                EP {ep.number}
                            </div>
                            {#if ep.isWatched}
                                <div class="absolute bottom-0 left-0 h-1 bg-primary w-full"></div>
                            {/if}
                        </div>

                        <div class="flex flex-col justify-center py-1 flex-1 min-w-0">
                            <h3 class="font-bold text-sm leading-tight line-clamp-2 group-hover/ep:text-primary transition-colors text-foreground/90">
                                {ep.title}
                            </h3>
                            {#if ep.description}
                                <p class="text-xs text-muted-foreground line-clamp-2 leading-relaxed mt-1">
                                    {ep.description}
                                </p>
                            {/if}
                        </div>
                    </a>
                {/each}
            </div>

            <div class="hidden sm:grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 w-full">
                {#each paginatedEpisodes as ep}
                    <a href={`/watch/${cid}/${ep.number}`} class="group/card flex flex-col h-full overflow-hidden rounded-xl border border-border/40 bg-card shadow-sm transition-all hover:border-primary/50 cursor-pointer">
                        <div class="relative aspect-video w-full overflow-hidden bg-muted">
                            {#if ep.thumbnail}
                                <img src={ep.thumbnail} alt={ep.title} class="h-full w-full object-cover transition-transform duration-300 group-hover/card:scale-105" />
                            {:else}
                                <div class="h-full w-full flex items-center justify-center bg-muted/80">
                                    <span class="text-4xl font-black text-muted-foreground/30">{ep.number}</span>
                                </div>
                            {/if}
                            <div class="absolute inset-0 bg-black/40 opacity-0 transition-opacity group-hover/card:opacity-100 flex items-center justify-center">
                                <PlayCircle class="h-12 w-12 text-white scale-90 transition-transform group-hover/card:scale-100 drop-shadow-lg" />
                            </div>
                            <div class="absolute bottom-2 left-2 bg-background/90 backdrop-blur-md px-2 py-0.5 rounded-md text-xs font-bold shadow-sm">
                                EP {ep.number}
                            </div>
                            {#if ep.isWatched}
                                <div class="absolute bottom-0 left-0 h-1 bg-primary/60 w-full"></div>
                            {/if}
                        </div>

                        <div class="flex flex-col flex-1 p-3.5 space-y-1.5">
                            <h3 class="font-bold text-sm leading-tight line-clamp-2 group-hover/card:text-primary transition-colors" title={ep.title}>
                                {ep.title}
                            </h3>
                            {#if ep.description}
                                <p class="text-[13px] text-muted-foreground line-clamp-2 leading-relaxed mt-auto pt-1">
                                    {ep.description}
                                </p>
                            {:else}
                                <p class="text-[13px] text-muted-foreground/40 italic mt-auto pt-1">{i18n.t('content.no_description_ep')}</p>
                            {/if}
                        </div>
                    </a>
                {/each}
            </div>
        </div>
    {:else}
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3 w-full">
            {#each paginatedEpisodes as ep}
                <Button
                        href={`/watch/${cid}/${ep.number}`}
                        variant={ep.isWatched ? "secondary" : "outline"}
                        class="h-12 justify-start px-4 w-full relative group overflow-hidden border-border/40 shadow-sm hover:border-primary/50 bg-card hover:bg-muted/50 transition-colors"
                >
                    <div class="flex items-center gap-3 z-10 w-full">
                        <span class="text-lg font-black text-muted-foreground/50 group-hover:text-primary transition-colors w-6">
                            {ep.number}
                        </span>
                        <span class="font-semibold text-sm flex-1 text-left line-clamp-1">{ep.title}</span>
                    </div>
                    {#if ep.isWatched}
                        <div class="absolute bottom-0 left-0 h-1 bg-primary/40 w-full"></div>
                    {/if}
                </Button>
            {/each}
        </div>
    {/if}

    {#if displayEpisodes.length > pageSize}
        <div class="pt-6 flex justify-center w-full">
            <Pagination.Root count={displayEpisodes.length} perPage={pageSize} bind:page={currentPage}>
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
</div>