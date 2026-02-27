<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { PlayCircle } from "lucide-svelte";
    import * as Select from "$lib/components/ui/select";

    let { cid, extensions, epsOrChapters }: { cid: string, extensions: any[], epsOrChapters?: number | null } = $props();

    const totalEpisodes = epsOrChapters && epsOrChapters > 0 ? epsOrChapters : 12;

    const mockEpisodes = Array.from({ length: totalEpisodes }, (_, i) => ({
    number: i + 1,
        title: `Episode ${i + 1}`,
        isWatched: false
    }));
</script>

<div class="space-y-6">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <h2 class="text-2xl font-semibold tracking-tight">Episodes</h2>

        {#if extensions.length > 0}
            <div class="w-full sm:w-48">
                <Select.Root>
                    <Select.Trigger class="h-9">
                        <Select.Value placeholder="Source ({extensions[0].extensionName})" />
                    </Select.Trigger>
                    <Select.Content>
                        {#each extensions as ext}
                            <Select.Item value={ext.extensionId}>{ext.extensionName}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>
        {/if}
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
        {#each mockEpisodes as ep}
            <Button
                    variant={ep.isWatched ? "secondary" : "outline"}
                    class="h-14 justify-start px-4 w-full relative group overflow-hidden border-border/50 shadow-sm hover:border-primary/50"
            >
                <div class="flex items-center gap-4 z-10 w-full">
                    <span class="text-xl font-black text-muted-foreground/40 group-hover:text-primary/70 transition-colors min-w-[24px]">
                        {ep.number}
                    </span>
                    <span class="font-medium flex-1 text-left line-clamp-1">{ep.title}</span>
                    <PlayCircle class="h-5 w-5 opacity-0 group-hover:opacity-100 transition-opacity text-primary" />
                </div>
                {#if ep.isWatched}
                    <div class="absolute bottom-0 left-0 h-1 bg-primary/40 w-full"></div>
                {/if}
            </Button>
        {/each}
    </div>
</div>