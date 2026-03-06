<script lang="ts">
    import type { CoreMetadata } from '@/api/content/types';
    import { Button } from '$lib/components/ui/button';
    import { Badge } from "$lib/components/ui/badge";
    import { Play, Plus, Check, Loader2, Film } from 'lucide-svelte';
    import { fade, fly } from 'svelte/transition';
    import ListEditorModal from '$lib/components/ListEditorModal.svelte';
    import { listApi } from '@/api/list/list';
    import { i18n } from "$lib/i18n/index.svelte"; // <-- Importar i18n

    let {
        items = [],
        item = null
    }: {
        items?: CoreMetadata[];
        item?: CoreMetadata | null;
    } = $props();

    let displayItems = $derived(items.length > 0 ? items : (item ? [item] : []));
    let currentIndex = $state(0);
    let timer: ReturnType<typeof setInterval>;
    const DURATION = 8000;
    let isExpanded = $state(false);
    let showListModal = $state(false);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);

    let currentItem = $derived(displayItems[currentIndex]);
    let trailerId = $derived(getYoutubeId(currentItem?.trailerUrl));
    let synopsis = $derived(currentItem?.synopsis);

    $effect(() => {
        if (currentItem?.cid) {
            checkListStatus(currentItem.cid);
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
        if (displayItems.length <= 1) return;
        clearInterval(timer);
        timer = setInterval(() => {
            currentIndex = (currentIndex + 1) % displayItems.length;
            isExpanded = false;
        }, DURATION);
    };

    const setSlide = (index: number) => {
        currentIndex = index;
        isExpanded = false;
        startTimer();
    };

    $effect(() => {
        if (displayItems.length > 1) {
            startTimer();
        }
        return () => clearInterval(timer);
    });

    const toggleReadMore = () => {
        isExpanded = !isExpanded;
        if (isExpanded) clearInterval(timer);
        else startTimer();
    };
</script>

{#if currentItem}
    <div class="relative w-full overflow-hidden bg-background group {displayItems.length > 1 ? 'h-[85vh]' : ''}">
        {#key currentItem.cid}
            <div class="absolute inset-0 w-full h-full" in:fade={{ duration: 1000 }} out:fade={{ duration: 1000 }}>
                <div class="absolute inset-0 z-0 bg-black">
                    {#if trailerId}
                        <div class="absolute inset-0 w-[300%] h-[300%] -top-full -left-full pointer-events-none opacity-40">
                            <iframe
                                    src={`https://www.youtube.com/embed/${trailerId}?autoplay=1&mute=1&playsinline=1&controls=0&loop=1&playlist=${trailerId}&showinfo=0&rel=0&iv_load_policy=3&disablekb=1&modestbranding=1`}
                                    title="Hero Trailer" class="w-full h-full pointer-events-none"
                                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                            ></iframe>
                        </div>
                    {:else if currentItem.bannerImage}
                        <img src={currentItem.bannerImage} alt={currentItem.title} class="w-full h-full object-cover object-[center_20%] opacity-30" />
                    {:else if currentItem.coverImage}
                        <img src={currentItem.coverImage} alt={currentItem.title} class="w-full h-full object-cover object-[center_20%] opacity-20 blur-md" />
                    {/if}

                    <div class="absolute inset-0 bg-linear-to-t from-background via-background/60 to-transparent"></div>
                    <div class="absolute inset-x-0 bottom-0 h-40 bg-linear-to-t from-background to-transparent"></div>
                    <div class="absolute inset-0 bg-linear-to-r from-background via-background/40 to-transparent"></div>
                </div>
            </div>

            <div class="relative z-10 w-full px-4 md:px-12 {displayItems.length > 1 ? 'h-full flex items-end md:items-center pb-20 md:pb-0' : 'max-w-[1400px] mx-auto pt-32 md:pt-40 pb-16'}">
                <div class="flex flex-col md:flex-row gap-6 md:gap-10 items-start w-full">

                    {#if displayItems.length === 1}
                        <div class="w-40 md:w-64 shrink-0 rounded-xl overflow-hidden shadow-2xl shadow-black/60 border border-border/30" in:fly={{ y: 20, duration: 800 }}>
                            {#if currentItem.coverImage}
                                <img src={currentItem.coverImage} alt={currentItem.title} class="w-full h-auto object-cover aspect-2/3" />
                            {:else}
                                <div class="w-full aspect-2/3 bg-muted flex items-center justify-center">
                                    <Film class="h-12 w-12 text-muted-foreground" />
                                </div>
                            {/if}
                        </div>
                    {/if}

                    <div class="max-w-4xl flex-col flex-1 {displayItems.length === 1 ? 'pt-2 md:pt-6' : 'space-y-6'}">
                        <h1 class="font-black text-foreground tracking-tight drop-shadow-xl balance-text
                            {displayItems.length > 1 ? 'text-4xl md:text-5xl lg:text-7xl leading-tight line-clamp-2' : 'text-3xl md:text-5xl lg:text-6xl'}"
                            in:fly={{ y: 20, duration: 800, delay: 200 }}>
                            {currentItem.title}
                        </h1>

                        {#if displayItems.length === 1 && currentItem.altTitles && currentItem.altTitles.length > 0}
                            <p class="text-muted-foreground mt-3 text-sm md:text-base font-medium" in:fly={{ y: 20, duration: 800, delay: 250 }}>
                                {currentItem.altTitles.slice(0, 2).join(" • ")}
                            </p>
                        {/if}

                        <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm md:text-base font-medium text-foreground/80 drop-shadow-md {displayItems.length === 1 ? 'mt-5' : ''}" in:fly={{ y: 20, duration: 800, delay: 300 }}>
                            {#if displayItems.length === 1 && currentItem.contentType}
                                <Badge variant="secondary" class="font-bold uppercase tracking-wider text-xs bg-secondary/80 backdrop-blur-md">
                                    {i18n.t(currentItem.contentType)}
                                    {#if currentItem.subtype && currentItem.subtype.toLowerCase() !== currentItem.contentType.toLowerCase()}
                                        <span class="opacity-70 ml-1">• {currentItem.subtype.replace('_', ' ')}</span>
                                    {/if}
                                </Badge>
                            {/if}

                            {#if currentItem.status && displayItems.length === 1}
                                <Badge variant={currentItem.status.toLowerCase() === 'ongoing' ? 'default' : 'secondary'} class="font-semibold capitalize">
                                    {i18n.t(currentItem.status.toLowerCase()) || currentItem.status}
                                </Badge>
                            {/if}

                            {#if currentItem.rating}
                                {#if displayItems.length === 1}
                                    <Badge variant="outline" class="bg-background/50 backdrop-blur-sm border-primary/30 text-primary font-bold">
                                        ★ {currentItem.rating} / 10
                                    </Badge>
                                {:else}
                                    <span class="text-green-500 font-bold">{(currentItem.rating * 10).toFixed(0)}% {i18n.t('rating_label')}</span>
                                {/if}
                            {/if}

                            {#if currentItem.releaseDate && displayItems.length > 1}
                                <span class="text-muted-foreground">|</span>
                                <span>{currentItem.releaseDate.split('-')[0]}</span>
                            {/if}
                            {#if displayItems.length > 1}
                                <span class="text-muted-foreground">|</span>
                                <span class="capitalize">{currentItem.subtype || i18n.t(currentItem.contentType)}</span>
                                <span class="border border-border rounded px-1.5 text-xs text-muted-foreground ml-2">HD</span>
                            {/if}
                        </div>

                        <div class="max-w-2xl relative z-20 {displayItems.length === 1 ? 'mt-6' : ''}" in:fly={{ y: 20, duration: 800, delay: 400 }}>
                            <div class="text-foreground/90 text-sm md:text-lg drop-shadow-lg font-normal leading-relaxed transition-all duration-300 {isExpanded ? 'max-h-[30vh] overflow-y-auto bg-background/80 p-4 rounded-xl backdrop-blur-md border border-border/50' : (displayItems.length === 1 ? 'line-clamp-4' : 'line-clamp-3')} pointer-events-auto">
                                {@html synopsis || i18n.t('no_synopsis')}
                            </div>
                            {#if synopsis && synopsis.length > 150}
                                <button class="text-primary font-bold hover:underline mt-2 text-sm drop-shadow-md transition-colors pointer-events-auto cursor-pointer" onclick={toggleReadMore}>
                                    {isExpanded ? i18n.t('show_less') : i18n.t('read_more')}
                                </button>
                            {/if}
                        </div>

                        <div class="flex flex-wrap items-center gap-4 {displayItems.length === 1 ? 'mt-8' : 'pt-4'}" in:fly={{ y: 20, duration: 800, delay: 600 }}>
                            <Button size="lg" class="bg-primary text-primary-foreground hover:bg-primary/90 font-bold px-8 h-14 text-lg rounded gap-3 shadow-xl transition-transform active:scale-95">
                                <Play class="w-6 h-6 fill-primary-foreground" />
                                {currentItem.contentType === 'anime' ? i18n.t('watch_now') : i18n.t('read_now')}
                            </Button>

                            <Button variant="secondary" size="lg" class="bg-secondary/60 hover:bg-secondary/80 text-secondary-foreground h-14 px-6 text-lg rounded gap-3 backdrop-blur-md shadow-xl transition-transform active:scale-95" onclick={() => showListModal = true} disabled={isEntryLoading}>
                                {#if isEntryLoading} <Loader2 class="w-6 h-6 animate-spin" /> {i18n.t('loading')}
                                {:else if hasEntry} <Check class="w-6 h-6 text-green-500" /> {i18n.t('in_my_list')}
                                {:else} <Plus class="w-6 h-6" /> {i18n.t('my_list')} {/if}
                            </Button>

                            {#if displayItems.length === 1 && currentItem.trailerUrl && !trailerId}
                                <Button variant="secondary" size="lg" class="bg-secondary/60 hover:bg-secondary/80 text-secondary-foreground h-14 px-6 text-lg rounded gap-3 backdrop-blur-md shadow-xl transition-transform active:scale-95" href={currentItem.trailerUrl} target="_blank">
                                    <Film class="w-6 h-6" /> {i18n.t('trailer')}
                                </Button>
                            {/if}
                        </div>
                    </div>
                </div>
            </div>
        {/key}

        {#if displayItems.length > 1}
            <div class="absolute bottom-8 right-8 z-30 flex gap-2">
                {#each displayItems as _, i}
                    <button class="h-2 rounded-full transition-all duration-300 {i === currentIndex ? 'w-8 bg-primary' : 'w-2 bg-primary/40 hover:bg-primary/60'}" onclick={() => setSlide(i)} aria-label={`${i18n.t('go_to_slide')} ${i + 1}`}></button>
                {/each}
            </div>
        {/if}
    </div>

    <ListEditorModal
            bind:open={showListModal}
            cid={currentItem.cid}
            title={currentItem.title}
            contentType={currentItem.contentType || 'anime'}
            coverImage={currentItem.coverImage ?? undefined}
    />
{/if}