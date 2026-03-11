<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade, fly } from "svelte/transition";

    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { i18n } from "$lib/i18n/index.svelte";
    import { primaryMetadata, type ContentWithMappings } from "$lib/api/content/types";

    import ContentSidebar from "$lib/components/content/ContentSidebar.svelte";
    import EpisodeSelector from "$lib/components/content/EpisodeSelector.svelte";
    import ChapterTable from "$lib/components/content/ChapterTable.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import RelationsTab from "$lib/components/content/Relations.svelte";
    import TrackerCandidatesModal from "$lib/components/content/TrackerCandidatesModal.svelte";
    import TrackerManagerModal from "$lib/components/content/TrackerManagerModal.svelte";
    import ListEditorModal from '$lib/components/ListEditorModal.svelte';

    import { Skeleton } from "$lib/components/ui/skeleton";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Loader2, Play, BookmarkPlus, Check, Plus } from "lucide-svelte";
    import { listApi } from "@/api/list/list";

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
            new Promise<T>((_, reject) => setTimeout(() => reject(new Error("Timeout")), ms))
        ]);
    }

    const mockContent: ContentWithMappings = {
        content: { cid: "loading", contentType: "anime", nsfw: false, createdAt: 0, updatedAt: 0 },
        metadata: [{ cid: "loading", sourceName: "mock", title: "Loading...", characters: [], staff: [], externalIds: {}, createdAt: 0, updatedAt: 0 }],
        trackerMappings: [],
        extensionSources: [],
        relations: [],
        contentUnits: []
    };

    const contentPromise = $derived(
        cid.startsWith("ext:") ? Promise.resolve(mockContent) : withTimeout(contentApi.get(cid), 8000)
    );

    const extensionsPromise = $derived(
        contentPromise.then(res => {
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
        if (!cid.startsWith("ext:")) {
            isEntryLoading = true;
            listApi.getEntry(cid).then(res => hasEntry = res.found).catch(() => hasEntry = false).finally(() => isEntryLoading = false);
        }
    });

    const formatType = (type: string | undefined | null) => {
        if (!type) return '';
        if (type === 'TV') return i18n.t('series');
        return type.replace('_', ' ').toUpperCase();
    };

    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            anilist: 'anilist.co', myanimelist: 'myanimelist.net', mal: 'myanimelist.net', simkl: 'simkl.com', kitsu: 'kitsu.io', trakt: 'trakt.tv', trakttvslug: 'trakt.tv', animeplanet: 'anime-planet.com', imdb: 'imdb.com', tmdb: 'themoviedb.org', tvdb: 'thetvdb.com'
        };
        return `https://www.google.com/s2/favicons?domain=${domains[trackerName.toLowerCase()] || `${trackerName}.com`}&sz=64`;
    }
</script>

<svelte:head>
    {#await contentPromise}
        <title>{i18n.t('loading_content')}</title>
    {:then res}
        <title>{primaryMetadata(res)?.title || 'Details'}</title>
    {:catch e}
        <title>{i18n.t('error')}</title>
    {/await}
</svelte:head>

{#if cid.startsWith("ext:") || isResolving}
    <div class="min-h-screen bg-background flex flex-col items-center justify-center gap-4">
        <Loader2 class="w-12 h-12 text-primary animate-spin" />
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('importing_content')}</h2>
    </div>
{:else}
    <div class="min-h-screen bg-background pb-24">
        {#await contentPromise}
            <div class="w-full h-[50vh] bg-muted animate-pulse"></div>
        {:then fullContent}
            {@const meta = primaryMetadata(fullContent)}
            {@const score = meta?.rating ? Math.round(meta.rating * 10) : null}

            <!-- 1. BACKGROUND INMERSIVO FULL BLEED -->
            <div class="absolute top-0 inset-x-0 w-full h-[60vh] md:h-[80vh] overflow-hidden pointer-events-none" in:fade={{ duration: 800 }}>
                <img src={meta?.bannerImage || meta?.coverImage} alt="Background" class="w-full h-full object-cover opacity-20 md:opacity-30 {meta?.bannerImage ? '' : 'blur-xl scale-110'}" />
                <div class="absolute inset-0 bg-gradient-to-t from-background via-background/80 to-transparent"></div>
                <div class="absolute inset-0 bg-gradient-to-r from-background via-background/40 to-transparent hidden md:block"></div>
            </div>

            <!-- 2. CONTENEDOR PRINCIPAL -->
            <main class="relative z-10 w-full max-w-[1500px] mx-auto px-4 md:px-8 lg:px-12 pt-16 md:pt-32 lg:pt-48">

                <div class="grid grid-cols-1 lg:grid-cols-[280px_1fr] xl:grid-cols-[320px_1fr] gap-6 lg:gap-12 items-start">

                    <!-- COLUMNA IZQUIERDA -->
                    <div class="flex flex-col gap-6 lg:sticky lg:top-24 z-20">

                        <!-- HEADER MÓVIL (Póster + Título) -->
                        <div class="flex gap-4 lg:hidden mb-6 items-start" in:fly={{ y: 10, duration: 600 }}>
                            <div class="w-24 sm:w-32 shrink-0 rounded-xl overflow-hidden shadow-xl border border-border/50">
                                <img src={meta?.coverImage} alt="Cover" class="w-full aspect-[2/3] object-cover" />
                            </div>
                            <div class="flex flex-col flex-1 py-0.5 gap-2.5">
                                <h1 class="text-xl sm:text-2xl font-black leading-tight line-clamp-2 md:line-clamp-3">{meta?.title}</h1>

                                {#if score}
                                    <Badge variant="outline" class="w-fit bg-green-500/10 text-green-500 font-bold border-green-500/20 px-2.5 py-1">
                                        {score}% Rating
                                    </Badge>
                                {/if}

                                <Button variant="secondary" class="w-fit h-9 rounded-full px-4 text-xs font-bold bg-primary/10 text-primary border border-primary/20" onclick={() => showListModal = true}>
                                    {#if hasEntry} <Check class="w-3.5 h-3.5 mr-1.5" /> In List
                                    {:else} <BookmarkPlus class="w-3.5 h-3.5 mr-1.5" /> Favorite {/if}
                                </Button>
                            </div>
                        </div>

                        <!-- PÓSTER ESCRITORIO -->
                        <div class="hidden lg:block w-full aspect-[2/3] rounded-2xl overflow-hidden shadow-[0_20px_50px_rgba(0,0,0,0.5)] border border-white/10" in:fly={{ y: 20, duration: 600 }}>
                            <img src={meta?.coverImage} alt="Cover" class="w-full h-full object-cover" />
                        </div>

                        <!-- TARJETA DE METADATOS RÁPIDOS -->
                        <div class="hidden lg:block bg-muted/30 rounded-2xl p-5 border border-border/40 backdrop-blur-sm" in:fly={{ y: 20, duration: 600, delay: 100 }}>
                            <div class="grid grid-cols-[80px_1fr] gap-y-3 text-sm">
                                <span class="text-muted-foreground font-medium">Format</span>
                                <span class="font-semibold text-foreground truncate">{formatType(meta?.subtype || fullContent.content.contentType)}</span>

                                <span class="text-muted-foreground font-medium">Status</span>
                                <span class="font-semibold {meta?.status?.toLowerCase() === 'releasing' ? 'text-green-500' : 'text-foreground'} truncate">{meta?.status || 'TBA'}</span>

                                {#if meta?.epsOrChapters}
                                    <span class="text-muted-foreground font-medium">{fullContent.content.contentType === 'anime' ? 'Eps' : 'Ch'}</span>
                                    <span class="font-semibold text-foreground">{meta.epsOrChapters}</span>
                                {/if}

                                {#if meta?.studio}
                                    <span class="text-muted-foreground font-medium">Studio</span>
                                    <span class="font-semibold text-foreground truncate" title={meta.studio}>{meta.studio}</span>
                                {/if}

                                {#if meta?.releaseDate}
                                    <span class="text-muted-foreground font-medium">Aired</span>
                                    <span class="font-semibold text-foreground truncate">{new Date(meta.releaseDate).toLocaleDateString(i18n.locale || 'en-US', { year: 'numeric', month: 'short' })}</span>
                                {/if}

                                {#if meta?.nsfw}
                                    <span class="text-muted-foreground font-medium">Rating</span>
                                    <span class="font-black text-destructive">18+ (NSFW)</span>
                                {/if}
                            </div>
                        </div>

                        <!-- Sidebar Original para Escritorio -->
                        <div class="hidden lg:block">
                            <ContentSidebar cid={fullContent.content.cid} metadata={meta} extensions={fullContent.extensionSources} />
                        </div>
                    </div>

                    <!-- COLUMNA DERECHA -->
                    <div class="flex flex-col w-full min-w-0">

                        <!-- HEADER ESCRITORIO -->
                        <div class="hidden lg:flex flex-col gap-5 mb-10 pt-4" in:fly={{ y: 20, duration: 600, delay: 200 }}>
                            <h1 class="text-5xl xl:text-6xl font-black drop-shadow-2xl leading-[1.1]">{meta?.title}</h1>

                            <div class="flex flex-wrap items-center gap-3 text-sm font-bold">
                                {#if score} <Badge class="bg-green-500/20 text-green-500 hover:bg-green-500/30 border-green-500/30 border">{score}% Rating</Badge> {/if}
                                {#if meta?.releaseDate} <span class="text-muted-foreground font-semibold">{meta.releaseDate.split('-')[0]}</span> {/if}
                                {#if meta?.epsOrChapters} <span class="text-muted-foreground font-semibold">• {meta.epsOrChapters} eps</span> {/if}
                            </div>

                            {#if meta?.synopsis}
                                <p class="text-muted-foreground text-lg leading-relaxed max-w-3xl line-clamp-3">{@html meta.synopsis.replace(/<[^>]*>?/gm, '')}</p>
                            {/if}

                            <!-- BOTONES Y TRACKERS -->
                            <div class="flex flex-wrap items-center gap-3 pt-2">
                                <Button size="lg" class="rounded-full px-8 h-12 font-bold bg-primary text-primary-foreground text-base shadow-lg hover:scale-105 transition-transform"><Play class="w-5 h-5 mr-2 fill-current" /> Watch Now</Button>
                                <Button size="icon" variant="secondary" class="rounded-full w-12 h-12 bg-secondary/80 backdrop-blur-md shadow-lg border border-border/50" onclick={() => showListModal = true} title="Add to List">
                                    {#if hasEntry} <Check class="w-5 h-5 text-green-500" /> {:else} <BookmarkPlus class="w-5 h-5 text-foreground" /> {/if}
                                </Button>
                                <div class="h-8 w-px bg-border/60 mx-1"></div>
                                {#each fullContent.trackerMappings as tracker}
                                    <a href={tracker.trackerUrl || '#'} target="_blank" class="w-12 h-12 rounded-full bg-card/80 backdrop-blur-md border border-border/50 shadow-lg flex items-center justify-center hover:scale-105 hover:border-primary/50 transition-all" title={tracker.trackerName}><img src={getTrackerFavicon(tracker.trackerName)} class="w-5 h-5 rounded-sm" alt={tracker.trackerName} /></a>
                                {/each}
                                <button class="w-12 h-12 rounded-full bg-muted/30 backdrop-blur-md border border-border/50 shadow-lg flex items-center justify-center hover:bg-muted text-muted-foreground hover:text-foreground transition-all" onclick={() => showTrackerModal = true} title="Manage Trackers"><Plus class="w-5 h-5" /></button>
                            </div>
                        </div>

                        <!-- SINOPSIS Y TRACKERS MÓVIL -->
                        <div class="lg:hidden space-y-6 mb-8" in:fly={{ y: 20, duration: 600, delay: 200 }}>
                            <div class="space-y-4">
                                <p class="text-sm text-muted-foreground leading-relaxed line-clamp-5">{@html meta?.synopsis?.replace(/<[^>]*>?/gm, '') || 'No description available.'}</p>
                            </div>

                            <div class="flex flex-wrap items-center gap-2.5 pt-2 border-t border-border/20">
                                <h3 class="text-xs font-bold uppercase tracking-wider text-muted-foreground mr-1">Trackers</h3>
                                {#each fullContent.trackerMappings as tracker}
                                    <a href={tracker.trackerUrl || '#'} target="_blank" class="w-10 h-10 rounded-full bg-card border border-border/50 shadow-sm flex items-center justify-center hover:bg-muted transition-colors"><img src={getTrackerFavicon(tracker.trackerName)} class="w-4 h-4 rounded-sm" alt={tracker.trackerName} /></a>
                                {/each}
                                <button class="w-10 h-10 rounded-full bg-muted/50 border border-border/50 shadow-sm flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors" onclick={() => showTrackerModal = true}><Plus class="w-4 h-4" /></button>
                            </div>

                            <div class="bg-muted/10 rounded-xl p-4 border border-border/40 backdrop-blur-sm" in:fly={{ y: 20, duration: 600, delay: 250 }}>
                                <div class="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs text-muted-foreground font-medium">Format</span>
                                        <span class="font-bold text-foreground text-sm truncate">{formatType(meta?.subtype || fullContent.content.contentType)}</span>
                                    </div>
                                    <div class="flex flex-col gap-0.5">
                                        <span class="text-xs text-muted-foreground font-medium">Status</span>
                                        <span class="font-bold text-sm {meta?.status?.toLowerCase() === 'releasing' ? 'text-green-500' : 'text-foreground'}">{meta?.status || 'TBA'}</span>
                                    </div>
                                    {#if meta?.epsOrChapters}
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-xs text-muted-foreground font-medium">{fullContent.content.contentType === 'anime' ? 'Episodes' : 'Chapters'}</span>
                                            <span class="font-bold text-foreground text-sm">{meta.epsOrChapters}</span>
                                        </div>
                                    {/if}
                                    {#if meta?.studio}
                                        <div class="flex flex-col gap-0.5">
                                            <span class="text-xs text-muted-foreground font-medium">Studio</span>
                                            <span class="font-bold text-foreground text-sm truncate">{meta.studio}</span>
                                        </div>
                                    {/if}
                                </div>
                            </div>
                        </div>

                        <!-- PESTAÑAS (Overview & Episodes) -->
                        <div class="w-full mt-4 md:mt-8" in:fly={{ y: 20, duration: 600, delay: 300 }}>
                            <!-- 1. Cambiamos el value por defecto a "overview" -->
                            <Tabs.Root value="overview" class="w-full">

                                <Tabs.List class="w-full flex border-b border-border/20 bg-transparent h-14 md:h-16 p-0 mb-8 overflow-x-auto hide-scrollbar sticky top-0 z-30 backdrop-blur-xl bg-background/60">
                                    <!-- 2. Ponemos el botón de Overview primero -->
                                    <Tabs.Trigger value="overview" class="flex-1 h-full rounded-none border-b-2 border-transparent data-[state=active]:border-foreground data-[state=active]:bg-muted/20 data-[state=active]:text-foreground text-muted-foreground font-bold text-sm md:text-base transition-all hover:bg-muted/10 hover:text-foreground px-4 md:px-8">
                                        Overview
                                    </Tabs.Trigger>
                                    <Tabs.Trigger value="episodes" class="flex-1 h-full rounded-none border-b-2 border-transparent data-[state=active]:border-foreground data-[state=active]:bg-muted/20 data-[state=active]:text-foreground text-muted-foreground font-bold text-sm md:text-base transition-all hover:bg-muted/10 hover:text-foreground px-4 md:px-8">
                                        {fullContent.content.contentType === 'anime' ? 'Episodes' : 'Chapters'}
                                    </Tabs.Trigger>
                                </Tabs.List>

                                <!-- OVERVIEW (Contenido principal por defecto) -->
                                <Tabs.Content value="overview" class="outline-none space-y-12 animate-in fade-in-50 pb-12">

                                    <!-- Géneros y Tags -->
                                    {#if (meta?.genres && meta.genres.length > 0) || (meta?.tags && meta.tags.length > 0)}
                                        <div class="space-y-6">
                                            <h3 class="text-xl font-semibold tracking-tight">{i18n.t('themes_tags') || 'Genres & Themes'}</h3>
                                            <div class="bg-muted/10 border border-border/40 rounded-xl p-5 space-y-5">
                                                {#if meta?.genres && meta.genres.length > 0}
                                                    <div class="space-y-2.5">
                                                        <div class="flex flex-wrap gap-2">
                                                            {#each meta.genres as genre}
                                                                <span class="px-3 py-1.5 bg-muted/50 border border-border/50 text-foreground text-xs font-semibold rounded-lg">
                                                                    {i18n.t(genre.toLowerCase()) || genre}
                                                                </span>
                                                            {/each}
                                                        </div>
                                                    </div>
                                                {/if}
                                                {#if meta?.tags && meta.tags.length > 0}
                                                    <div class="flex flex-wrap gap-1.5 pt-2 border-t border-border/40">
                                                        {#each meta.tags as tag}
                                                            <Badge variant="outline" class="text-[11px] font-medium text-muted-foreground hover:text-foreground transition-colors bg-background">
                                                                {tag}
                                                            </Badge>
                                                        {/each}
                                                    </div>
                                                {/if}
                                            </div>
                                        </div>
                                    {/if}

                                    <!-- Personajes y Staff -->
                                    {#if (meta?.characters && meta.characters.length > 0) || (meta?.staff && meta.staff.length > 0)}
                                        <div>
                                            <CastAndStaff characters={meta.characters || []} staff={meta.staff || []} />
                                        </div>
                                    {/if}

                                    <!-- Relaciones -->
                                    {#if fullContent.relations && fullContent.relations.length > 0}
                                        <div class="pt-4 border-t border-border/20">
                                            <RelationsTab relations={fullContent.relations} />
                                        </div>
                                    {/if}

                                </Tabs.Content>

                                <!-- EPISODIOS -->
                                <Tabs.Content value="episodes" class="outline-none space-y-8 animate-in fade-in-50">
                                    {#if fullContent.content.contentType === 'anime'}
                                        {#if meta?.subtype !== 'MOVIE'}
                                            <EpisodeSelector cid={fullContent.content.cid} epsOrChapters={meta?.epsOrChapters} contentUnits={fullContent.contentUnits} />
                                        {/if}
                                    {:else}
                                        {#await extensionsPromise then extRes}
                                            <ChapterTable cid={fullContent.content.cid} contentType={fullContent.content.contentType} availableExtensions={extRes || []} />
                                        {/await}
                                    {/if}
                                </Tabs.Content>

                            </Tabs.Root>

                            <!-- Sidebar Móvil -->
                            <div class="lg:hidden mt-12 pt-12 border-t border-border/20">
                                <h3 class="text-xl font-bold tracking-tight mb-6">{i18n.t('information') || 'Information'}</h3>
                                <ContentSidebar
                                        cid={fullContent.content.cid}
                                        metadata={meta}
                                        extensions={fullContent.extensionSources}
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </main>

            <!-- Modales -->
            <ListEditorModal bind:open={showListModal} cid={fullContent.content.cid} title={meta?.title} contentType={fullContent.content.contentType} coverImage={meta?.coverImage ?? undefined} />
            <TrackerManagerModal bind:open={showTrackerModal} cid={fullContent.content.cid} trackers={fullContent.trackerMappings} />
            <TrackerCandidatesModal bind:open={showCandidatesModal} cid={fullContent.content.cid} candidates={stateCandidates} />

        {:catch error}
            <div class="flex h-[85vh] flex-col items-center justify-center gap-4">
                <p class="text-lg text-muted-foreground">{i18n.t('failed_load_content')}</p>
                <Button variant="outline" onclick={() => location.reload()}>{i18n.t('retry')}</Button>
            </div>
        {/await}
    </div>
{/if}