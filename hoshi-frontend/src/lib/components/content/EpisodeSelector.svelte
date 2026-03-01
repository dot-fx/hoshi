<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { PlayCircle } from "lucide-svelte";
    import * as Select from "$lib/components/ui/select";
    import * as Carousel from "$lib/components/ui/carousel";
    import type { ContentUnit } from "$lib/api/content/types";

    let { cid, extensions, epsOrChapters, contentUnits = [] }: {
        cid: string,
        extensions: any[],
        epsOrChapters?: number | null,
        contentUnits?: ContentUnit[]
    } = $props();

    let selectedSource = $state("");

    $effect(() => {
        if (!selectedSource && extensions && extensions.length > 0) {
            selectedSource = extensions[0].extensionId;
        }
    });

    const displayEpisodes = $derived.by(() => {
        if (contentUnits && contentUnits.length > 0) {
            const regularEpisodes = contentUnits
                .filter(u => u.contentType === 'episode')
                .sort((a, b) => a.unitNumber - b.unitNumber);

            if (regularEpisodes.length > 0) {
                return regularEpisodes.map(u => ({
                    number: u.unitNumber,
                    title: u.title || `Episode ${u.unitNumber}`,
                    description: u.description,
                    thumbnail: u.thumbnailUrl ? u.thumbnailUrl.replace('_m.', '_w.') : null,
                    isWatched: false
                }));
            }
        }

        const totalEpisodes = epsOrChapters && epsOrChapters > 0 ? epsOrChapters : 12;
        return Array.from({ length: totalEpisodes }, (_, i) => ({
            number: i + 1,
            title: `Episode ${i + 1}`,
            description: null,
            thumbnail: null,
            isWatched: false
        }));
    });

    const isRichMode = $derived(displayEpisodes.length > 0 && displayEpisodes[0]?.thumbnail);
</script>

<div class="space-y-6">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <h2 class="text-2xl font-semibold tracking-tight">Episodes</h2>

        {#if extensions.length > 0}
            <div class="w-full sm:w-48">
                <Select.Root type="single" bind:value={selectedSource}>
                    <Select.Trigger class="h-9">
                        {extensions.find(e => e.extensionId === selectedSource)?.extensionName || `Source (${extensions[0].extensionName})`}
                    </Select.Trigger>
                    <Select.Content>
                        {#each extensions as ext}
                            <Select.Item value={ext.extensionId}>{ext.extensionName}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>
        {/if}
    </div>

    {#if isRichMode}
        <div class="w-full">

            <div class="flex flex-col gap-4 sm:hidden max-h-[70vh] overflow-y-auto pr-2 pb-4" style="scrollbar-width: thin;">
                {#each displayEpisodes as ep}
                    <div class="flex flex-col gap-2.5 relative group/ep cursor-pointer rounded-xl p-2 -mx-2 hover:bg-card/60 transition-colors border border-transparent hover:border-border/50">
                        <div class="flex gap-3 sm:gap-4">

                            <div class="relative w-36 shrink-0 aspect-video bg-muted rounded-lg overflow-hidden border border-border/40 shadow-sm">
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

                            <div class="flex flex-col justify-center py-1">
                                <h3 class="font-semibold text-sm leading-snug line-clamp-3 group-hover/ep:text-primary transition-colors">
                                    {ep.title}
                                </h3>
                            </div>
                        </div>

                        {#if ep.description}
                            <p class="text-xs text-muted-foreground line-clamp-3 leading-relaxed mt-1">
                                {ep.description}
                            </p>
                        {/if}
                    </div>
                {/each}
            </div>

            <div class="hidden sm:block relative w-full">
                <Carousel.Root opts={{ align: "start", dragFree: true }} class="w-full relative group/carousel">
                    <Carousel.Content class="-ml-4 flex py-2">
                        {#each displayEpisodes as ep}
                            <Carousel.Item class="pl-4 basis-[100%] sm:basis-[80%] md:basis-[50%] lg:basis-[33.333%] min-w-0 flex-none">
                                <div class="group/card relative flex flex-col h-full overflow-hidden rounded-xl border border-border/50 bg-card/40 text-card-foreground shadow-sm transition-all hover:bg-card/80 hover:border-primary/50 cursor-pointer">

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

                                        <div class="absolute bottom-2 left-2 bg-background/90 backdrop-blur-md px-2 py-0.5 rounded text-xs font-bold shadow-sm">
                                            EP {ep.number}
                                        </div>
                                    </div>

                                    <div class="flex flex-1 flex-col p-4 space-y-1.5 min-h-[100px]">
                                        <h3 class="font-semibold text-sm leading-tight line-clamp-2 group-hover/card:text-primary transition-colors" title={ep.title}>
                                            {ep.title}
                                        </h3>
                                        {#if ep.description}
                                            <p class="text-[13px] text-muted-foreground line-clamp-2 leading-snug mt-auto pt-1">
                                                {ep.description}
                                            </p>
                                        {:else}
                                            <p class="text-[13px] text-muted-foreground/50 italic mt-auto pt-1">No description.</p>
                                        {/if}
                                    </div>

                                    {#if ep.isWatched}
                                        <div class="absolute bottom-0 left-0 h-1 bg-primary/60 w-full"></div>
                                    {/if}
                                </div>
                            </Carousel.Item>
                        {/each}
                    </Carousel.Content>

                    <Carousel.Previous
                            class="absolute left-2 top-1/2 -translate-y-1/2 z-40 h-12 w-12 md:h-14 md:w-14 rounded-full border border-white/10 bg-background/50 backdrop-blur-xl text-foreground shadow-[0_0_20px_rgba(0,0,0,0.5)] opacity-0 group-hover/carousel:opacity-100 transition-all duration-300 hover:scale-110 hover:bg-foreground hover:text-background hidden md:flex items-center justify-center disabled:opacity-0"
                    />

                    <Carousel.Next
                            class="absolute right-2 top-1/2 -translate-y-1/2 z-40 h-12 w-12 md:h-14 md:w-14 rounded-full border border-white/10 bg-background/50 backdrop-blur-xl text-foreground shadow-[0_0_20px_rgba(0,0,0,0.5)] opacity-0 group-hover/carousel:opacity-100 transition-all duration-300 hover:scale-110 hover:bg-foreground hover:text-background hidden md:flex items-center justify-center disabled:opacity-0"
                    />
                </Carousel.Root>
            </div>
        </div>

    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 max-h-[60vh] sm:max-h-none overflow-y-auto sm:overflow-visible pr-2 sm:pr-0" style="scrollbar-width: thin;">
            {#each displayEpisodes as ep}
                <Button
                        variant={ep.isWatched ? "secondary" : "outline"}
                        class="h-14 justify-start px-4 w-full relative group overflow-hidden border-border/50 shadow-sm hover:border-primary/50"
                >
                    <div class="flex items-center gap-4 z-10 w-full">
                        <span class="text-xl font-black text-muted-foreground/40 group-hover:text-primary/70 transition-colors min-w-[24px]">
                            {ep.number}
                        </span>
                        <span class="font-medium flex-1 text-left line-clamp-1">{ep.title}</span>
                        <PlayCircle class="h-5 w-5 opacity-0 group-hover:opacity-100 transition-opacity text-primary flex-shrink-0" />
                    </div>
                    {#if ep.isWatched}
                        <div class="absolute bottom-0 left-0 h-1 bg-primary/40 w-full"></div>
                    {/if}
                </Button>
            {/each}
        </div>
    {/if}
</div>