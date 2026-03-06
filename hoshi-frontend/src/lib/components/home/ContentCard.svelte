<script lang="ts">
    import type { CoreMetadata } from '@/api/content/types';
    import { AspectRatio } from '$lib/components/ui/aspect-ratio';
    import { i18n } from '$lib/i18n/index.svelte';
    import { Star } from 'lucide-svelte'; // <-- Importamos icono para el rating

    let { item }: { item: CoreMetadata } = $props();

    let href = $derived(item ? `/content/${item.cid}` : '#');

    let year = $derived(item?.releaseDate ? item.releaseDate.split('-')[0] : null);

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        if (type === 'TV') return i18n.t('series');

        const formatted = type.replace('_', ' ').toLowerCase();
        const translationKey = formatted.replace(' ', '_') as any;
        const translated = i18n.t(translationKey);

        if (translated === translationKey) {
            return formatted.replace(/\b\w/g, l => l.toUpperCase());
        }
        return translated;
    };

    const formatStatus = (status: string | undefined | null) => {
        if (!status) return '';
        const key = status.toLowerCase() as any;
        const translated = i18n.t(key);
        return translated === key ? status : translated;
    }
</script>

{#if item}
    <a {href} class="group block w-full outline-none">
        <div class="flex flex-col gap-3">

            <!-- Contenedor de Imagen -->
            <div class="relative overflow-hidden rounded-xl bg-muted shadow-sm border border-border/50 group-hover:border-primary/50 group-hover:shadow-md transition-all duration-300">

                <AspectRatio ratio={2/3}>
                    <img
                            src={item.coverImage}
                            alt={item.title}
                            loading="lazy"
                            class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
                    />
                </AspectRatio>

                <!-- Degradado inferior para resaltar las etiquetas -->
                <div class="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/80 via-black/20 to-transparent pointer-events-none"></div>

                <!-- Badge: Rating -->
                {#if item.rating}
                    <div class="absolute top-2 right-2 bg-black/60 backdrop-blur-md px-1.5 py-1 rounded-lg text-[10px] font-bold text-white flex items-center gap-1 border border-white/10 shadow-sm">
                        <Star class="w-3 h-3 text-yellow-500 fill-yellow-500" />
                        {item.rating.toFixed(1)}
                    </div>
                {/if}

                <!-- Badge: NSFW -->
                {#if item.nsfw}
                    <div class="absolute top-2 left-2 bg-destructive/90 backdrop-blur-md text-destructive-foreground text-[9px] uppercase tracking-widest px-2 py-0.5 rounded-md font-black shadow-sm">
                        {i18n.t('nsfw')}
                    </div>
                {/if}

                <!-- Badge: Formato (TV, Movie, etc.) -->
                {#if item.subtype}
                    <div class="absolute bottom-2 left-2 bg-background/90 text-foreground text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-md shadow-sm backdrop-blur-md border border-border/50">
                        {formatType(item.subtype)}
                    </div>
                {/if}
            </div>

            <!-- Contenedor de Texto -->
            <div class="space-y-1.5 px-0.5">
                <h3 class="font-bold text-sm leading-tight line-clamp-2 min-h-[2.5rem] group-hover:text-primary transition-colors text-foreground/90" title={item.title}>
                    {item.title}
                </h3>

                <div class="flex items-center gap-1.5 text-[11px] font-semibold text-muted-foreground/80 truncate">
                    {#if year}
                        <span>{year}</span>
                    {/if}

                    {#if item.status}
                        {#if year}<span class="text-muted-foreground/40">•</span>{/if}
                        <span class="truncate">{formatStatus(item.status)}</span>
                    {/if}

                    {#if item.studio}
                        {#if year || item.status}<span class="text-muted-foreground/40">•</span>{/if}
                        <span class="truncate">{item.studio}</span>
                    {/if}
                </div>
            </div>

        </div>
    </a>
{/if}