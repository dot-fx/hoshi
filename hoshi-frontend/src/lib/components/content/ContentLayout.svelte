<script lang="ts">
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { fade } from "svelte/transition";

    import ContentHero from "$lib/components/content/ContentHero.svelte";
    import ContentSidebar from "$lib/components/content/ContentSidebar.svelte";

    import EpisodeSelector from "$lib/components/content/EpisodeSelector.svelte";
    import ChapterTable from "$lib/components/content/ChapterTable.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import RelationsTab from "$lib/components/content/Relations.svelte";

    import { Skeleton } from "$lib/components/ui/skeleton";

    import * as Drawer from "$lib/components/ui/drawer";
    import { Button } from "$lib/components/ui/button";
    import { Info } from "lucide-svelte";

    let { cid }: { cid: string } = $props();

    const contentPromise = $derived(contentApi.get(cid || ""));

    const extensionsPromise = $derived(
        contentPromise.then(res => {
            const type = res.data.contentType;
            if (type === 'anime') return extensionsApi.getAnime();
            if (type === 'manga') return extensionsApi.getManga();
            return extensionsApi.getNovel();
        })
    );
</script>

<svelte:head>
    {#await contentPromise}
        <title>Loading...</title>
    {:then res}
        <title>{res.data.title}</title>
    {:catch e}
        <title>Error</title>
    {/await}
</svelte:head>

<main class="min-h-screen bg-background pb-20 overflow-x-hidden">
    {#await contentPromise}
        <div in:fade={{ duration: 200 }} class="w-full">
            <div class="relative w-full h-[400px] md:h-[550px]">
                <Skeleton class="w-full h-full rounded-none bg-card/50" />
                <div class="absolute inset-0 bg-gradient-to-t from-background via-background/80 to-transparent"></div>
            </div>
            <div class="w-full px-4 md:px-12 relative z-20 space-y-12 -mt-16 md:-mt-24 max-w-7xl mx-auto">
                <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
                    <div class="lg:col-span-8 xl:col-span-9 space-y-8">
                        <Skeleton class="h-12 md:h-20 w-3/4 bg-muted rounded-lg" />
                        <div class="space-y-2 mt-4">
                            <Skeleton class="h-4 w-full bg-muted rounded" />
                            <Skeleton class="h-4 w-4/5 bg-muted rounded" />
                        </div>
                        <Skeleton class="h-[400px] w-full bg-muted rounded-xl mt-8" />
                    </div>
                    <div class="hidden lg:block lg:col-span-4 xl:col-span-3">
                        <Skeleton class="h-[300px] w-full bg-muted rounded-xl" />
                    </div>
                </div>
            </div>
        </div>

    {:then res}
        {@const content = res.data}

        <div in:fade={{ duration: 500 }} class="w-full">
            <ContentHero item={content} />

            <div class="w-full px-4 md:px-12 relative z-20 space-y-8 -mt-4 md:-mt-8 max-w-[1400px] mx-auto">

                <div class="lg:hidden pt-8">
                    <Drawer.Root>
                        <Drawer.Trigger>
                            <Button variant="secondary" class="w-full flex items-center justify-center gap-2 shadow-sm h-12 rounded-xl border border-border/50 bg-card/80 backdrop-blur-sm">
                                <Info class="w-5 h-5 text-primary" />
                                <span class="font-semibold text-foreground/90">View Info, Trackers & Tags</span>
                            </Button>
                        </Drawer.Trigger>
                        <Drawer.Content class="h-[85vh]">
                            <div class="p-6 overflow-y-auto">
                                <h2 class="font-bold text-xl mb-6">Details</h2>
                                <ContentSidebar metadata={content} trackers={content.trackerMappings || []} />
                            </div>
                        </Drawer.Content>
                    </Drawer.Root>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">

                    <div class="lg:col-span-8 xl:col-span-9 lg:pt-8">
                        <div class="flex flex-col gap-10 divide-y divide-border/60">

                            {#if (content.characters && content.characters.length > 0) || (content.staff && content.staff.length > 0)}
                                <section class="pt-10 first:pt-0">
                                    <CastAndStaff characters={content.characters || []} staff={content.staff || []} />
                                </section>
                            {/if}

                            {#if content.relations && content.relations.length > 0}
                                <section class="pt-10 first:pt-0">
                                    <RelationsTab relations={content.relations} />
                                </section>
                            {/if}

                            {#if content.contentType === 'anime'}
                                {#if content.subtype !== 'MOVIE'}
                                    <section class="pt-10 first:pt-0">
                                        <EpisodeSelector
                                                cid={content.cid}
                                                extensions={content.extensionSources || []}
                                                epsOrChapters={content.epsOrChapters}
                                                contentUnits={content.contentUnits || []}
                                        />
                                    </section>
                                {/if}
                            {:else}
                                <section class="pt-10 first:pt-0">
                                    {#await extensionsPromise}
                                        <Skeleton class="h-[300px] w-full bg-muted rounded-xl" />
                                    {:then extRes}
                                        <ChapterTable
                                                cid={content.cid}
                                                extensions={content.extensionSources || []}
                                                availableExtensions={extRes?.extensions || []}
                                        />
                                    {/await}
                                </section>
                            {/if}
                        </div>
                    </div>

                    <div class="hidden lg:block lg:col-span-4 xl:col-span-3 pt-4 md:pt-8">
                        <ContentSidebar metadata={content} trackers={content.trackerMappings || []} />
                    </div>
                </div>
            </div>
        </div>
    {:catch error}
        <div class="flex h-[85vh] flex-col items-center justify-center gap-4 text-muted-foreground">
            <p class="text-lg">Failed to load content.</p>
            <button class="text-white hover:underline transition-colors" onclick={() => location.reload()}>
                Try again
            </button>
        </div>
    {/await}
</main>