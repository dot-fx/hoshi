<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade, fly } from "svelte/transition";

    import { contentApi } from "$lib/api/content/content";
    import { i18n } from "$lib/i18n/index.svelte";
    import { primaryMetadata } from "$lib/api/content/types";
    import Sidebar from "$lib/components/content/Sidebar.svelte";
    import Episodes from "@/components/content/Episodes.svelte";
    import Chapters from "@/components/content/Chapters.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import RelationsTab from "$lib/components/content/Relations.svelte";
    import TrackerCandidates from "@/components/modals/TrackerCandidates.svelte";
    import TrackerManager from "@/components/modals/TrackerManager.svelte";
    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { layoutState } from '@/layout.svelte.js';
    import * as Tabs from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Spinner } from "$lib/components/ui/spinner";
    import { Play, BookmarkPlus, Check, Plus, AlertCircle, BookOpen } from "lucide-svelte";
    import { listApi } from "@/api/list/list";
    import { appConfig } from "@/config.svelte";
    import {contentCache} from "@/contentCache.svelte";

    const cid = $derived(page.params.cid || "");
    let isResolving = $state(false);
    let showCandidatesModal = $state(false);
    let showListModal = $state(false);
    let showTrackerModal = $state(false);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);
    let stateCandidates = $derived((page.state as any)?.candidates || []);

    function withTimeout<T>(promise: Promise<T>, ms = 8000): Promise<T> {
        return Promise.race([
            promise,
            new Promise<T>((_, reject) => setTimeout(() => reject({ key: 'errors.timeout' }), ms))
        ]);
    }

    const contentPromise = $derived.by(() => {
        if (cid.startsWith("ext:") || !cid) return null;

        const cachedData = contentCache.get(cid);
        if (cachedData) {
            return Promise.resolve(cachedData)
        }

        return withTimeout(
            contentApi.get(cid).then(res => {
                contentCache.set(cid, res);
                return res;
            }),
            8000
        );
    });

    $effect(() => {
        if (cid.startsWith("ext:") && !isResolving) {
            isResolving = true;
            const [_, extName, extId] = cid.split(":");
            contentApi.resolveExtensionItem(extName, extId).then(async res => {
                const resolvedCid = res.data.content.cid;
                if (!res.autoLinked && res.trackerCandidates && res.trackerCandidates.length > 0) {
                    await goto(`/content/${resolvedCid}`, { replaceState: true, state: { candidates: res.trackerCandidates } });
                } else {
                    await goto(`/content/${resolvedCid}`, { replaceState: true });
                }
                isResolving = false;
            }).catch(() => isResolving = false);
        }
    });

    $effect(() => {
        if (!cid.startsWith("ext:") && cid !== "") {
            isEntryLoading = true;
            listApi.getEntry(cid).then(res => hasEntry = res.found).catch(() => hasEntry = false).finally(() => isEntryLoading = false);
        }
    });

    $effect(() => {
        layoutState.title = i18n.t('content.loading');
        layoutState.showBack = true;
        layoutState.backUrl = "/";

        if (contentPromise) {
            contentPromise.then(res => {
                const meta = primaryMetadata(res, appConfig.data?.content?.preferredMetadataProvider);
                if (meta) {
                    const pref = appConfig.data?.ui?.titleLanguage || 'romaji';
                    const title = meta.titleI18n?.[pref] || meta.title || '';
                    layoutState.title = title.length > 35
                        ? title.slice(0, 35).trim() + '...'
                        : title;
                }
            }).catch(() => {
                layoutState.title = i18n.t('errors.error');
            });
        }
    });

    function watchNow(fullContent) {
        if (fullContent.content.contentType === 'anime') {
            goto(`/watch/${cid}/1`);
        } else {

        }
    }

    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            anilist: 'anilist.co', myanimelist: 'myanimelist.net', mal: 'myanimelist.net', simkl: 'simkl.com', kitsu: 'kitsu.io', trakt: 'trakt.tv', trakttvslug: 'trakt.tv', animeplanet: 'anime-planet.com', imdb: 'imdb.com', tmdb: 'themoviedb.org', tvdb: 'thetvdb.com'
        };
        return `https://www.google.com/s2/favicons?domain=${domains[trackerName.toLowerCase()] || `${trackerName}.com`}&sz=64`;
    }
</script>

<svelte:head>
    {#if contentPromise}
        {#await contentPromise}
            <title>{i18n.t('content.loading')}</title>
        {:then res}
            {@const meta = primaryMetadata(res, appConfig.data?.content?.preferredMetadataProvider)}
            {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
            {@const title = meta?.titleI18n?.[pref] || meta?.title || i18n.t('content.details')}
            <title>{title}</title>
        {:catch e}
            <title>error</title>
        {/await}
    {:else}
        <title>{i18n.t('content.importing')}</title>
    {/if}
</svelte:head>

{#if cid.startsWith("ext:") || isResolving || !contentPromise}
    <div class="min-h-screen bg-background flex flex-col items-center justify-center gap-6" in:fade>
        <div class="relative flex items-center justify-center">
            <Spinner class="w-14 h-14 text-primary" />
            <div class="absolute w-2 h-2 bg-primary rounded-full animate-ping"></div>
        </div>
        <div class="space-y-2 text-center">
            <h2 class="text-2xl font-black tracking-tighter uppercase italic">{i18n.t('content.importing')}</h2>
        </div>
    </div>
{:else}
    <div class="min-h-screen bg-background pb-24">
        {#await contentPromise}
            <div class="w-full h-screen bg-background flex flex-col items-center justify-center">
                <Spinner class="w-10 h-10 text-muted-foreground/20" />
            </div>
        {:then fullContent}
            {@const meta = primaryMetadata(fullContent, appConfig.data?.content?.preferredMetadataProvider)}
            {@const isMovie = meta?.subtype === 'MOVIE'}
            {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
            {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}

            {@const score = meta?.rating ? Math.round(meta.rating * 10) : null}
            {@const isExplicitlyNsfw = fullContent.content.nsfw}
            {@const hasAdultGenre = meta?.genres?.some(g => ['hentai', 'adult'].includes(g.toLowerCase())) ?? false}
            {@const isAdultContent = isExplicitlyNsfw || hasAdultGenre}
            {@const shouldBlur = isAdultContent && (appConfig.data?.general?.blurAdultContent ?? true)}

            <div class="absolute top-0 inset-x-0 w-full h-[60vh] md:h-[75vh] overflow-hidden pointer-events-none" in:fade={{ duration: 800 }}>
                <div class="absolute inset-0 w-full h-full" style="mask-image: linear-gradient(to bottom, black 40%, transparent 100%); -webkit-mask-image: linear-gradient(to bottom, black 40%, transparent 100%);">
                    <img src={meta?.bannerImage || meta?.coverImage} alt="Background" class="w-full h-full object-cover opacity-15 md:opacity-25 {meta?.bannerImage && !shouldBlur ? '' : 'blur-xl scale-110'} {shouldBlur ? 'blur-3xl scale-125 opacity-10' : ''}" />
                </div>
                <div class="absolute inset-0 bg-linear-to-b from-background/40 via-background/10 to-transparent"></div>
                <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,var(--tw-gradient-stops))] from-black/20 via-transparent to-transparent"></div>
            </div>

            <main class="relative z-10 w-full max-w-[2000px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 pt-16 md:pt-28 lg:pt-36" in:fade={{ delay: 200 }}>
                <div class="grid grid-cols-1 xl:grid-cols-[280px_1fr] gap-6 lg:gap-12 items-start">
                    <div class="flex flex-col gap-6 z-20 h-fit self-start pb-10">

                        <div class="hidden lg:block w-full aspect-[2/2.8] rounded-2xl overflow-hidden shadow-2xl bg-muted relative group">
                            <img
                                    src={meta?.coverImage}
                                    alt="Cover"
                                    class="w-full h-full object-cover transition-transform duration-500 ease-out group-hover:scale-110 {shouldBlur ? 'blur-2xl' : ''}"
                            />
                        </div>

                        <div class="hidden lg:block">
                            <Sidebar cid={fullContent.content.cid} metadata={meta} extensions={fullContent.extensionSources} />
                        </div>
                    </div>

                    <div class="flex flex-col w-full min-w-0">
                        <div class="pt-4 mb-10">
                            <div class="flex gap-4 lg:hidden mb-6 items-start">
                                <div class="w-24 sm:w-32 shrink-0 rounded-xl overflow-hidden shadow-xl bg-muted relative">
                                    <img src={meta?.coverImage} alt="Cover" class="w-full aspect-[2/3] object-cover {shouldBlur ? 'blur-2xl scale-110' : ''}" />
                                </div>
                                <div class="flex flex-col flex-1 py-0.5 gap-2.5">
                                    <h1 class="text-xl sm:text-2xl font-black leading-tight line-clamp-2 md:line-clamp-3">{displayTitle}</h1>
                                    {#if score}
                                        <Badge variant="outline" class="w-fit bg-green-500/10 text-green-500 font-bold border-green-500/20 px-2.5 py-1">
                                            {i18n.t('content.score', { score: score })}
                                        </Badge>
                                    {/if}
                                    <Button variant="secondary" class="w-fit h-9 rounded-full px-4 text-xs font-bold bg-primary/10 text-primary border border-primary/20" onclick={() => showListModal = true}>
                                        {#if hasEntry} <Check class="w-3.5 h-3.5 mr-1.5" /> {i18n.t('content.in_list')}
                                        {:else} <BookmarkPlus class="w-3.5 h-3.5 mr-1.5" /> {i18n.t('content.favorite')} {/if}
                                    </Button>
                                </div>
                            </div>

                            <div class="hidden lg:flex flex-col gap-5">
                                <h1 class="text-5xl xl:text-6xl font-black drop-shadow-2xl leading-[1.1]">{displayTitle}</h1>

                                <div class="flex flex-wrap items-center gap-3 text-sm font-bold">
                                    {#if score} <Badge class="bg-green-500/20 text-green-500 hover:bg-green-500/30 border-green-500/30 border">{score}% Rating</Badge> {/if}
                                    {#if meta?.releaseDate} <span class="text-muted-foreground font-semibold">{meta.releaseDate.split('-')[0]}</span> {/if}
                                    {#if meta?.epsOrChapters} <span class="text-muted-foreground font-semibold">• {fullContent.content.contentType === 'anime' ? i18n.t('content.eps_count', { count: meta.epsOrChapters }) : i18n.t('content.ch_count', { count: meta.epsOrChapters })}</span> {/if}
                                </div>

                                {#if meta?.synopsis}
                                    <p class="text-muted-foreground text-lg leading-relaxed max-w-3xl line-clamp-3">{@html meta.synopsis.replace(/<[^>]*>?/gm, '')}</p>
                                {/if}

                                <div class="flex flex-wrap items-center gap-3 pt-2">
                                    <Button size="lg" onclick={() => watchNow(fullContent)} class="rounded-full px-8 h-12 font-bold bg-primary text-primary-foreground text-base shadow-lg hover:scale-105 transition-transform">
                                        {#if fullContent.content.contentType === 'anime'}
                                            <Play class="w-5 h-5 mr-2 fill-current" /> {i18n.t('content.watch_now')}
                                        {:else}
                                            <BookOpen class="w-5 h-5 mr-2 fill-current" /> {i18n.t('content.read_now')}
                                        {/if}
                                    </Button>
                                    <Button size="icon" variant="secondary" class="rounded-full w-12 h-12 bg-secondary/80 backdrop-blur-md shadow-lg border border-border/50" onclick={() => showListModal = true} title={i18n.t('list.add_to_list')}>
                                        {#if hasEntry} <Check class="w-5 h-5 text-green-500" /> {:else} <BookmarkPlus class="w-5 h-5 text-foreground" /> {/if}
                                    </Button>
                                    <div class="h-8 w-px bg-border/60 mx-1"></div>
                                    {#each fullContent.trackerMappings as tracker}
                                        <a href={tracker.trackerUrl || '#'} target="_blank" class="w-12 h-12 rounded-full bg-card/80 backdrop-blur-md border border-border/50 shadow-lg flex items-center justify-center hover:scale-105 hover:border-primary/50 transition-all" title={tracker.trackerName}>
                                            <img src={getTrackerFavicon(tracker.trackerName)} class="w-5 h-5 rounded-sm" alt={tracker.trackerName} />
                                        </a>
                                    {/each}
                                    <button class="w-12 h-12 rounded-full bg-muted/30 backdrop-blur-md border border-border/50 shadow-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition-all" onclick={() => showTrackerModal = true} title={i18n.t('content.manage_trackers')}>
                                        <Plus class="w-5 h-5" />
                                    </button>
                                </div>
                            </div>
                        </div>
                        <div class="w-full">
                            {#if isMovie}
                                <div class="space-y-12 pb-12">
                                    {#if (meta?.characters && meta.characters.length > 0) || (meta?.staff && meta.staff.length > 0)}
                                        <div in:fly={{ y: 20, delay: 100 }}>
                                            <CastAndStaff characters={meta.characters || []} staff={meta.staff || []} />
                                        </div>
                                    {/if}

                                    {#if fullContent.relations && fullContent.relations.length > 0}
                                        <div class="pt-6 border-t border-border/20" in:fly={{ y: 20, delay: 150 }}>
                                            <RelationsTab relations={fullContent.relations} />
                                        </div>
                                    {/if}
                                </div>
                            {:else}
                                <Tabs.Root value="overview" class="w-full">
                                    <Tabs.List class="w-full flex justify-start gap-10 border-b border-border/40 bg-transparent h-12 p-0 mb-8 overflow-x-auto hide-scrollbar">
                                        <Tabs.Trigger value="overview" class="h-full rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none text-muted-foreground font-bold text-base transition-all hover:text-foreground px-1 bg-transparent">
                                            {i18n.t('content.overview')}
                                        </Tabs.Trigger>
                                        <Tabs.Trigger value="episodes" class="h-full rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none text-muted-foreground font-bold text-base transition-all hover:text-foreground px-1 bg-transparent">
                                            {fullContent.content.contentType === 'anime' ? i18n.t('content.episodes') : i18n.t('content.chapters')}
                                        </Tabs.Trigger>
                                    </Tabs.List>

                                    <Tabs.Content value="overview" class="outline-none space-y-12 pb-12">
                                        {#if (meta?.characters && meta.characters.length > 0) || (meta?.staff && meta.staff.length > 0)}
                                            <div in:fly={{ y: 20, delay: 100 }}>
                                                <CastAndStaff characters={meta.characters || []} staff={meta.staff || []} />
                                            </div>
                                        {/if}

                                        {#if fullContent.relations && fullContent.relations.length > 0}
                                            <div class="pt-6 border-t border-border/20" in:fly={{ y: 20, delay: 150 }}>
                                                <RelationsTab relations={fullContent.relations} />
                                            </div>
                                        {/if}
                                    </Tabs.Content>

                                    <Tabs.Content value="episodes" class="outline-none space-y-8">
                                        {#if fullContent.content.contentType === 'anime'}
                                            <Episodes cid={fullContent.content.cid} epsOrChapters={meta?.epsOrChapters} contentUnits={fullContent.contentUnits} />
                                        {:else}
                                            <Chapters cid={fullContent.content.cid} contentType={fullContent.content.contentType} />
                                        {/if}
                                    </Tabs.Content>
                                </Tabs.Root>
                            {/if}

                            <div class="lg:hidden mt-12 pt-12 border-t border-border/20">
                                <h3 class="text-xl font-bold tracking-tight mb-6">{i18n.t('content.information')}</h3>
                                <Sidebar cid={fullContent.content.cid} metadata={meta} extensions={fullContent.extensionSources} />
                            </div>
                        </div>
                    </div>
                </div>
            </main>

            <ListEditor bind:open={showListModal} cid={fullContent.content.cid} title={displayTitle} contentType={fullContent.content.contentType} coverImage={meta?.coverImage ?? undefined} />
            <TrackerManager bind:open={showTrackerModal} cid={fullContent.content.cid} trackers={fullContent.trackerMappings} />
            <TrackerCandidates bind:open={showCandidatesModal} cid={fullContent.content.cid} candidates={stateCandidates} />

        {:catch error}
            <div class="flex h-[85vh] flex-col items-center justify-center gap-4" in:fade>
                <AlertCircle class="w-12 h-12 text-destructive opacity-20" />

                <p class="text-lg text-muted-foreground font-medium">
                    {i18n.t(error?.key)}
                </p>

                <Button variant="outline" class="rounded-full font-bold px-6" onclick={() => location.reload()}>
                    {i18n.t('content.retry')}
                </Button>
            </div>
        {/await}
    </div>
{/if}