<script lang="ts">
    import type { CoreMetadata } from '@/api/content/types';
    import { Button } from '$lib/components/ui/button';
    import { Play, Plus, Check, Loader2 } from 'lucide-svelte';
    import { fade, fly } from 'svelte/transition';
    import ListEditorModal from '$lib/components/ListEditorModal.svelte';
    import { listApi } from '@/api/list/list';

    let { heroes = [] }: { heroes: CoreMetadata[] } = $props();

    let currentIndex = $state(0);
    let timer: ReturnType<typeof setInterval>;
    const DURATION = 8000;
    let isExpanded = $state(false);

    let showListModal = $state(false);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);

    let currentHero = $derived(heroes[currentIndex]);
    let trailerId = $derived(getYoutubeId(currentHero?.trailerUrl));
    let synopsis = $derived(currentHero?.synopsis);

    $effect(() => {
        if (currentHero?.cid) {
            checkListStatus(currentHero.cid);
        }
    });

    async function checkListStatus(cid: string) {
        isEntryLoading = true;
        try {
            const res = await listApi.getEntry(cid);
            hasEntry = res.found;
        } catch (err) {
            console.error("Error checking list status:", err);
            hasEntry = false;
        } finally {
            isEntryLoading = false;
        }
    }

    function getYoutubeId(url: string | null | undefined): string | null {
        if (!url) return null;
        const regExp = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;
        const match = url.match(regExp);
        return (match && match[2].length === 11) ? match[2] : null;
    }

    const startTimer = () => {
        clearInterval(timer);
        timer = setInterval(() => {
            if (heroes.length > 0) {
                currentIndex = (currentIndex + 1) % heroes.length;
                isExpanded = false;
            }
        }, DURATION);
    };

    const setSlide = (index: number) => {
        currentIndex = index;
        isExpanded = false;
        startTimer();
    };

    $effect(() => {
        if (heroes.length > 1) {
            startTimer();
        }
        return () => clearInterval(timer);
    });

    const toggleReadMore = () => {
        isExpanded = !isExpanded;
        if (isExpanded) {
            clearInterval(timer);
        } else {
            startTimer();
        }
    };
</script>

{#if currentHero}
    <div class="relative w-full h-[85vh] overflow-hidden bg-background group">
        {#key currentHero.cid}
            <div
                    class="absolute inset-0 w-full h-full"
                    in:fade={{ duration: 1000 }}
                    out:fade={{ duration: 1000 }}
            >
                <div class="absolute inset-0 z-0">
                    {#if trailerId}
                        <div class="absolute inset-0 w-[300%] h-[300%] -top-full -left-full pointer-events-none opacity-60">
                            <iframe
                                    src={`https://www.youtube.com/embed/${trailerId}?autoplay=1&mute=1&playsinline=1&controls=0&loop=1&playlist=${trailerId}&showinfo=0&rel=0&iv_load_policy=3&disablekb=1&modestbranding=1`}
                                    title="Hero Trailer"
                                    class="w-full h-full pointer-events-none"
                                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                            ></iframe>
                        </div>
                    {:else if currentHero.bannerImage}
                        <img
                                src={currentHero.bannerImage}
                                alt={currentHero.title}
                                class="w-full h-full object-cover object-[center_20%] opacity-50"
                        />
                    {/if}
                    <div class="absolute inset-0 bg-linear-to-t from-background via-background/60 to-transparent"></div>
                    <div class="absolute inset-0 bg-linear-to-r from-background via-background/40 to-transparent"></div>
                </div>

                <div class="absolute inset-0 z-10 flex items-end md:items-center pb-20 md:pb-0">
                    <div class="w-full px-4 md:px-12">
                        <div class="max-w-4xl space-y-6">
                            <h1 class="text-4xl md:text-5xl lg:text-7xl font-black text-foreground leading-tight tracking-tight drop-shadow-xl line-clamp-2"
                                in:fly={{ y: 20, duration: 800, delay: 200 }}
                                title={currentHero.title}> {currentHero.title}
                            </h1>

                            <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm md:text-base font-medium text-foreground/80 drop-shadow-md"
                                 in:fly={{ y: 20, duration: 800, delay: 300 }}>
                                {#if currentHero.rating}
                                    <span class="text-green-500 font-bold">{(currentHero.rating * 10).toFixed(0)}% Rating</span>
                                {/if}
                                {#if currentHero.releaseDate}
                                    <span class="text-muted-foreground">|</span>
                                    <span>{currentHero.releaseDate.split('-')[0]}</span>
                                {/if}
                                <span class="text-muted-foreground">|</span>
                                <span class="capitalize">{currentHero.subtype || currentHero.contentType}</span>
                                <span class="border border-border rounded px-1.5 text-xs text-muted-foreground ml-2">HD</span>
                            </div>

                            <div class="max-w-2xl relative z-20" in:fly={{ y: 20, duration: 800, delay: 400 }}>
                                <div class="text-foreground/90 text-sm md:text-lg drop-shadow-lg font-normal leading-relaxed transition-all duration-300 {isExpanded ? 'max-h-[30vh] overflow-y-auto bg-background/80 p-4 rounded-xl backdrop-blur-md border border-border/50' : 'line-clamp-3'} pointer-events-auto">
                                    {#if synopsis}
                                        {@html synopsis}
                                    {:else}
                                        No synopsis available.
                                    {/if}
                                </div>
                                {#if synopsis && synopsis.length > 150}
                                    <button
                                            class="text-primary font-bold hover:underline mt-2 text-sm drop-shadow-md transition-colors pointer-events-auto cursor-pointer"
                                            onclick={toggleReadMore}
                                    >
                                        {isExpanded ? 'Show less' : 'Read more'}
                                    </button>
                                {/if}
                            </div>

                            {#if currentHero.genres && currentHero.genres.length > 0}
                                <div class="flex gap-2 pt-1" in:fly={{ y: 20, duration: 800, delay: 500 }}>
                                    {#each currentHero.genres as genre}
                                    <span class="text-xs md:text-sm text-muted-foreground hover:text-foreground transition-colors cursor-default border-r border-border pr-2 last:border-0">
                                        {genre}
                                    </span>
                                    {/each}
                                </div>
                            {/if}

                            <div class="flex flex-wrap items-center gap-4 pt-4" in:fly={{ y: 20, duration: 800, delay: 600 }}>
                                <Button size="lg" class="bg-primary text-primary-foreground hover:bg-primary/90 font-bold px-8 h-14 text-lg rounded gap-3 shadow-xl transition-transform active:scale-95">
                                    <Play class="w-6 h-6 fill-primary-foreground" />
                                    Play
                                </Button>

                                <Button
                                        variant="secondary"
                                        size="lg"
                                        class="bg-secondary/60 hover:bg-secondary/80 text-secondary-foreground h-14 px-6 text-lg rounded gap-3 backdrop-blur-md shadow-xl transition-transform active:scale-95"
                                        onclick={() => showListModal = true}
                                        disabled={isEntryLoading}
                                >
                                    {#if isEntryLoading}
                                        <Loader2 class="w-6 h-6 animate-spin" />
                                        Loading...
                                    {:else if hasEntry}
                                        <Check class="w-6 h-6 text-green-500" />
                                        In My List
                                    {:else}
                                        <Plus class="w-6 h-6" />
                                        My List
                                    {/if}
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        {/key}

        {#if heroes.length > 1}
            <div class="absolute bottom-8 right-8 z-30 flex gap-2">
                {#each heroes as _, i}
                    <button
                            class="h-2 rounded-full transition-all duration-300 {i === currentIndex ? 'w-8 bg-primary' : 'w-2 bg-primary/40 hover:bg-primary/60'}"
                            onclick={() => setSlide(i)}
                            aria-label={`Go to slide ${i + 1}`}
                    ></button>
                {/each}
            </div>
        {/if}
    </div>
{/if}

{#if currentHero}
    <ListEditorModal
            bind:open={showListModal}
            cid={currentHero.cid}
            title={currentHero.title}
            contentType={currentHero.contentType}
            coverImage={currentHero.coverImage ?? undefined}
    />
{/if}