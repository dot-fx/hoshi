<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade, fly } from "svelte/transition";

    import { contentApi } from "@/api/content/content";
    import { i18n } from "@/i18n/index.svelte.js";
    import { primaryMetadata } from "@/api/content/types";
    import type { FullContent } from "@/api/content/types";
    import Sidebar from "@/components/content/Sidebar.svelte";
    import Episodes from "@/components/content/Episodes.svelte";
    import Chapters from "@/components/content/Chapters.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import RelationsTab from "@/components/content/Relations.svelte";
    import TrackerManager from "@/components/modals/TrackerManager.svelte";
    import ListEditor from '@/components/modals/ListEditor.svelte';
    import { layoutState } from '@/layout.svelte.js';
    import * as Tabs from "@/components/ui/tabs";
    import { Button } from "@/components/ui/button";
    import { Badge } from "@/components/ui/badge";
    import { Spinner } from "@/components/ui/spinner";
    import { Play, BookmarkPlus, Check, Plus, AlertCircle, BookOpen } from "lucide-svelte";
    import { listApi } from "@/api/list/list";
    import { appConfig } from "@/config.svelte.js";
    import { contentCache } from "@/contentCache.svelte.js";

    const source = $derived(page.params.source || "");
    const id = $derived(page.params.id || "");

    let showListModal = $state(false);
    let showTrackerModal = $state(false);

    let isLoading = $state(true);
    let error = $state<any>(null);
    let fullContent = $state<FullContent | null>(null);
    let isEntryLoading = $state(false);
    let hasEntry = $state(false);

    $effect(() => {
        layoutState.title = i18n.t('content.loading');
        layoutState.showBack = true;
        layoutState.backUrl = "/";

        if (source && id) {
            loadContent(source, id);
        }
    });

    async function loadContent(src: string, entryId: string) {
        isLoading = true;
        error = null;
        fullContent = null;

        try {
            //const cacheKey = `${src}:${entryId}`;
            //const cachedData = contentCache.get(cacheKey);

            const res =  await contentApi.get(src, entryId);

            //if (!cachedData) {
            //    contentCache.set(cacheKey, res);
            //}

            fullContent = res;
            console.log(res)

            const meta = primaryMetadata(res, appConfig.data?.content?.preferredMetadataProvider);
            if (meta) {
                const pref = appConfig.data?.ui?.titleLanguage || 'romaji';
                const title = meta.titleI18n?.[pref] || meta.title || '';
                layoutState.title = title.length > 35 ? title.slice(0, 35).trim() + '...' : title;
            }

            isEntryLoading = true;
            try {
                const listRes = await listApi.getEntry(res.content.cid);
                hasEntry = listRes.found;
            } catch {
                hasEntry = false;
            } finally {
                isEntryLoading = false;
            }

        } catch (e) {
            error = e;
            console.log(e)
            layoutState.title = i18n.t('errors.error');
        } finally {
            isLoading = false;
        }
    }

    function watchNow(contentData: FullContent) {
        const cid = contentData.content.cid;
        if (contentData.content.contentType === 'anime') {
            goto(`/watch/${cid}/1`);
        } else {
            // Lógica para manga/novelas
        }
    }

    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            anilist: 'anilist.co', myanimelist: 'myanimelist.net', mal: 'myanimelist.net',
            simkl: 'simkl.com', kitsu: 'kitsu.io', trakt: 'trakt.tv'
        };
        return `https://www.google.com/s2/favicons?domain=${domains[trackerName.toLowerCase()] || `${trackerName}.com`}&sz=64`;
    }
</script>

<svelte:head>
    {#if isLoading}
        <title>{i18n.t('content.loading')}</title>
    {:else if error}
        <title>Error</title>
    {:else if fullContent}
        {@const meta = primaryMetadata(fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const title = meta?.titleI18n?.[pref] || meta?.title || i18n.t('content.details')}
        <title>{title}</title>
    {/if}
</svelte:head>

<div class="min-h-screen bg-background pb-24">
    {#if isLoading}
        <div class="flex h-[85vh] w-full bg-background items-center justify-center">
            <Spinner class="w-10 h-10 text-muted-foreground/20" />
        </div>
    {:else if error}
        <div class="flex h-[85vh] flex-col items-center justify-center gap-4">
            <AlertCircle class="w-12 h-12 text-destructive opacity-20" />
            <p class="text-lg text-muted-foreground font-medium">{i18n.t(error?.key || 'errors.error')}</p>
            <Button variant="outline" class="rounded-full" onclick={() => loadContent(source, id)}>{i18n.t('content.retry')}</Button>
        </div>
    {:else if fullContent}
        {@const meta = primaryMetadata(fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const isMovie = meta?.subtype === 'MOVIE'}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}
        {@const score = meta?.rating ? Math.round(meta.rating * 10) : null}
        {@const isAdultContent = fullContent.content.nsfw ||
            meta?.genres?.some(g => ['hentai', 'adult'].includes(g.toLowerCase()))}
        {@const shouldBlur = isAdultContent && (appConfig.data?.general?.blurAdultContent ?? true)}

        <div class="absolute top-0 inset-x-0 w-full h-[60vh] md:h-[75vh] overflow-hidden pointer-events-none" in:fade={{ duration: 800 }}>
            <div class="absolute inset-0 w-full h-full" style="mask-image: linear-gradient(to bottom, black 40%, transparent 100%);">
                <img src={meta?.bannerImage || meta?.coverImage} alt="Background" class="w-full h-full object-cover opacity-15 md:opacity-25 {meta?.bannerImage && !shouldBlur ? '' : 'blur-xl scale-110'} {shouldBlur ? 'blur-3xl scale-125 opacity-10' : ''}" />
            </div>
            <div class="absolute inset-0 bg-linear-to-b from-background/40 via-background/10 to-transparent"></div>
        </div>

        <main class="relative z-10 w-full max-w-[2000px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 pt-16 md:pt-28 lg:pt-36" in:fade={{ delay: 200 }}>
            <div class="grid grid-cols-1 xl:grid-cols-[280px_1fr] gap-6 lg:gap-12 items-start">
                <div class="flex flex-col gap-6 z-20 h-fit self-start pb-10">
                    <div class="hidden lg:block w-full aspect-[2/2.8] rounded-2xl overflow-hidden shadow-2xl bg-muted relative group">
                        <img src={meta?.coverImage} alt="Cover" class="w-full h-full object-cover transition-transform duration-500 ease-out group-hover:scale-110 {shouldBlur ? 'blur-2xl' : ''}" />
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
                                <h1 class="text-xl sm:text-2xl font-black leading-tight line-clamp-2">{displayTitle}</h1>
                                {#if score}
                                    <Badge variant="outline" class="w-fit bg-green-500/10 text-green-500 font-bold">{score}%</Badge>
                                {/if}
                                <Button variant="secondary" class="w-fit h-9 rounded-full" onclick={() => showListModal = true}>
                                    {#if isEntryLoading} <Spinner class="w-3.5 h-3.5 mr-1.5" />
                                    {:else if hasEntry} <Check class="w-3.5 h-3.5 mr-1.5" /> {i18n.t('content.in_list')}
                                    {:else} <BookmarkPlus class="w-3.5 h-3.5 mr-1.5" /> {i18n.t('content.favorite')} {/if}
                                </Button>
                            </div>
                        </div>

                        <div class="hidden lg:flex flex-col gap-5">
                            <h1 class="text-5xl xl:text-6xl font-black drop-shadow-2xl leading-[1.1]">{displayTitle}</h1>
                            <div class="flex flex-wrap items-center gap-3 text-sm font-bold">
                                {#if score} <Badge class="bg-green-500/20 text-green-500 border-green-500/30">{score}% Rating</Badge> {/if}
                                {#if meta?.releaseDate} <span class="text-muted-foreground">{meta.releaseDate.split('-')[0]}</span> {/if}
                                {#if meta?.epsOrChapters}
                                    <span class="text-muted-foreground">•
                                        {fullContent.content.contentType === 'anime' ? i18n.t('content.eps_count', { count: meta.epsOrChapters }) : i18n.t('content.ch_count', { count: meta.epsOrChapters })}
                                    </span>
                                {/if}
                            </div>
                            {#if meta?.synopsis}
                                <p class="text-muted-foreground text-lg leading-relaxed max-w-3xl line-clamp-3">{@html meta.synopsis.replace(/<[^>]*>?/gm, '')}</p>
                            {/if}

                            <div class="flex flex-wrap items-center gap-3 pt-2">
                                <Button size="lg" onclick={() => watchNow(fullContent as FullContent)} class="rounded-full px-8 h-12 font-bold shadow-lg">
                                    {#if fullContent.content.contentType === 'anime'}
                                        <Play class="w-5 h-5 mr-2 fill-current" /> {i18n.t('content.watch_now')}
                                    {:else}
                                        <BookOpen class="w-5 h-5 mr-2 fill-current" /> {i18n.t('content.read_now')}
                                    {/if}
                                </Button>
                                <Button size="icon" variant="secondary" class="rounded-full w-12 h-12" onclick={() => showListModal = true}>
                                    {#if isEntryLoading} <Spinner class="w-5 h-5 text-muted-foreground" />
                                    {:else if hasEntry} <Check class="w-5 h-5 text-green-500" />
                                    {:else} <BookmarkPlus class="w-5 h-5" /> {/if}
                                </Button>
                                <div class="h-8 w-px bg-border/60 mx-1"></div>
                                {#each fullContent.trackerMappings as tracker}
                                    <a href={tracker.trackerUrl || '#'} target="_blank" class="w-12 h-12 rounded-full bg-card/80 border border-border/50 flex items-center justify-center hover:scale-105 transition-all">
                                        <img src={getTrackerFavicon(tracker.trackerName)} class="w-5 h-5" alt={tracker.trackerName} />
                                    </a>
                                {/each}
                                <button class="w-12 h-12 rounded-full bg-muted/30 border border-border/50 flex items-center justify-center hover:bg-muted" onclick={() => showTrackerModal = true}>
                                    <Plus class="w-5 h-5" />
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="w-full">
                        {#if isMovie}
                            <div class="space-y-12 pb-12">
                                <CastAndStaff characters={meta?.characters || []} staff={meta?.staff || []} />
                                {#if fullContent.relations.length > 0}
                                    <div class="pt-6 border-t border-border/20">
                                        <RelationsTab relations={fullContent.relations} />
                                    </div>
                                {/if}
                            </div>
                        {:else}
                            <Tabs.Root value="overview" class="w-full">
                                <Tabs.List class="w-full flex justify-start gap-10 border-b border-border/40 bg-transparent h-12 p-0 mb-8 overflow-x-auto hide-scrollbar">
                                    <Tabs.Trigger value="overview" class="h-full rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none text-muted-foreground font-bold text-base transition-all hover:text-foreground px-1 bg-transparent">{i18n.t('content.overview')}</Tabs.Trigger>
                                    <Tabs.Trigger value="episodes" class="h-full rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none text-muted-foreground font-bold text-base transition-all hover:text-foreground px-1 bg-transparent">
                                        {fullContent.content.contentType === 'anime' ? i18n.t('content.episodes') : i18n.t('content.chapters')}
                                    </Tabs.Trigger>
                                </Tabs.List>

                                <Tabs.Content value="overview" class="space-y-12 pb-12">
                                    <CastAndStaff characters={meta?.characters || []} staff={meta?.staff || []} />
                                    {#if fullContent.relations.length > 0}
                                        <div class="pt-6 border-t border-border/20">
                                            <RelationsTab relations={fullContent.relations} />
                                        </div>
                                    {/if}
                                </Tabs.Content>

                                <Tabs.Content value="episodes">
                                    {#if fullContent.content.contentType === 'anime'}
                                        <Episodes cid={fullContent.content.cid} epsOrChapters={meta?.epsOrChapters} contentUnits={fullContent.contentUnits} />
                                    {:else}
                                        <Chapters cid={fullContent.content.cid} contentType={fullContent.content.contentType} />
                                    {/if}
                                </Tabs.Content>
                            </Tabs.Root>
                        {/if}
                    </div>
                </div>
            </div>
        </main>

        <ListEditor bind:open={showListModal} cid={fullContent.content.cid} title={displayTitle} contentType={fullContent.content.contentType} coverImage={meta?.coverImage ?? undefined} />
        <TrackerManager bind:open={showTrackerModal} cid={fullContent.content.cid} trackers={fullContent.trackerMappings} />
    {/if}
</div>