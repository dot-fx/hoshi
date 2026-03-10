<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade } from "svelte/transition";

    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { i18n } from "$lib/i18n/index.svelte";

    import { primaryMetadata, type ContentWithMappings, type ContentMetadata } from "$lib/api/content/types";

    import ContentHero from "$lib/components/content/ContentHero.svelte";
    import ContentSidebar from "$lib/components/content/ContentSidebar.svelte";
    import EpisodeSelector from "$lib/components/content/EpisodeSelector.svelte";
    import ChapterTable from "$lib/components/content/ChapterTable.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import RelationsTab from "$lib/components/content/Relations.svelte";
    import ExtensionManagerModal from "$lib/components/content/ExtensionManagerModal.svelte";
    import TrackerCandidatesModal from "$lib/components/content/TrackerCandidatesModal.svelte";

    import { Skeleton } from "$lib/components/ui/skeleton";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Button } from "$lib/components/ui/button";
    import { Info, Loader2 } from "lucide-svelte";

    const cid = $derived(page.params.cid || "");

    let isResolving = $state(false);
    let showCandidatesModal = $state(false);
    let showExtensionsModal = $state(false);

    let stateCandidates = $derived((page.state as any)?.candidates || []);

    function withTimeout<T>(promise: Promise<T>, ms = 8000): Promise<T> {
        return Promise.race([
            promise,
            new Promise<T>((_, reject) =>
                setTimeout(() => reject(new Error("Timeout: La petición tardó demasiado")), ms)
            )
        ]);
    }

    const contentPromise = $derived(
        cid.startsWith("ext:")
            ? new Promise<ContentWithMappings>(() => {})
            : withTimeout(contentApi.get(cid), 8000)
    );

    const extensionsPromise = $derived(
        contentPromise.then(res => {
            if (!res) return [];

            const type = res.content.contentType;

            if (type === 'anime') return extensionsApi.getAnime();
            if (type === 'manga') return extensionsApi.getManga();
            return extensionsApi.getNovel();
        })
    );

    $effect(() => {
        if (cid.startsWith("ext:") && !isResolving) {
            isResolving = true;
            const [_, extName, extId] = cid.split(":");

            contentApi.resolveExtensionItem(extName, extId)
                .then(async res => {
                    const contentData = res.data;
                    const resolvedCid = contentData.content.cid;

                    if (!res.autoLinked && res.trackerCandidates && res.trackerCandidates.length > 0) {
                        await goto(`/content/${resolvedCid}`, {
                            replaceState: true,
                            state: { candidates: res.trackerCandidates }
                        });
                    } else {
                        await goto(`/content/${resolvedCid}`, { replaceState: true });
                    }

                    isResolving = false;
                })
                .catch(err => {
                    console.error("Resolution failed", err);
                    isResolving = false;
                });
        }
    });

    $effect(() => {
        if (stateCandidates.length > 0) {
            showCandidatesModal = true;
        }
    });

    function getSafeMeta(res: ContentWithMappings): ContentMetadata {
        return primaryMetadata(res) || ({} as ContentMetadata);
    }

    function buildHeroItem(res: ContentWithMappings) {
        const meta = getSafeMeta(res);
        return {
            ...meta,
            cid: res.content.cid,
            contentType: res.content.contentType
        };
    }
</script>

<svelte:head>
    {#await contentPromise}
        <title>{i18n.t('loading_content')}</title>
    {:then res}
        <title>{getSafeMeta(res).title || 'Details'}</title>
    {:catch e}
        <title>{i18n.t('error')}</title>
    {/await}
</svelte:head>

{#if cid.startsWith("ext:") || isResolving}
    <div class="min-h-screen bg-background flex flex-col items-center justify-center gap-4">
        <Loader2 class="w-12 h-12 text-primary animate-spin" />
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('importing_content')}</h2>
        <p class="text-muted-foreground">{i18n.t('setting_up_entry')}</p>
    </div>
{:else}
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
            {@const fullContent = res}

            <!-- Usamos las funciones auxiliares para que el HTML esté libre de TypeScript -->
            {@const meta = getSafeMeta(fullContent)}
            {@const heroItem = buildHeroItem(fullContent)}

            <div in:fade={{ duration: 500 }} class="w-full">
                <ContentHero item={heroItem} />

                <div class="w-full px-4 md:px-12 relative z-20 space-y-8 -mt-4 md:-mt-8 max-w-[1400px] mx-auto">
                    <div class="lg:hidden pt-8">
                        <Drawer.Root>
                            <Drawer.Trigger>
                                <Button variant="secondary" class="w-full flex items-center justify-center gap-2 shadow-sm h-12 rounded-xl border border-border/50 bg-card/80 backdrop-blur-sm">
                                    <Info class="w-5 h-5 text-primary" />
                                    <span class="font-semibold text-foreground/90">{i18n.t('view_info_trackers')}</span>
                                </Button>
                            </Drawer.Trigger>
                            <Drawer.Content class="h-[85vh]">
                                <div class="p-6 overflow-y-auto">
                                    <h2 class="font-bold text-xl mb-6">{i18n.t('details')}</h2>
                                    <ContentSidebar
                                            cid={fullContent.content.cid}
                                            metadata={meta || {}}
                                            trackers={fullContent.trackerMappings || []}
                                            extensions={fullContent.extensionSources || []}
                                    />                                </div>
                            </Drawer.Content>
                        </Drawer.Root>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
                        <div class="lg:col-span-8 xl:col-span-9 lg:pt-8">
                            <div class="flex flex-col gap-10 divide-y divide-border/60">

                                {#if (meta.characters?.length ?? 0 > 0) || (meta.staff?.length ?? 0 > 0)}
                                    <section class="pt-10 first:pt-0">
                                        <CastAndStaff
                                                characters={meta.characters || []}
                                                staff={meta.staff || []}
                                        />
                                    </section>
                                {/if}

                                {#if fullContent.relations && fullContent.relations.length > 0}
                                    <section class="pt-10 first:pt-0">
                                        <RelationsTab relations={fullContent.relations} />
                                    </section>
                                {/if}

                                {#if fullContent.content.contentType === 'anime'}
                                    {#if meta.subtype !== 'MOVIE'}
                                        <section class="pt-10 first:pt-0">
                                            <EpisodeSelector
                                                    cid={fullContent.content.cid}
                                                    extensions={fullContent.extensionSources}
                                                    epsOrChapters={meta.epsOrChapters}
                                                    contentUnits={fullContent.contentUnits}
                                            />
                                        </section>
                                    {/if}
                                {:else}
                                    <section class="pt-10 first:pt-0">
                                        {#await extensionsPromise}
                                            <Skeleton class="h-[300px] w-full bg-muted rounded-xl" />
                                        {:then extRes}
                                            <ChapterTable
                                                    cid={fullContent.content.cid}
                                                    contentType={fullContent.content.contentType}
                                                    availableExtensions={extRes || []}
                                            />
                                        {/await}
                                    </section>
                                {/if}
                            </div>
                        </div>

                        <div class="hidden lg:block lg:col-span-4 xl:col-span-3 pt-4 md:pt-8">
                            <ContentSidebar
                                    cid={fullContent.content.cid}
                                    metadata={meta || {} }
                                    trackers={fullContent.trackerMappings || []}
                                    extensions={fullContent.extensionSources || []}
                            />                        </div>
                    </div>
                </div>
            </div>
        {:catch error}
            <div class="flex h-[85vh] flex-col items-center justify-center gap-4 text-muted-foreground">
                <p class="text-lg">{i18n.t('failed_load_content')}</p>
                <button class="text-white hover:underline transition-colors" onclick={() => location.reload()}>
                    {i18n.t('retry')}
                </button>
            </div>
        {/await}
    </main>

    <TrackerCandidatesModal bind:open={showCandidatesModal} {cid} candidates={stateCandidates} />
{/if}