<script lang="ts">
    import { Skeleton } from '@/components/ui/skeleton';
    import { Tv, Book, BookText } from "lucide-svelte";
    import Hero from '@/components/content/Hero.svelte';
    import ContentCarousel from '@/components/content/Carousel.svelte';
    import ContinueCarousel from '@/components/content/Continue.svelte';
    import { fade } from 'svelte/transition';
    import { contentApi } from '@/api/content/content';
    import { progressApi } from '@/api/progress/progress';
    import type { ContentType, MediaSection } from '@/api/content/types';
    import { appConfig } from "@/config.svelte.js";
    import { layoutState } from '@/layout.svelte.js';
    import { i18n } from '@/i18n/index.svelte.js';
    import { auth } from '@/auth.svelte';
    import { homeState, type MappedHomeSection } from '@/home.svelte.js';
    import type { CoreError } from '@/api/client';

    let loading = $state(false);

    let error = $state<CoreError | null>(null);

    const isSkeletonVisible = $derived((auth.loading || !auth.initialized || loading) && !homeState.hasData);

    let currentMode = $state<ContentType>('anime');
    let initializedMode = $state(false);

    const modes = [
        { id: 'anime', label: 'Anime', icon: Tv },
        { id: 'manga', label: 'Manga', icon: Book },
        { id: 'novel', label: 'Novel', icon: BookText }
    ] as const;

    $effect(() => {
        if (appConfig.data && !initializedMode) {
            currentMode = appConfig.data.ui.defaultHomeSection as ContentType;
            initializedMode = true;
        }
        layoutState.title = i18n.t("home.title");
        layoutState.showBack = false;
        layoutState.backUrl = null;
        layoutState.headerAction = mobileHeaderAction;

        return () => { layoutState.headerAction = undefined; };
    });

    $effect(() => {
        if (auth.loading || !auth.initialized || !auth.user) return;
        loadHomeData();
    });

    async function loadHomeData() {
        if (!homeState.hasData) loading = true;
        error = null;

        try {
            const [res, progRes] = await Promise.all([
                contentApi.getHome(),
                progressApi.getContinueWatching(20)
            ]);

            homeState.content = {
                anime: mapSection(res.anime),
                manga: mapSection(res.manga),
                novel: mapSection(res.novel)
            };
            homeState.continueItems = progRes.items || [];
        } catch (err) {
            console.error("Failed to load home content", err);
            if (!homeState.hasData) {
                error = err as CoreError;
            }
        } finally {
            loading = false;
        }
    }

    let currentContinueItems = $derived(homeState.continueItems.filter(item => item.contentType === currentMode));
    let currentTrending = $derived(homeState.content?.[currentMode]?.trending ?? []);
    let currentSeasonal = $derived(homeState.content?.[currentMode]?.seasonal ?? []);
    let currentTopRated = $derived(homeState.content?.[currentMode]?.topRated ?? []);

    const mapSection = (section: MediaSection | undefined): MappedHomeSection => ({
        trending: section?.trending || [],
        seasonal: section?.seasonal || [],
        topRated: section?.topRated || []
    });
</script>

<svelte:head>
    <title>{i18n.t("home.title")}</title>
</svelte:head>

{#snippet mobileHeaderAction()}
    <div class="flex items-center bg-muted/40 p-1 rounded-xl border border-border/40 shadow-inner h-10 mr-1" in:fade={{duration: 150}}>
        {#each modes as { id, label, icon: Icon }}
            <button
                    class="relative flex items-center justify-center px-3 sm:px-4 h-full rounded-lg text-xs font-bold transition-all duration-300 {currentMode === id ? 'bg-background text-primary shadow-sm ring-1 ring-border/30' : 'text-muted-foreground hover:text-foreground'}"
                    onclick={() => currentMode = id}
                    aria-label={label}
            >
                <Icon class="w-[18px] h-[18px] transition-transform duration-300 {currentMode === id ? 'scale-110' : ''} sm:mr-1.5" />
                <span class="hidden sm:inline-block tracking-wide">{label}</span>
            </button>
        {/each}
    </div>
{/snippet}

<div class="min-h-screen bg-background pb-20 overflow-x-hidden relative">

    {#if isSkeletonVisible}
        <div class="w-full space-y-12" in:fade={{ duration: 300 }}>
            <div class="w-full h-[85vh] bg-card/30 flex items-end p-4 md:p-12">
                <div class="space-y-6 max-w-3xl w-full mb-10">
                    <Skeleton class="h-16 md:h-24 w-3/4 rounded-2xl" />
                    <Skeleton class="h-6 md:h-8 w-1/2 rounded-lg" />
                </div>
            </div>
        </div>
    {:else if error}
        <div class="h-screen w-full flex flex-col items-center justify-center text-muted-foreground gap-4 pt-20">
            <p class="text-lg font-bold">{i18n.t(error.key)}</p>
            <button class="text-primary hover:underline font-medium" onclick={() => location.reload()}>{i18n.t("home.try_again")}</button>
        </div>
    {:else}
        <div in:fade={{ duration: 400 }}>
            {#if currentTrending.length > 0}
                <div class="w-full relative">
                    <Hero items={currentTrending.slice(0, 5)} />
                </div>
            {/if}

            <div class="hidden md:flex fixed top-10 left-1/2 -translate-x-1/2 z-[60] transition-all duration-300">
                <div class="flex items-center p-1.5 bg-background/80 backdrop-blur-xl border border-border/50 rounded-full shadow-lg">
                    {#each modes as { id, label, icon: Icon }}
                        <button
                                class="relative flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-bold transition-all duration-300 {currentMode === id ? 'bg-primary text-primary-foreground shadow-lg scale-105' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
                                onclick={() => currentMode = id}
                        >
                            <Icon class="size-4 shrink-0" />
                            <span>{label}</span>
                        </button>
                    {/each}
                </div>
            </div>

            <div class="w-full px-4 md:px-12 lg:pl-32 py-8 relative z-20 space-y-12 -mt-16 md:-mt-24 pb-safe">
                {#if currentContinueItems.length > 0}
                    <ContinueCarousel items={currentContinueItems} mode={currentMode} />
                {/if}

                {#if currentTrending.length > 0}
                    <ContentCarousel title={i18n.t("home.trending")} items={currentTrending} />
                {/if}

                {#if currentSeasonal.length > 0}
                    <ContentCarousel title={currentMode === 'anime' ? i18n.t("home.simulcast") : i18n.t("home.latest")} items={currentSeasonal} />
                {/if}

                {#if currentTopRated.length > 0}
                    <ContentCarousel title={i18n.t("home.critically_acclaimed")} items={currentTopRated} />
                {/if}
            </div>
        </div>
    {/if}
</div>