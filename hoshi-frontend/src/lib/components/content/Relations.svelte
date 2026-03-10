<script lang="ts">
    import type { ContentRelation } from "$lib/api/content/types";
    import { contentApi } from "$lib/api/content/content";
    import { primaryMetadata } from "$lib/api/content/types";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Badge } from "$lib/components/ui/badge";
    import { Network } from "lucide-svelte";
    import { i18n } from "$lib/i18n/index.svelte";

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

<div class="space-y-4 sm:space-y-6">
    <h3 class="text-xl font-semibold tracking-tight flex items-center gap-2 px-1">
        <Network class="h-5 w-5 text-primary" /> {i18n.t('related_media')}
    </h3>

    {#if relations.length === 0}
        <p class="text-muted-foreground text-sm px-1">{i18n.t('no_related_media')}</p>
    {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5 sm:gap-4">
            {#each relations as relation}
                {#await contentApi.get(relation.targetCid)}
                    <Skeleton class="h-[72px] sm:h-28 w-full rounded-xl bg-card" />
                {:then res}
                    <!-- EXTRAEMOS LA INFO CORRECTAMENTE SEGÚN LOS NUEVOS TIPOS -->
                    {@const meta = primaryMetadata(res) || {}}
                    {@const content = res.content}

                    <a
                            href={`/content/${content.cid}`}
                            class="flex gap-3 sm:gap-4 p-2 sm:p-2.5 rounded-xl border border-border/40 bg-card hover:bg-muted/30 hover:border-primary/50 hover:shadow-md transition-all group"
                    >
                        {#if meta.coverImage}
                            <img
                                    src={meta.coverImage}
                                    alt={meta.title || 'Cover'}
                                    class="w-12 sm:w-16 aspect-[2/3] object-cover rounded-md sm:rounded-lg shadow-sm shrink-0"
                            />
                        {:else}
                            <Skeleton class="w-12 sm:w-16 aspect-[2/3] rounded-md sm:rounded-lg shrink-0" />
                        {/if}

                        <div class="flex flex-col justify-center py-0.5 overflow-hidden w-full">
                            <Badge variant="secondary" class="w-fit text-[9px] sm:text-[10px] uppercase font-bold tracking-wider mb-1 sm:mb-1.5 bg-primary/10 text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-colors px-1.5 py-0 sm:py-0.5">
                                {formatRelationType(relation.relationType)}
                            </Badge>

                            <h4 class="font-medium text-sm line-clamp-2 leading-tight text-foreground group-hover:text-primary transition-colors">
                                {meta.title || 'Unknown'}
                            </h4>

                            <span class="text-[10px] sm:text-xs text-muted-foreground mt-0.5 sm:mt-1 capitalize truncate">
                                {i18n.t(content.contentType)} • {meta.status ? i18n.t(meta.status.toLowerCase()) : i18n.t('unknown_date')}
                            </span>
                        </div>
                    </a>
                {/await}
            {/each}
        </div>
    {/if}
</div>