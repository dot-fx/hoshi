<script lang="ts">
    import type { ContentWithMappings } from '@/api/content/types';
    import { primaryMetadata } from '@/api/content/types';
    import { AspectRatio } from '$lib/components/ui/aspect-ratio';
    import * as HoverCard from '$lib/components/ui/hover-card';
    import { i18n } from '$lib/i18n/index.svelte';
    import { Star, Play, BookmarkPlus, Tv, Calendar } from 'lucide-svelte';

    import ListDialog from '$lib/components/ListEditorModal.svelte';

    // ⚠️ AÑADIDO: Prop para desactivar el hover cuando lo necesites (ej: en 'Mi Lista')
    let { item, disableHover = false }: { item: ContentWithMappings, disableHover?: boolean } = $props();

    let meta = $derived(item ? primaryMetadata(item) : undefined);
    let href = $derived(item?.content?.cid ? `/content/${item.content.cid}` : '#');
    let year = $derived(meta?.releaseDate ? meta.releaseDate.split('-')[0] : null);

    let formattedScore = $derived(meta?.rating ? Math.round(meta.rating * 10) : null);
    let isListDialogOpen = $state(false);

    // Función para extraer el ID de YouTube y convertirlo en un iframe embed
    const getYoutubeId = (url: string | undefined | null) => {
        if (!url) return null;
        const regExp = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|&v=)([^#&?]*).*/;
        const match = url.match(regExp);
        return (match && match[2].length === 11) ? match[2] : null;
    };
    let ytId = $derived(getYoutubeId(meta?.trailerUrl));

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        if (type === 'TV') return i18n.t('series');
        const formatted = type.replace('_', ' ').toLowerCase();
        const translationKey = formatted.replace(' ', '_') as any;
        const translated = i18n.t(translationKey);
        return translated === translationKey ? formatted.replace(/\b\w/g, l => l.toUpperCase()) : translated;
    };

    const formatStatus = (status: string | undefined | null) => {
        if (!status) return '';
        const key = status.toLowerCase() as any;
        const translated = i18n.t(key);
        return translated === key ? status : translated;
    };
</script>

{#if item && meta}
    <ListDialog
            bind:open={isListDialogOpen}
            cid={item.content.cid}
            title={meta.title}
            contentType={item.content.contentType}
            coverImage={meta.coverImage || ''}
    />

    <!-- ========================================== -->
    <!-- 1. DEFINICIÓN DEL SNIPPET DE LA TARJETA BASE -->
    <!-- ========================================== -->
    {#snippet baseCard()}
        <div class="flex flex-col gap-2.5">
            <!-- Contenedor de Imagen -->
            <div class="relative overflow-hidden rounded-xl bg-muted/20 shadow-sm ring-1 ring-border/10 transition-all duration-300 group-hover:shadow-lg group-hover:shadow-primary/5 group-hover:ring-primary/20">
                <AspectRatio ratio={2/3}>
                    <img src={meta.coverImage} alt={meta.title} loading="lazy" class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105" />

                    <!-- Gradiente sutil que aparece en hover para darle profundidad -->
                    <div class="absolute inset-0 bg-gradient-to-t from-background/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"></div>
                </AspectRatio>

                <!-- Rating Badge -->
                {#if formattedScore}
                    <div class="absolute top-2 right-2 bg-black/70 backdrop-blur-md px-2 py-1 rounded-lg text-xs font-black text-white flex items-center gap-1.5 shadow-sm border border-white/10">
                        <Star class="w-3.5 h-3.5 text-yellow-500 fill-yellow-500" />
                        {(formattedScore / 10).toFixed(1)}
                    </div>
                {/if}
            </div>

            <!-- Texto Base -->
            <div class="space-y-1.5 px-1">
                <div class="flex justify-between items-center text-xs font-bold text-muted-foreground/80">
                    <span class="uppercase tracking-wider">{formatType(meta.subtype)}</span>
                    {#if year}<span>{year}</span>{/if}
                </div>

                <div class="flex items-start gap-2">
                    <!-- Indicador de emisión (Punto verde con pulso) -->
                    {#if meta.status?.toUpperCase() === 'RELEASING'}
                        <span class="relative flex h-2 w-2 mt-1.5 shrink-0" title="Airing Now">
                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                            <span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                        </span>
                    {/if}

                    <h3 class="font-bold text-sm md:text-base leading-tight line-clamp-2 text-foreground/90 group-hover:text-primary transition-colors">
                        {meta.title}
                    </h3>
                </div>
            </div>
        </div>
    {/snippet}

    <!-- ========================================== -->
    <!-- 2. RENDERIZADO CONDICIONAL (Hover vs Simple) -->
    <!-- ========================================== -->

    {#if disableHover}
        <!-- MODO SIMPLE: Sin HoverCard (Ideal para tu página de 'Mi Lista') -->
        <a {href} class="block w-full outline-none group cursor-pointer">
            {@render baseCard()}
        </a>
    {:else}
        <!-- MODO COMPLETO: Con HoverCard y Popover (Por defecto) -->
        <HoverCard.Root openDelay={350} closeDelay={150}>

            <HoverCard.Trigger>
                {#snippet child({ props })}
                    <a {href} {...props} class="block w-full outline-none group cursor-pointer">
                        {@render baseCard()}
                    </a>
                {/snippet}
            </HoverCard.Trigger>

            <!-- TARJETA EXPANDIDA (POPOVER) -->
            <HoverCard.Content
                    side="right"
                    align="start"
                    sideOffset={12}
                    class="w-[320px] p-0 overflow-hidden shadow-2xl border-border/40 rounded-xl z-50 hidden md:flex flex-col bg-card"
            >
                <!-- Área superior: Trailer Auto-play o Banner -->
                <a {href} class="relative w-full aspect-video bg-black block group cursor-pointer overflow-hidden">
                    {#if ytId}
                        <!-- Reproductor silencioso de YouTube -->
                        <iframe
                                src="https://www.youtube.com/embed/{ytId}?autoplay=1&mute=1&controls=0&loop=1&playlist={ytId}&showinfo=0&modestbranding=1"
                                class="absolute inset-0 w-full h-full object-cover scale-[1.35] pointer-events-none opacity-90"
                                frameborder="0"
                                allow="autoplay; encrypted-media"
                                title="Trailer"
                        ></iframe>
                    {:else if meta.bannerImage}
                        <img src={meta.bannerImage} alt="Banner" class="w-full h-full object-cover opacity-90" />
                    {:else}
                        <img src={meta.coverImage} alt="Cover" class="w-full h-full object-cover blur-md scale-110 opacity-70" />
                    {/if}

                    <!-- Overlay gradiente y Botón Play -->
                    <div class="absolute inset-0 bg-gradient-to-t from-card via-black/20 to-transparent flex items-center justify-center transition-colors group-hover:bg-black/30">
                        <div class="w-12 h-12 bg-white/20 backdrop-blur-sm border border-white/30 rounded-full flex items-center justify-center shadow-lg transition-transform group-hover:scale-110 group-hover:bg-primary group-hover:border-primary">
                            <Play class="w-5 h-5 text-white fill-white ml-1" />
                        </div>
                    </div>
                </a>

                <!-- Área de Información -->
                <div class="p-4 flex flex-col gap-3 -mt-2 relative z-10">
                    <h3 class="font-bold text-base leading-tight">{meta.title}</h3>

                    <div class="flex flex-wrap gap-2 items-center">
                        {#if meta.subtype}
                            <span class="text-[10px] uppercase font-bold bg-muted px-2 py-1 rounded border border-border/50 text-muted-foreground">{formatType(meta.subtype)}</span>
                        {/if}
                        {#if meta.status}
                            <span class="text-[10px] uppercase font-bold {meta.status.toUpperCase() === 'RELEASING' ? 'text-green-500 border-green-500/30 bg-green-500/10' : 'text-primary border-primary/30 bg-primary/10'} border px-2 py-1 rounded">
                                {formatStatus(meta.status)}
                            </span>
                        {/if}
                        {#if meta.genres?.[0]}
                            <span class="text-[10px] font-bold bg-muted px-2 py-1 rounded border border-border/50 text-muted-foreground">{meta.genres[0]}</span>
                        {/if}
                    </div>

                    <div class="flex items-center gap-4 text-xs text-muted-foreground font-semibold">
                        {#if formattedScore}
                            <span class="flex items-center gap-1 text-yellow-500"><Star class="w-3.5 h-3.5 fill-yellow-500" /> {formattedScore}%</span>
                        {/if}
                        {#if meta.epsOrChapters}
                            <span class="flex items-center gap-1"><Tv class="w-3.5 h-3.5" /> {meta.epsOrChapters} eps</span>
                        {/if}
                        {#if meta.releaseDate}
                            <span class="flex items-center gap-1"><Calendar class="w-3.5 h-3.5" /> {meta.releaseDate}</span>
                        {/if}
                    </div>

                    {#if meta.synopsis}
                        <p class="text-xs text-muted-foreground line-clamp-3 leading-relaxed mt-1" title={meta.synopsis}>
                            {meta.synopsis.replace(/<[^>]*>?/gm, '')}
                        </p>
                    {/if}

                    <div class="flex gap-2 mt-3">
                        <a {href} class="flex-1 bg-primary text-primary-foreground text-sm font-bold py-2.5 rounded-lg flex items-center justify-center gap-2 hover:opacity-90 transition-opacity shadow-sm">
                            <Play class="w-4 h-4 fill-current" /> Watch now
                        </a>

                        <button
                                onclick={(e) => { e.preventDefault(); isListDialogOpen = true; }}
                                class="w-11 h-11 rounded-lg bg-muted border border-border/50 flex items-center justify-center hover:bg-muted/80 hover:text-primary transition-colors shrink-0 shadow-sm"
                                title={i18n.t('add_to_list') || 'Add to list'}
                        >
                            <BookmarkPlus class="w-5 h-5" />
                        </button>
                    </div>
                </div>
            </HoverCard.Content>
        </HoverCard.Root>
    {/if}
{/if}