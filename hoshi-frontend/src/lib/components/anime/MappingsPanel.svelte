<script lang="ts">
    import type { ExtensionSource } from "$lib/api/content/types";
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Link2 } from "lucide-svelte";

    let { extensions }: { extensions: ExtensionSource[] } = $props();
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <Card.Root class="border-border/50 shadow-sm bg-card/50 backdrop-blur-sm">
        <Card.Header class="pb-3">
            <Card.Title class="text-lg flex items-center gap-2">
                <Link2 class="h-5 w-5 text-primary" /> Video Sources
            </Card.Title>
        </Card.Header>
        <Card.Content>
            {#if extensions.length === 0}
                <p class="text-sm text-muted-foreground">No video sources connected.</p>
            {:else}
                <ul class="space-y-3">
                    {#each extensions as ext}
                        <li class="flex items-center justify-between bg-muted/20 p-3 rounded-lg border border-border/40 hover:border-primary/50 transition-colors">
                            <div>
                                <p class="font-medium">{ext.extensionName}</p>
                                <div class="flex gap-2 mt-1">
                                    {#if ext.language}<Badge variant="outline" class="text-[10px] h-4">{ext.language}</Badge>{/if}
                                    {#if ext.quality}<Badge variant="outline" class="text-[10px] h-4">{ext.quality}</Badge>{/if}
                                </div>
                            </div>
                        </li>
                    {/each}
                </ul>
            {/if}
        </Card.Content>
    </Card.Root>
</div>