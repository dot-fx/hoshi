<script lang="ts">
    import type { ContentWithMappings } from '@/api/content/types';
    import ContentCard from './Card.svelte';
    import * as Carousel from '@/components/ui/carousel';
    import { i18n } from '@/i18n/index.svelte.js';

    let { title, items = [] }: { title: string; items: ContentWithMappings[] } = $props();
    let displayItems = $derived(items.slice(0, 15));
</script>

<section class="space-y-4">
    <h2 class="text-xl md:text-2xl font-bold tracking-tight text-foreground px-1">
        {title}
    </h2>

    {#if displayItems.length > 0}
        <Carousel.Root opts={{ align: 'start', dragFree: true }} class="w-full relative">
            <Carousel.Content class="-ml-4 py-4">
                {#each displayItems as item (item.content.cid)}
                    <Carousel.Item class="pl-4 basis-[45%] sm:basis-1/3 md:basis-1/4 lg:basis-1/5 xl:basis-1/6 2xl:basis-[14%]">
                        <ContentCard {item} />
                    </Carousel.Item>
                {/each}
            </Carousel.Content>
        </Carousel.Root>
    {:else}
        <div class="text-muted-foreground text-sm px-1">{i18n.t('home.no_content')}</div>
    {/if}
</section>