<script lang="ts">
    import Hero from '$lib/components/content/ContentHero.svelte';
    import ContentCarousel from '$lib/components/home/ContentCarousel.svelte';
    import { Skeleton } from '$lib/components/ui/skeleton';
    import { fade } from 'svelte/transition';
    import { contentApi } from '@/api/content/content';

    let loading = $state(true);
    let error = $state(false);
    let content = $state<Record<string, any>>({});

    $effect(() => {
        contentApi.getHome()
            .then((res: any) => {
                content = res;
            })
            .catch(() => {
                error = true;
            })
            .finally(() => {
                loading = false;
            });
    });
</script>

<svelte:head>
    <title>Home</title>
</svelte:head>

<div class="min-h-screen bg-background pb-20 overflow-x-hidden">

    {#if loading}
        <div class="w-full h-[85vh] bg-card/50 flex items-end p-4 md:p-12 animate-pulse">
            <div class="space-y-6 max-w-3xl w-full mb-10">
                <Skeleton class="h-12 md:h-20 w-3/4 bg-muted rounded-lg" />
                <div class="flex gap-4">
                    <Skeleton class="h-6 w-20 bg-muted rounded" />
                    <Skeleton class="h-6 w-20 bg-muted rounded" />
                    <Skeleton class="h-6 w-20 bg-muted rounded" />
                </div>
                <div class="space-y-2">
                    <Skeleton class="h-4 w-full bg-muted rounded" />
                    <Skeleton class="h-4 w-2/3 bg-muted rounded" />
                </div>
                <div class="flex gap-4 pt-4">
                    <Skeleton class="h-14 w-32 bg-muted rounded" />
                    <Skeleton class="h-14 w-32 bg-muted rounded" />
                </div>
            </div>
        </div>

    {:else if error}
        <div class="h-screen w-full flex flex-col items-center justify-center text-muted-foreground gap-4">
            <p class="text-lg">Failed to load content.</p>
            <button class="text-white hover:underline" onclick={() => location.reload()}>
                Try again
            </button>
        </div>

    {:else}
        <div in:fade={{ duration: 500 }}>

            {#if content.trending_anime && content.trending_anime.length > 0}
                <Hero items={content.trending_anime.slice(0, 5)} />
            {/if}

            <div class="w-full px-4 md:px-12 py-8 relative z-20 space-y-12 -mt-16 md:-mt-24">
                {#if content.trending_anime?.length}
                    <ContentCarousel title="Trending Now" items={content.trending_anime} />
                {/if}

                {#if content.seasonal?.length}
                    <ContentCarousel title="Simulcast Season" items={content.seasonal} />
                {/if}

                {#if content.trending_manga?.length}
                    <ContentCarousel title="Top Manga Reading" items={content.trending_manga} />
                {/if}

                {#if content.top_rated?.length}
                    <ContentCarousel title="Critically Acclaimed" items={content.top_rated} />
                {/if}
            </div>
        </div>
    {/if}

</div>