<script lang="ts">
    import type { ContentRelation } from "$lib/api/content/types";
    import { contentApi } from "$lib/api/content/content";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { i18n } from "$lib/i18n/index.svelte";
    import ContentCard from "@/components/content/Card.svelte";

    let { relations }: { relations: ContentRelation[] } = $props();

    function formatRelationType(type: string) {
        const key = type.toLowerCase() as any;
        const translated = i18n.t(key);

        if (translated === key) {
            return type.replace(/_/g, ' ').toLowerCase().replace(/\b\w/g, l => l.toUpperCase());
        }
        return translated;
    }
</script>

<div class="space-y-6">
    <h3 class="text-xl md:text-2xl font-bold tracking-tight">{i18n.t('related_media') || 'Related Media'}</h3>

    {#if relations.length === 0}
        <p class="text-muted-foreground">{i18n.t('no_related_media') || 'No related media found.'}</p>
    {:else}
        <!-- Grid adaptado a tarjetas verticales -->
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 sm:gap-6">
            {#each relations as relation}
                <div class="relative group w-full h-full">

                    <!-- SKELETON MIENTRAS CARGA -->
                    {#await contentApi.get(relation.targetCid)}
                        <div class="flex flex-col gap-2.5 w-full">
                            <Skeleton class="aspect-[2/3] w-full rounded-xl bg-muted/20" />
                            <div class="space-y-2 px-1">
                                <Skeleton class="h-3 w-1/3 bg-muted" />
                                <Skeleton class="h-4 w-3/4 bg-muted" />
                            </div>
                        </div>

                        <!-- CONTENIDO CARGADO -->
                    {:then res}
                        <div class="relative h-full w-full block">
                            <!-- Etiqueta de Relación flotando encima de tu ContentCard -->
                            <div class="absolute top-2 left-2 z-20 bg-background/95 backdrop-blur-md px-2 py-1 rounded text-[9px] font-black uppercase tracking-widest border border-border/50 shadow-md text-foreground pointer-events-none">
                                {formatRelationType(relation.relationType)}
                            </div>

                            <!-- TU COMPONENTE ORIGINAL -->
                            <ContentCard item={res} />
                        </div>
                    {/await}
                </div>
            {/each}
        </div>
    {/if}
</div>