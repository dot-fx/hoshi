<script lang="ts">
    import { page } from "$app/state";
    import { contentApi } from "$lib/api/content/content";
    import { fade } from "svelte/transition";
    import { tick } from "svelte";

    import AnimeHero from "$lib/components/anime/AnimeHero.svelte";
    import AnimeSidebar from "$lib/components/anime/AnimeSidebar.svelte";
    import EpisodeSelector from "$lib/components/anime/EpisodeSelector.svelte";
    import MappingsPanel from "$lib/components/anime/MappingsPanel.svelte";
    import CastAndStaffTab from "$lib/components/anime/CastAndStaffTab.svelte";
    import RelationsTab from "$lib/components/anime/RelationsTab.svelte";

    import * as Tabs from "$lib/components/ui/tabs";
    import { Skeleton } from "$lib/components/ui/skeleton";

    const cid = $derived(page.params.cid);
    const contentPromise = $derived(contentApi.get(cid || ""));

    let activeTab = $state("overview");
    let indicatorWidth = $state(0);
    let indicatorLeft = $state(0);

    $effect(() => {
        const _tab = activeTab;

        const updateIndicator = async () => {
            await tick();
            const list = document.getElementById("anime-tabs-list");
            if (list) {
                const activeEl = list.querySelector('[data-state="active"]') as HTMLElement;
                if (activeEl) {
                    indicatorWidth = activeEl.offsetWidth;
                    indicatorLeft = activeEl.offsetLeft;
                }
            }
        };

        updateIndicator();

        window.addEventListener('resize', updateIndicator);
        return () => window.removeEventListener('resize', updateIndicator);
    });
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
                            <Skeleton class="h-4 w-2/3 bg-muted rounded" />
                        </div>
                        <Skeleton class="h-[400px] w-full bg-muted rounded-xl mt-8" />
                    </div>
                    <div class="lg:col-span-4 xl:col-span-3">
                        <Skeleton class="h-[300px] w-full bg-muted rounded-xl" />
                    </div>
                </div>
            </div>
        </div>

    {:then res}
        {@const content = res.data}

        <div in:fade={{ duration: 500 }} class="w-full">
            <AnimeHero metadata={content} />

            <div class="w-full px-4 md:px-12 relative z-20 space-y-8 -mt-4 md:-mt-8 max-w-[1400px] mx-auto">
                <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
                    <div class="lg:col-span-8 xl:col-span-9 space-y-8">

                        <Tabs.Root bind:value={activeTab} class="w-full">

                            <Tabs.List
                                    id="anime-tabs-list"
                                    class="relative w-full flex justify-start bg-transparent border-b border-border/40 rounded-none p-0 h-auto overflow-x-auto flex-nowrap hide-scrollbar gap-2"
                            >
                                <Tabs.Trigger
                                        value="overview"
                                        class="relative z-10 text-base font-medium whitespace-nowrap bg-transparent px-4 py-3 transition-colors duration-300 text-muted-foreground hover:text-foreground data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none outline-none border-none ring-0 focus-visible:ring-0"
                                >
                                    Overview
                                </Tabs.Trigger>

                                <Tabs.Trigger
                                        value="sources"
                                        class="relative z-10 text-base font-medium whitespace-nowrap bg-transparent px-4 py-3 transition-colors duration-300 text-muted-foreground hover:text-foreground data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none outline-none border-none ring-0 focus-visible:ring-0"
                                >
                                    Video Sources
                                </Tabs.Trigger>

                                <div
                                        class="absolute bottom-0 h-[2px] bg-primary transition-all duration-300 ease-out z-20"
                                        style="width: {indicatorWidth}px; transform: translateX({indicatorLeft}px);"
                                ></div>
                            </Tabs.List>

                            <Tabs.Content value="overview" class="pt-8 outline-none">
                                <div class="flex flex-col gap-10 divide-y divide-border/60">
                                    {#if (content.characters && content.characters.length > 0) || (content.staff && content.staff.length > 0)}
                                        <section class="pt-10 first:pt-0">
                                            <CastAndStaffTab characters={content.characters || []} staff={content.staff || []} />
                                        </section>
                                    {/if}

                                    {#if content.relations && content.relations.length > 0}
                                        <section class="pt-10 first:pt-0">
                                            <RelationsTab relations={content.relations} />
                                        </section>
                                    {/if}

                                    {#if content.subtype !== 'MOVIE'}
                                        <section class="pt-10 first:pt-0">
                                            <EpisodeSelector
                                            cid={content.cid}
                                            extensions={content.extensionSources || []}
                                            epsOrChapters={content.epsOrChapters}
                                            contentUnits={content.contentUnits || []} />
                                        </section>
                                    {/if}
                                </div>
                            </Tabs.Content>

                            <Tabs.Content value="sources" class="pt-6 outline-none">
                                <MappingsPanel extensions={content.extensionSources || []} />
                            </Tabs.Content>
                        </Tabs.Root>
                    </div>

                    <div class="lg:col-span-4 xl:col-span-3">
                        <AnimeSidebar metadata={content} trackers={content.trackerMappings || []} />
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