<script lang="ts">
    import type { NormalizedCard } from '@/utils/normalize';
    import { getCardTitle, getCardShouldBlur, getCardTrailerUrl } from '@/utils/normalize';
    import { Button } from '@/components/ui/button';
    import { Play, Plus, Check } from 'lucide-svelte';
    import { fade, fly } from 'svelte/transition';
    import { i18n } from "@/stores/i18n.svelte.js";
    import { listStore } from '@/app/list.svelte.js';
    import ListEditorButton from "@/components/ListEditorButton.svelte";

    const YOUTUBE_REGEXP = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;

    let { items = [], animate = true }: { items: NormalizedCard[], animate?: boolean } = $props();

    let currentIndex = $state(0);
    let timer: ReturnType<typeof setInterval>;
    const DURATION = 8000;

    let iframeReady = $state(false);

    let currentItem = $derived(items[currentIndex]);

    let title      = $derived(currentItem ? getCardTitle(currentItem) : "");
    let trailerUrl = $derived(currentItem ? getCardTrailerUrl(currentItem) : null);
    let shouldBlur = $derived(currentItem ? getCardShouldBlur(currentItem) : false);

    function getYoutubeId(url: string | null | undefined): string | null {
        if (!url) return null;
        const match = url.match(YOUTUBE_REGEXP);
        return (match && match[2].length === 11) ? match[2] : null;
    }

    let trailerId    = $derived(getYoutubeId(trailerUrl));
    let thumbnailSrc = $derived(trailerId ? `https://i.ytimg.com/vi/${trailerId}/maxresdefault.jpg` : null);

    const startTimer = () => {
        if (items.length <= 1) return;
        clearInterval(timer);
        timer = setInterval(() => {
            currentIndex = (currentIndex + 1) % items.length;
        }, DURATION);
    };

    const pauseTimer  = () => clearInterval(timer);
    const resumeTimer = () => { if (items.length > 1) startTimer(); };
    const setSlide    = (index: number) => { currentIndex = index; startTimer(); };

    $effect(() => {
        if (items.length > 1) startTimer();
        return () => clearInterval(timer);
    });

    $effect(() => {
        currentIndex;
        iframeReady = false;
    });

    function onIframeLoad() {
        setTimeout(() => (iframeReady = true), 300);
    }
</script>

{#if currentItem}
    {@const cid = currentItem.href.replace('/c/', '')}
    {@const hasEntry = listStore.hasCid(cid)}

    <div
            class="relative w-full h-[70vh] md:h-[85vh] min-h-[500px] overflow-hidden bg-background"
            onmouseenter={pauseTimer}
            onmouseleave={resumeTimer}
            role="region"
            aria-roledescription="carousel"
    >
        {#key currentItem.href}
            <div
                    class="absolute inset-0 w-full h-full"
                    in:fade={{ duration: animate ? 900 : 0 }}
                    out:fade={{ duration: animate ? 700 : 0 }}
            >
                {#if trailerId}
                    <div class="absolute inset-0 w-full h-full pointer-events-none overflow-hidden flex items-center justify-center opacity-60">
                        <img
                                src={thumbnailSrc}
                                alt=""
                                class="absolute inset-0 w-full h-full object-cover transition-opacity duration-500 {shouldBlur ? 'blur-xl scale-110' : ''} {iframeReady ? 'opacity-0' : 'opacity-100'}"
                        />
                        <iframe
                                src="https://www.youtube.com/embed/{trailerId}?autoplay=1&mute=1&controls=0&loop=1&playlist={trailerId}&enablejsapi=1&rel=0&modestbranding=1"
                                title="Trailer"
                                class="w-[110vw] h-[110vh] min-w-[1920px] min-h-[1080px] object-cover pointer-events-none {shouldBlur ? 'blur-xl scale-110' : ''}"
                                frameborder="0"
                                allow="autoplay; fullscreen; picture-in-picture"
                                onload={onIframeLoad}
                        ></iframe>
                    </div>
                {:else if currentItem.bannerImage}
                    <img
                            src={currentItem.bannerImage}
                            alt={title}
                            class="w-full h-full object-cover object-center opacity-50 {shouldBlur ? 'blur-xl scale-110' : ''}"
                    />
                {:else if currentItem.cover}
                    <img
                            src={currentItem.cover}
                            alt={title}
                            class="w-full h-full object-cover object-center opacity-30 blur-lg scale-110"
                    />
                {/if}

                <div class="absolute inset-0 z-10 bg-[linear-gradient(to_top,var(--color-background),color-mix(in_srgb,var(--color-background)_30%,transparent),transparent),linear-gradient(to_right,var(--color-background),color-mix(in_srgb,var(--color-background)_10%,transparent),transparent)]"></div>

                <div class="absolute inset-0 z-20 w-full h-full max-w-[2000px] mx-auto px-4 md:px-12 lg:pl-32 flex flex-col justify-end pb-16 md:pb-24 pt-40 pointer-events-none">
                    <div class="max-w-3xl space-y-4 md:space-y-6 pointer-events-auto">

                        <h1
                                class="font-black text-foreground tracking-tight drop-shadow-2xl text-3xl md:text-4xl lg:text-5xl leading-tight line-clamp-2 md:line-clamp-3"
                                in:fly={{ y: animate ? 40 : 0, duration: animate ? 700 : 0, delay: animate ? 100 : 0 }}
                        >
                            {title}
                        </h1>

                        <div
                                class="flex flex-wrap items-center gap-3 text-xs md:text-sm font-bold drop-shadow-md text-foreground"
                                in:fly={{ y: animate ? 40 : 0, duration: animate ? 700 : 0, delay: animate ? 250 : 0 }}
                        >
                            {#if currentItem.contentTypeLabel}
                                <span class="bg-secondary text-secondary-foreground px-2.5 py-1 rounded-md uppercase tracking-wider border border-border/50">
                                    {currentItem.contentTypeLabel}
                                </span>
                            {/if}
                            {#if currentItem.score}
                                <span class="text-green-500 font-black">{currentItem.score}% {i18n.t('home.hero.rating')}</span>
                            {/if}
                            {#if currentItem.year}
                                <span class="text-muted-foreground">{currentItem.year}</span>
                            {/if}
                            {#if currentItem.episodeCount}
                                <span class="text-muted-foreground">
                                    • {currentItem.contentType === 'anime' ? i18n.t('home.hero.eps', {count: currentItem.episodeCount}) : i18n.t('home.hero.chapters', {count: currentItem.episodeCount})}
                                </span>
                            {/if}
                        </div>

                        <div
                                class="text-muted-foreground text-sm md:text-base drop-shadow-lg font-medium leading-relaxed max-w-2xl line-clamp-3 md:line-clamp-4"
                                in:fly={{ y: 30, duration: 700, delay: 400 }}
                        >
                            {currentItem.synopsis || i18n.t('home.hero.no_synopsis')}
                        </div>

                        <div
                                class="flex flex-wrap items-center gap-3 pt-4"
                                in:fly={{ y: animate ? 40 : 0, duration: animate ? 700 : 0, delay: animate ? 550 : 0 }}
                        >
                            <a
                                    href={currentItem.href}
                                    class="bg-primary hover:bg-primary/90 text-primary-foreground font-bold px-6 md:px-8 py-3 rounded-sm flex items-center gap-2.5 transition-all active:scale-95 shadow-lg border border-transparent"
                            >
                                <Play class="w-5 h-5 fill-current" />
                                {currentItem.contentType === 'anime' ? i18n.t('home.hero.watch') : i18n.t('home.hero.read')}
                            </a>

                            <ListEditorButton
                                    cid={cid}
                                    title={title}
                                    contentType={currentItem.contentType}
                                    coverImage={currentItem.cover ?? undefined}
                                    size="icon"
                                    class="h-12 w-12"
                            />
                        </div>
                    </div>
                </div>
            </div>
        {/key}

        {#if items.length > 1}
            <div class="absolute bottom-6 right-6 md:right-12 z-30 flex gap-2">
                {#each items as _, i}
                    <button
                            aria-label={`Ir a diapositiva ${i + 1}`}
                            class="h-1.5 rounded-sm transition-all duration-300 shadow-sm {i === currentIndex
                            ? 'w-8 bg-primary scale-110'
                            : 'w-2 bg-primary/40 hover:bg-primary/80'}"
                            onclick={() => setSlide(i)}
                    ></button>
                {/each}
            </div>
        {/if}
    </div>
{/if}