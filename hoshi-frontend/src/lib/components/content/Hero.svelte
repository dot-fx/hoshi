<script lang="ts">
    import type { ContentWithMappings } from '@/api/content/types';
    import { primaryMetadata } from '@/api/content/types';
    import { Button } from '$lib/components/ui/button';
    import { Play, Plus, Check, Loader2, Info } from 'lucide-svelte';
    import { fade, fly } from 'svelte/transition';
    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { listApi } from '@/api/list/list';
    import { i18n } from "$lib/i18n/index.svelte";

    let {
        items = [],
        item = null
    }: {
        items?: ContentWithMappings[];
        item?: ContentWithMappings | null;
    } = $props();

    let displayItems = $derived(items.length > 0 ? items : (item ? [item] : []));
    let currentIndex = $state(0);
    let timer: ReturnType<typeof setInterval>;
    const DURATION = 8000;

    let showListModal = $state(false);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);

    let currentItem = $derived(displayItems[currentIndex]);
    let meta = $derived(currentItem ? primaryMetadata(currentItem) : undefined);
    let synopsis = $derived(meta?.synopsis);

    let formattedScore = $derived(meta?.rating ? Math.round(meta.rating * 10) : null);
    let trailerId = $derived(getYoutubeId(meta?.trailerUrl));
    let href = $derived(currentItem?.content?.cid ? `/content/${currentItem.content.cid}` : '#');

    $effect(() => {
        if (currentItem?.content?.cid) {
            checkListStatus(currentItem.content.cid);
        }
    });

    async function checkListStatus(cid: string) {
        isEntryLoading = true;
        try {
            const res = await listApi.getEntry(cid);
            hasEntry = res.found;
        } catch (err) {
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
        }, DURATION);
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
        if (type === 'TV') return i18n.t('card.TV');
        const normalized = type.charAt(0).toUpperCase() + type.slice(1).toLowerCase();
        const key = `tags.${normalized}` as any;
        const translated = i18n.t(key);
        return translated === key ? normalized : translated;
    };
</script>

{#if currentItem && meta}
    <div class="relative w-full h-[70vh] md:h-[85vh] min-h-[500px] overflow-hidden bg-background">

        {#key currentItem.content.cid}
            <div class="absolute inset-0 w-full h-full" in:fade={{ duration: 800 }} out:fade={{ duration: 800 }}>
                {#if trailerId}
                    <div class="absolute inset-0 w-full h-full pointer-events-none overflow-hidden flex items-center justify-center opacity-60">
                        <iframe
                                src={`https://www.youtube.com/embed/${trailerId}?autoplay=1&mute=1&playsinline=1&controls=0&loop=1&playlist=${trailerId}&showinfo=0&rel=0&iv_load_policy=3&disablekb=1&modestbranding=1`}
                                title={i18n.t('home.hero.trailer')}
                                class="w-[150vw] h-[150vh] min-w-[1920px] min-h-[1080px] object-cover pointer-events-none"
                                frameborder="0"
                                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                        ></iframe>
                    </div>
                {:else if meta.bannerImage}
                    <img src={meta.bannerImage} alt={meta.title} class="w-full h-full object-cover object-center opacity-50" />
                {:else if meta.coverImage}
                    <img src={meta.coverImage} alt={meta.title} class="w-full h-full object-cover object-center opacity-30 blur-lg scale-110" />
                {/if}

                <!-- OVERLAYS GRADIENTES (Usando bg-background para respetar el tema) -->
                <div class="absolute inset-0 bg-gradient-to-t from-background via-background/60 to-transparent"></div>
                <div class="absolute inset-0 bg-gradient-to-r from-background via-background/40 to-transparent"></div>
            </div>

            <div class="relative z-10 w-full h-full max-w-[2000px] mx-auto px-4 md:px-12 flex flex-col justify-end pb-16 md:pb-24 pt-32">
                <div class="max-w-3xl space-y-4 md:space-y-6">

                    <h1
                            class="font-black text-foreground tracking-tight drop-shadow-2xl text-3xl md:text-4xl lg:text-5xl leading-tight line-clamp-2 md:line-clamp-3"
                            in:fly={{ y: 20, duration: 800, delay: 200 }}
                    >
                        {meta.title}
                    </h1>

                    <div
                            class="flex flex-wrap items-center gap-3 text-xs md:text-sm font-bold drop-shadow-md text-foreground"
                            in:fly={{ y: 20, duration: 800, delay: 300 }}
                    >
                        {#if currentItem.content.contentType}
                            <span class="bg-secondary text-secondary-foreground px-2.5 py-1 rounded-md uppercase tracking-wider border border-border/50">
                                {formatType(meta.subtype || currentItem.content.contentType)}
                            </span>
                        {/if}

                        {#if formattedScore}
                            <span class="text-green-500 font-black">{formattedScore}% {i18n.t('home.hero.rating')}</span>
                        {/if}

                        {#if meta.releaseDate}
                            <span class="text-muted-foreground">{meta.releaseDate.split('-')[0]}</span>
                        {/if}

                        {#if meta.epsOrChapters}
                            <span class="text-muted-foreground">• {meta.epsOrChapters} {currentItem.content.contentType === 'anime' ? i18n.t('home.hero.eps', { count: meta.epsOrChapters }) : i18n.t('home.hero.chapters', { count: meta.epsOrChapters })}</span>
                        {/if}
                    </div>

                    <!-- Sinopsis -->
                    <div
                            class="text-muted-foreground text-sm md:text-base drop-shadow-lg font-medium leading-relaxed max-w-2xl line-clamp-3 md:line-clamp-4"
                            in:fly={{ y: 20, duration: 800, delay: 400 }}
                    >
                        {@html synopsis?.replace(/<[^>]*>?/gm, '') || i18n.t('home.hero.no_synopsis')}
                    </div>

                    <div
                            class="flex flex-wrap items-center gap-3 pt-4"
                            in:fly={{ y: 20, duration: 800, delay: 500 }}
                    >
                        <a
                                {href}
                                class="bg-primary hover:bg-primary/90 text-primary-foreground font-bold px-6 md:px-8 py-3 rounded-full flex items-center gap-2.5 transition-transform active:scale-95 shadow-lg border border-transparent"
                        >
                            <Play class="w-5 h-5 fill-current" />
                            {currentItem.content.contentType === 'anime' ? i18n.t('home.hero.watch') : i18n.t('home.hero.read')}
                        </a>

                        {#if displayItems.length > 1}
                            <a
                                    {href}
                                    class="bg-secondary/80 hover:bg-secondary text-secondary-foreground backdrop-blur-md font-bold px-6 py-3 rounded-full flex items-center gap-2.5 transition-colors shadow-lg border border-border/50"
                            >
                                <Info class="w-5 h-5" />
                                {i18n.t('home.hero.more_info')}
                            </a>
                        {/if}

                        <Button
                                variant="secondary"
                                class="w-12 h-12 rounded-full p-0 flex items-center justify-center transition-colors shadow-lg border border-border/50"
                                onclick={() => showListModal = true}
                                disabled={isEntryLoading}
                                title={i18n.t('list.add_to_list')}
                        >
                            {#if isEntryLoading}
                                <Loader2 class="w-5 h-5 animate-spin" />
                            {:else if hasEntry}
                                <Check class="w-5 h-5 text-green-500" />
                            {:else}
                                <Plus class="w-5 h-5" />
                            {/if}
                        </Button>
                    </div>
                </div>
            </div>
        {/key}

        {#if displayItems.length > 1}
            <div class="absolute bottom-6 right-6 md:right-12 z-30 flex gap-2">
                {#each displayItems as _, i}
                    <button
                            class="h-1.5 rounded-full transition-all duration-300 shadow-sm {i === currentIndex ? 'w-6 bg-primary' : 'w-2 bg-primary/40 hover:bg-primary/80'}"
                            onclick={() => setSlide(i)}
                            aria-label={`${i18n.t('slide')} ${i + 1}`}
                    ></button>
                {/each}
            </div>
        {/if}
    </div>

    <ListEditor
            bind:open={showListModal}
            cid={currentItem.content.cid}
            title={meta.title}
            contentType={currentItem.content.contentType}
            coverImage={meta.coverImage ?? undefined}
    />
{/if}