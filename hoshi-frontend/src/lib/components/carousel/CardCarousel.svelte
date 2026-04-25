<script lang="ts">
    import * as Carousel from '@/components/ui/carousel';
    import { fly } from 'svelte/transition';
    import CardWrapper from "@/components/card/CardWrapper.svelte";
    import type { NormalizedCard } from "@/utils/normalize";

    let { title, items = [] }: {
        title: string;
        items: NormalizedCard[];
    } = $props();

    let el = $state<HTMLElement | null>(null);
    let visible = $state(false);

    $effect(() => {
        if (!el) return;
        const observer = new IntersectionObserver(
            ([entry]) => { if (entry.isIntersecting) { visible = true; observer.disconnect(); } },
            { rootMargin: '800px' }
        );
        observer.observe(el);
        return () => observer.disconnect();
    });
</script>

<section bind:this={el} class="space-y-3 md:space-y-4 group/section">
    <div class="flex items-center justify-between px-1" in:fly={{ y: 20, duration: 400, delay: 100 }}>
        <h2 class="text-xl md:text-2xl font-bold tracking-tight text-foreground">{title}</h2>
    </div>

    {#if visible && items.length > 0}
        <Carousel.Root
                opts={{ align: 'start', dragFree: true, skipSnaps: false, containScroll: 'trimSnaps' }}
                class="w-full relative group/carousel overflow-visible px-4 md:px-6"
        >
            <Carousel.Content class="-ml-4 md:-ml-5 py-10 overflow-visible">
                {#each items as n (n.cid)}
                    <Carousel.Item class="pl-5 basis-[45%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%] overflow-visible">
                        <CardWrapper {...n} />
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    {:else if !visible}
        <div class="h-56"></div>
    {/if}
</section>

<style>
    :global(.embla__viewport) {
        overflow: visible !important;
    }

    :global(.embla__slide) {
        overflow: visible !important;
    }

    :global(.embla__slide:first-child .preview-card) {
        transform-origin: left center;
    }

    :global(.embla__slide:last-child .preview-card) {
        transform-origin: right center;
    }
</style>