<script lang="ts">
    import type { TrackerMedia } from '@/api/content/types';
    import ContentCard from './Card.svelte';
    import * as Carousel from '@/components/ui/carousel';
    import { i18n } from '@/i18n/index.svelte.js';
    import { fade, fly } from 'svelte/transition';

    let { title, items = [] }: { title: string; items: TrackerMedia[] } = $props();
    let displayItems = $derived(items.slice(0, 15));
</script>

<section class="space-y-4">
    <h2
            class="text-xl md:text-2xl font-bold tracking-tight text-foreground px-1"
            in:fly={{ y: 20, duration: 400, delay: 100 }}
    >
        {title}
    </h2>

    {#if displayItems.length > 0}
        <div in:fade={{ duration: 500 }}>
            <Carousel.Root
                    opts={{
                    align: 'start',
                    dragFree: true,
                    skipSnaps: false,
                    containScroll: 'trimSnaps'
                }}
                    class="w-full relative"
            >
                <Carousel.Content class="-ml-4 py-4 carousel-track">
                    {#each displayItems as item (item.trackerId)}
                        <Carousel.Item class="pl-4 basis-[45%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]">
                        <ContentCard {item} source="anilist" />
                    </Carousel.Item>
                    {/each}
                </Carousel.Content>
            </Carousel.Root>
        </div>
    {:else}
        <div
                class="text-muted-foreground text-sm px-1"
                in:fade={{ duration: 400 }}
        >
            {i18n.t('home.no_content')}
        </div>
    {/if}
</section>

<style>
    :global(.carousel-track) {
        transition: transform 450ms cubic-bezier(0.22, 1, 0.36, 1);
        will-change: transform;
        scroll-snap-type: x mandatory;
        cursor: grab;
    }

    :global(.carousel-track > *) {
        scroll-snap-align: start;
    }

    :global(.carousel-track:active) {
        cursor: grabbing;
    }
</style>