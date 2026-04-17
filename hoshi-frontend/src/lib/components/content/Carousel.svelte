<script lang="ts">
    import type {FullContent} from '@/api/content/types';
    import ContentCard from './Card.svelte';
    import * as Carousel from '@/components/ui/carousel';
    import { i18n } from '@/i18n/index.svelte.js';
    import { fade, fly } from 'svelte/transition';
    import { ChevronRight } from 'lucide-svelte';

    let {
        title,
        items = [],
    }: {
        title: string;
        items: FullContent[];
    } = $props();

    let displayItems = $derived(items);
</script>

<section class="space-y-4 group/section">
    <div class="flex items-center justify-between px-1" in:fly={{ y: 20, duration: 400, delay: 100 }}>
        <h2 class="text-xl md:text-2xl font-bold tracking-tight text-foreground">
            {title}
        </h2>
    </div>

    {#if displayItems.length > 0}
        <div in:fade={{ duration: 500 }}>
            <Carousel.Root
                    opts={{
                        align: 'start',
                        dragFree: true,
                        skipSnaps: false,
                        containScroll: 'trimSnaps'
                    }}
                    class="w-full relative group/carousel"
            >
                <Carousel.Content class="-ml-4 py-4">
                    {#each displayItems as item, i (item.content.cid)}
                        <Carousel.Item class="pl-4 basis-[45%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]">
                            <div in:fade={{ duration: 400, delay: 150 + (i * 50) }}>
                                <ContentCard {item} />
                            </div>
                        </Carousel.Item>
                    {/each}
                </Carousel.Content>

                <div class="hidden md:block opacity-0 group-hover/carousel:opacity-100 transition-opacity duration-300">
                    <Carousel.Previous class="absolute -left-4 top-1/2 -translate-y-1/2 shadow-md bg-background/80 backdrop-blur-sm" />
                    <Carousel.Next class="absolute -right-4 top-1/2 -translate-y-1/2 shadow-md bg-background/80 backdrop-blur-sm" />
                </div>
            </Carousel.Root>
        </div>
    {/if}
</section>