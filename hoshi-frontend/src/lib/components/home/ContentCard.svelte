<script lang="ts">
    import type { CoreMetadata } from '@/api/content/types';
    import { Card, CardContent } from '$lib/components/ui/card';
    import { AspectRatio } from '$lib/components/ui/aspect-ratio';

    let { item }: { item: CoreMetadata } = $props();

    let href = $derived(item ? `/content/${item.cid}` : '#');

    let year = $derived(item?.releaseDate ? item.releaseDate.split('-')[0] : null);

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        if (type === 'TV') return 'Series';
        return type.replace('_', ' ').toLowerCase().replace(/\b\w/g, l => l.toUpperCase());
    };
</script>

{#if item}
    <a {href} class="group block w-full outline-none">
        <div class="relative transition-all duration-300
              group-hover:-translate-y-1
              group-hover:scale-[1.02]">

            <div class="relative overflow-hidden rounded-xl">

                <AspectRatio ratio={2/3}>
                    <img
                            src={item.coverImage}
                            alt={item.title}
                            loading="lazy"
                            class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-[1.05]"
                    />
                </AspectRatio>

                <div class="absolute inset-0 bg-linear-to-t from-black/80 via-black/30 to-transparent"> </div>

                {#if item.rating}
                    <div class="absolute top-2 right-2
                    bg-black/70 backdrop-blur px-2 py-1
                    rounded-md text-xs font-semibold text-white">
                        ⭐ {item.rating.toFixed(1)}
                    </div>
                {/if}

                {#if item.nsfw}
                    <div class="absolute top-2 left-2
                    bg-red-600 text-white text-[10px]
                    px-2 py-0.5 rounded font-semibold">
                        18+
                    </div>
                {/if}

                {#if item.subtype}
                    <div class="absolute bottom-2 left-2
                    bg-white/10 backdrop-blur
                    text-white text-[10px]
                    px-2 py-0.5 rounded">
                        {formatType(item.subtype)}
                    </div>
                {/if}
            </div>

            <div class="mt-3 space-y-1">
                <h3 class="font-semibold text-sm text-foreground
                 leading-snug line-clamp-2 min-h-10
                 group-hover:text-primary transition-colors">
                    {item.title}
                </h3>

                <div class="flex items-center gap-2 text-xs text-muted-foreground">
                    {#if year}
                        <span>{year}</span>
                    {/if}

                    {#if item.status}
                        <span>•</span>
                        <span>{item.status}</span>
                    {/if}

                    {#if item.studio}
                        <span>•</span>
                        <span class="truncate max-w-20">{item.studio}</span>
                    {/if}
                </div>
            </div>
        </div>
    </a>
{/if}