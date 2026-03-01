<script lang="ts">
    import type { ContentRelation } from "$lib/api/content/types";
    import { contentApi } from "$lib/api/content/content";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Badge } from "$lib/components/ui/badge";
    import { Network } from "lucide-svelte";

    let { relations }: { relations: ContentRelation[] } = $props();

    function formatRelationType(type: string) {
        return type.replace('_', ' ');
    }
</script>

<div class="space-y-4 sm:space-y-6">
    <h3 class="text-xl font-semibold tracking-tight flex items-center gap-2 px-1">
        <Network class="h-5 w-5 text-primary" /> Related Media
    </h3>

    {#if relations.length === 0}
        <p class="text-muted-foreground text-sm px-1">No related media found.</p>
    {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2.5 sm:gap-4">
            {#each relations as relation}
                {#await contentApi.get(relation.targetCid)}
                    <Skeleton class="h-[72px] sm:h-28 w-full rounded-xl bg-card" />
                {:then res}
                    {@const target = res.data}
                    <a
                            href={`/content/${target.cid}`}
                            class="flex gap-3 sm:gap-4 p-2 sm:p-2.5 rounded-xl border border-border/40 bg-card hover:bg-muted/30 hover:border-primary/50 hover:shadow-md transition-all group"
                    >
                        {#if target.coverImage}
                            <img
                                    src={target.coverImage}
                                    alt={target.title}
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
                                {target.title}
                            </h4>

                            <span class="text-[10px] sm:text-xs text-muted-foreground mt-0.5 sm:mt-1 capitalize truncate">
                                {target.contentType} • {target.status || 'Unknown'}
                            </span>
                        </div>
                    </a>
                {/await}
            {/each}
        </div>
    {/if}
</div>