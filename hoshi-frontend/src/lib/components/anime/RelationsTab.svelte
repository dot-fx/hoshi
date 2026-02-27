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

<div class="space-y-6">
    <h3 class="text-xl font-semibold tracking-tight flex items-center gap-2">
        <Network class="h-5 w-5 text-primary" /> Related Media
    </h3>

    {#if relations.length === 0}
        <p class="text-muted-foreground text-sm">No related media found.</p>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each relations as relation}
                {#await contentApi.get(relation.targetCid)}
                    <Skeleton class="h-28 w-full rounded-xl bg-card" />
                {:then res}
                    {@const target = res.data}
                    <a
                            href={`/${target.type.toLowerCase()}/${target.cid}`}
                            class="flex gap-4 p-2.5 rounded-xl border border-border/40 bg-card hover:bg-muted/30 hover:border-primary/50 hover:shadow-md transition-all group"
                    >
                        {#if target.coverImage}
                            <img src={target.coverImage} alt={target.title} class="w-16 h-24 object-cover rounded-lg shadow-sm" />
                        {:else}
                            <Skeleton class="w-16 h-24 rounded-lg" />
                        {/if}

                        <div class="flex flex-col justify-center py-1 overflow-hidden w-full">
                            <Badge variant="secondary" class="w-fit text-[10px] uppercase font-bold tracking-wider mb-1.5 bg-primary/10 text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-colors">
                                {formatRelationType(relation.relationType)}
                            </Badge>
                            <h4 class="font-medium text-sm line-clamp-2 leading-tight text-foreground group-hover:text-primary transition-colors">
                                {target.title}
                            </h4>
                            <span class="text-xs text-muted-foreground mt-1 capitalize">
                                {target.type} • {target.status || 'Unknown'}
                            </span>
                        </div>
                    </a>
                {:catch}
                {/await}
            {/each}
        </div>
    {/if}
</div>