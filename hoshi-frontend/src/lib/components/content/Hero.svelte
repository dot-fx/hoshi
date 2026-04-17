<script lang="ts">
    import type {FullContent} from '@/api/content/types';
    import { primaryMetadata } from '@/api/content/types';
    import { Button } from '$lib/components/ui/button';
    import { Play, Plus, Check } from 'lucide-svelte';
    import { Spinner } from "$lib/components/ui/spinner";
    import { fade, fly } from 'svelte/transition';
    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { listApi } from '@/api/list/list';
    import { i18n } from "$lib/i18n/index.svelte";
    import { appConfig } from '@/stores/config.svelte.js';

    let { items = [], item = null, source = 'anilist', animate = true } = $props();

    let displayItems = $derived(items.length > 0 ? items : (item ? [item] : []));
    let currentIndex = $state(0);
    let timer: ReturnType<typeof setInterval>;
    const DURATION = 8000;

    let showListModal = $state(false);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);

    let currentItem = $derived(displayItems[currentIndex]);
    let meta = $derived(currentItem ? primaryMetadata(currentItem) : undefined);

    let cid = $derived(currentItem?.content.cid);
    let anilistMapping = $derived(
        currentItem?.trackerMappings.find(m => m.trackerName === source)
    );
    let displayTitle = $derived(
        meta ? (meta.titleI18n?.[appConfig.data?.ui?.titleLanguage || 'romaji'] || meta.title) : ''
    );
    let synopsis = $derived(meta?.synopsis);
    let formattedScore = $derived(meta?.rating ? Math.round(meta.rating * (meta.rating <= 10 ? 10 : 1)) : null);
    let trailerId = $derived(getYoutubeId(meta?.trailerUrl));
    let href = $derived(`/c/${cid}`);

    $effect(() => {
        if (!cid) return;
        const t = setTimeout(() => checkListStatus(cid), 500);
        return () => clearTimeout(t);
    });

    async function checkListStatus(cid: string) {
        isEntryLoading = true;
        try {
            const res = await listApi.getEntry(cid);
            hasEntry = res.found;
        } catch {
            hasEntry = false;
        } finally {
            isEntryLoading = false;
        }
    }

    function getYoutubeId(url: string | null | undefined): string | null {
        if (!url) return null;
        const regExp = /^.*((youtu.be\/)|(v\/)|(\/u\/\w\/)|(embed\/)|(watch\?))\??v?=?([^#&?]*).*/;
        const match = url.match(regExp);
        return (match && match[7].length === 11) ? match[7] : null;
    }

    const startTimer = () => {
        if (displayItems.length <= 1) return;
        clearInterval(timer);
        timer = setInterval(() => {
            currentIndex = (currentIndex + 1) % displayItems.length;
        }, DURATION);
    };

    const pauseTimer = () => clearInterval(timer);

    const resumeTimer = () => {
        if (displayItems.length > 1) startTimer();
    };

    const setSlide = (index: number) => {
        currentIndex = index;
        startTimer();
    };

    $effect(() => {
        if (displayItems.length > 1) startTimer();
        return () => clearInterval(timer);
    });

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        return i18n.t(`card.${type.toUpperCase()}`) || type;
    };
</script>

{#if currentItem && meta}
    <div
            class="relative w-full h-[70vh] md:h-[85vh] min-h-[500px] overflow-hidden bg-background"
            onmouseenter={pauseTimer}
            onmouseleave={resumeTimer}
            role="region"
            aria-roledescription="carousel"
    >

        {#key cid}
            <div
                    class="absolute inset-0 w-full h-full"
                    in:fade={{ duration: animate ? 900 : 0 }}
                    out:fade={{ duration: animate ? 700 : 0 }}
            >
                {#if trailerId}
                    <div class="absolute inset-0 w-full h-full pointer-events-none overflow-hidden flex items-center justify-center opacity-60">
                        <iframe
                                src="https://www.youtube.com/embed/{trailerId}?autoplay=1&mute=1&controls=0&loop=1&playlist={trailerId}&enablejsapi=1&rel=0&modestbranding=1"
                                title="Trailer"
                                class="w-[110vw] h-[110vh] min-w-[1920px] min-h-[1080px] object-cover pointer-events-none"
                                frameborder="0"
                                allow="autoplay; fullscreen; picture-in-picture"
                        ></iframe>
                    </div>
                {:else if meta.bannerImage}
                    <img src={meta.bannerImage} alt={displayTitle} class="w-full h-full object-cover object-center opacity-50" />
                {:else if meta.coverImage}
                    <img src={meta.coverImage} alt={displayTitle} class="w-full h-full object-cover object-center opacity-30 blur-lg scale-110" />
                {/if}

                <div class="absolute inset-0 z-10 bg-[linear-gradient(to_top,var(--color-background),color-mix(in_srgb,var(--color-background)_30%,transparent),transparent),linear-gradient(to_right,var(--color-background),color-mix(in_srgb,var(--color-background)_10%,transparent),transparent)]"></div>

                <div class="absolute inset-0 z-20 w-full h-full max-w-[2000px] mx-auto px-4 md:px-12 lg:pl-32 flex flex-col justify-end pb-16 md:pb-24 pt-40 pointer-events-none">
                    <div class="max-w-3xl space-y-4 md:space-y-6 pointer-events-auto">

                        <h1
                                class="font-black text-foreground tracking-tight drop-shadow-2xl text-3xl md:text-4xl lg:text-5xl leading-tight line-clamp-2 md:line-clamp-3"
                                in:fly={{ y: animate ? 40 : 0, duration: animate ? 700 : 0, delay: animate ? 100 : 0 }}
                        >
                            {displayTitle}
                        </h1>

                        <div
                                class="flex flex-wrap items-center gap-3 text-xs md:text-sm font-bold drop-shadow-md text-foreground"
                                in:fly={{ y: animate ? 40 : 0, duration: animate ? 700 : 0, delay: animate ? 250 : 0 }}
                        >
                            {#if meta.subtype}
                                <span class="bg-secondary text-secondary-foreground px-2.5 py-1 rounded-md uppercase tracking-wider border border-border/50">
                                    {formatType(meta.subtype)}
                                </span>
                            {/if}
                            {#if formattedScore}
                                <span class="text-green-500 font-black">{formattedScore}% {i18n.t('home.hero.rating')}</span>
                            {/if}
                            {#if meta.releaseDate}
                                <span class="text-muted-foreground">{meta.releaseDate.split('-')[0]}</span>
                            {/if}
                            {#if meta.epsOrChapters}
                                <span class="text-muted-foreground">
                                    • {currentItem.content.contentType === 'anime' ? i18n.t('home.hero.eps', {count: meta.epsOrChapters}) : i18n.t('home.hero.chapters', {count: meta.epsOrChapters})}
                                </span>
                            {/if}
                        </div>

                        <div
                                class="text-muted-foreground text-sm md:text-base drop-shadow-lg font-medium leading-relaxed max-w-2xl line-clamp-3 md:line-clamp-4"
                                in:fly={{ y: 30, duration: 700, delay: 400 }}
                        >
                            {@html synopsis?.replace(/<[^>]*>?/gm, '') || i18n.t('home.hero.no_synopsis')}
                        </div>

                        <div
                                class="flex flex-wrap items-center gap-3 pt-4"
                                in:fly={{ y: animate ? 40 : 0, duration: animate ? 700 : 0, delay: animate ? 550 : 0 }}
                        >
                            <a
                                    {href}
                                    class="bg-primary hover:bg-primary/90 text-primary-foreground font-bold px-6 md:px-8 py-3 rounded-full flex items-center gap-2.5 transition-all active:scale-95 shadow-lg border border-transparent"
                            >
                                <Play class="w-5 h-5 fill-current" />
                                {currentItem.content.contentType === 'anime' ? i18n.t('home.hero.watch') : i18n.t('home.hero.read')}
                            </a>

                            <Button
                                    variant="secondary"
                                    class="w-12 h-12 rounded-full p-0 flex items-center justify-center transition-all shadow-lg border border-border/50"
                                    onclick={() => showListModal = true}
                                    disabled={isEntryLoading}
                            >
                                {#if isEntryLoading}
                                    <Spinner class="w-5 h-5" />
                                {:else if hasEntry}
                                    <Check class="w-5 h-5 text-green-500" />
                                {:else}
                                    <Plus class="w-5 h-5" />
                                {/if}
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        {/key}

        {#if displayItems.length > 1}
            <div class="absolute bottom-6 right-6 md:right-12 z-30 flex gap-2">
                {#each displayItems as _, i}
                    <button
                            aria-label={`Ir a diapositiva ${i + 1}`}
                            class="h-1.5 rounded-full transition-all duration-300 shadow-sm {i === currentIndex
                            ? 'w-8 bg-primary scale-110'
                            : 'w-2 bg-primary/40 hover:bg-primary/80'}"
                            onclick={() => setSlide(i)}
                    ></button>
                {/each}
            </div>
        {/if}
    </div>

    <ListEditor
            bind:open={showListModal}
            cid={cid}
            title={displayTitle}
            contentType={currentItem.content.contentType}
            coverImage={meta.coverImage ?? undefined}
    />
{/if}