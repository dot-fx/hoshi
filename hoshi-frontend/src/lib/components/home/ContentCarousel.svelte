<script lang="ts">
    import type { ContentWithMappings } from '@/api/content/types';
    import ContentCard from './ContentCard.svelte';
    import * as Carousel from '$lib/components/ui/carousel';
    import { i18n } from '$lib/i18n/index.svelte'; // <-- Importar i18n

    let { title, items = [] }: { title: string; items: ContentWithMappings[] } = $props();
</script>

<section class="space-y-4">
    <!-- El título ya debería venir traducido desde donde se llama al componente -->
    <h2 class="text-xl md:text-2xl font-bold tracking-tight text-foreground px-1">
        {title}
    </h2>

    {#if items.length > 0}
        <Carousel.Root opts={{ align: 'start', loop: true, dragFree: true }} class="w-full relative group/carousel">

            <Carousel.Content class="-ml-4 py-4">
                {#each items as item (item.content.cid)}
                    <Carousel.Item class="pl-4 basis-[45%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]">
                        <ContentCard {item} />
                    </Carousel.Item>
                {/each}
            </Carousel.Content>

            <Carousel.Previous
                    class="absolute left-2 top-1/2 -translate-y-1/2 z-40 h-12 w-12 md:h-14 md:w-14 rounded-full border border-white/10 bg-background/50 backdrop-blur-xl text-foreground shadow-[0_0_20px_rgba(0,0,0,0.5)] opacity-0 group-hover/carousel:opacity-100 transition-all duration-300 hover:scale-110 hover:bg-foreground hover:text-background hidden md:flex items-center justify-center disabled:opacity-0"
            />

            <Carousel.Next
                    class="absolute right-2 top-1/2 -translate-y-1/2 z-40 h-12 w-12 md:h-14 md:w-14 rounded-full border border-white/10 bg-background/50 backdrop-blur-xl text-foreground shadow-[0_0_20px_rgba(0,0,0,0.5)] opacity-0 group-hover/carousel:opacity-100 transition-all duration-300 hover:scale-110 hover:bg-foreground hover:text-background hidden md:flex items-center justify-center disabled:opacity-0"
            />

        </Carousel.Root>
    {:else}
        <div class="text-muted-foreground text-sm px-1">{i18n.t('no_content_available')}</div> <!-- Traducido -->
    {/if}
</section>