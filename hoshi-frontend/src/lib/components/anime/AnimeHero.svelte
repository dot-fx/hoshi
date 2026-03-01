<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Play, Film, Plus, Check, Loader2 } from "lucide-svelte";
    import ListEditorModal from '$lib/components/ListEditorModal.svelte';
    import { listApi } from '@/api/list/list';

    let { metadata }: { metadata: any } = $props();

    let showListModal = $state(false);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);

    $effect(() => {
        if (metadata?.cid) {
            checkListStatus(metadata.cid);
        }
    });

    async function checkListStatus(cid: string) {
        isEntryLoading = true;
        try {
            const res = await listApi.getEntry(cid);
            hasEntry = res.found;
        } catch (err) {
            console.error("Error checking list status:", err);
            hasEntry = false;
        } finally {
            isEntryLoading = false;
        }
    }
</script>

<div class="relative w-full">
    <div class="absolute inset-0 w-full h-112.5 md:h-137.5 z-0 overflow-hidden">
        {#if metadata.bannerImage}
            <img
                    src={metadata.bannerImage}
                    alt="Banner"
                    class="w-full h-full object-cover opacity-40 blur-[2px]"
            />
        {:else if metadata.coverImage}
            <img
                    src={metadata.coverImage}
                    alt="Banner Fallback"
                    class="w-full h-full object-cover opacity-20 blur-md"
            />
        {/if}
        <div class="absolute inset-0 bg-linear-to-t from-background via-background/80 to-transparent"></div>
    </div>

    <div class="container mx-auto px-4 md:px-12 max-w-350 relative z-10 pt-32 md:pt-40 pb-16">
        <div class="flex flex-col md:flex-row gap-6 md:gap-10 items-start">

            <div class="w-40 md:w-64 shrink-0 rounded-xl overflow-hidden shadow-2xl shadow-black/60 border border-border/30">
                {#if metadata.coverImage}
                    <img src={metadata.coverImage} alt={metadata.title} class="w-full h-auto object-cover aspect-2/3" />
                {:else}
                    <div class="w-full aspect-2/3 bg-muted flex items-center justify-center">
                        <Film class="h-12 w-12 text-muted-foreground" />
                    </div>
                {/if}
            </div>

            <div class="flex flex-col flex-1 pt-2 md:pt-6">
                <h1 class="text-3xl md:text-5xl lg:text-6xl font-bold tracking-tight text-foreground balance-text">
                    {metadata.title}
                </h1>

                {#if metadata.altTitles && metadata.altTitles.length > 0}
                    <p class="text-muted-foreground mt-3 text-sm md:text-base font-medium">
                        {metadata.altTitles.slice(0, 2).join(" • ")}
                    </p>
                {/if}

                <div class="flex flex-wrap gap-2 mt-5">
                    {#if metadata.status}
                        <Badge variant={metadata.status.toLowerCase() === 'ongoing' ? 'default' : 'secondary'} class="font-semibold capitalize">
                            {metadata.status}
                        </Badge>
                    {/if}
                    {#if metadata.rating}
                        <Badge variant="outline" class="bg-background/50 backdrop-blur-sm border-primary/30 text-primary font-bold">
                            ★ {metadata.rating} / 10
                        </Badge>
                    {/if}
                </div>

                <p class="mt-6 text-foreground/80 leading-relaxed md:text-lg max-w-4xl line-clamp-4 hover:line-clamp-none transition-all duration-300">
                    {@html metadata.synopsis || "No synopsis available."}
                </p>

                <div class="mt-8 flex gap-4">
                    <Button size="lg" class="shadow-lg hover:scale-105 transition-transform">
                        <Play class="mr-2 h-5 w-5 fill-current" />
                        Watch Now
                    </Button>
                    {#if metadata.trailerUrl}
                        <Button size="lg" variant="secondary" class="shadow-sm hover:scale-105 transition-transform" href={metadata.trailerUrl} target="_blank">
                            <Film class="mr-2 h-5 w-5" />
                            Trailer
                        </Button>
                    {/if}

                    <Button
                            size="lg"
                            variant="secondary"
                            class="shadow-sm hover:scale-105 transition-transform"
                            onclick={() => showListModal = true}
                            disabled={isEntryLoading}
                    >
                        {#if isEntryLoading}
                            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
                            Loading...
                        {:else if hasEntry}
                            <Check class="mr-2 h-5 w-5 text-green-500" />
                            In My List
                        {:else}
                            <Plus class="mr-2 h-5 w-5" />
                            My List
                        {/if}
                    </Button>
                </div>
            </div>
        </div>
    </div>
</div>

{#if metadata}
    <ListEditorModal
            bind:open={showListModal}
            cid={metadata.cid}
            title={metadata.title}
            contentType={metadata.contentType || 'anime'}
            coverImage={metadata.coverImage ?? undefined}
    />
{/if}