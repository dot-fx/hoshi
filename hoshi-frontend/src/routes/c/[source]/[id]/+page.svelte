<script lang="ts">
    import { fade } from "svelte/transition";

    import { i18n } from "@/i18n/index.svelte.js";
    import { primaryMetadata } from "@/api/content/types";
    import Sidebar from "@/components/content/Sidebar.svelte";
    import Episodes from "@/components/content/Episodes.svelte";
    import Chapters from "@/components/content/Chapters.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import RelationsTab from "@/components/content/Relations.svelte";
    import TrackerManager from "@/components/modals/TrackerManager.svelte";
    import ListEditor from '@/components/modals/ListEditor.svelte';
    import ExtensionManager from '@/components/modals/ExtensionManager.svelte';
    import { appConfig } from "@/stores/config.svelte.js";

    import * as Tabs from "@/components/ui/tabs";
    import { Button } from "@/components/ui/button";
    import { Badge } from "@/components/ui/badge";
    import { Spinner } from "@/components/ui/spinner";
    import { Play, BookOpen, BookmarkPlus, Check, Link, Plug, AlertCircle } from "lucide-svelte";

    import { ContentDetailState } from "@/app/content.svelte";
    import { layoutState } from "@/stores/layout.svelte.js";

    const detail = new ContentDetailState();

    let showListModal = $state(false);
    let showTrackerModal = $state(false);
    let showExtensionModal = $state(false);

    $effect(() => {
        layoutState.showBack = true;
        layoutState.backUrl = "/";
    });
</script>

<svelte:head>
    {#if detail.isLoading}
        <title>{i18n.t('content.loading')}</title>
    {:else if detail.error}
        <title>Error</title>
    {:else if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const title = meta?.titleI18n?.[pref] || meta?.title || i18n.t('content.details')}
        <title>{title}</title>
    {/if}
</svelte:head>

<div class="min-h-screen bg-background pb-24">
    {#if detail.isLoading}
        <div class="flex h-[85vh] w-full bg-background items-center justify-center">
            <Spinner class="w-10 h-10 text-muted-foreground/20" />
        </div>

    {:else if detail.error}
        <div class="flex h-[85vh] flex-col items-center justify-center gap-4">
            <AlertCircle class="w-12 h-12 text-destructive opacity-20" />
            <p class="text-lg text-muted-foreground font-medium">{i18n.t(detail.error?.key || 'errors.error')}</p>
            <Button variant="outline" class="rounded-full" onclick={() => detail.retry()}>
                {i18n.t('content.retry')}
            </Button>
        </div>

    {:else if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const isMovie = meta?.subtype === 'MOVIE'}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}
        {@const score = meta?.rating ? Math.round(meta.rating * 10) : null}
        {@const isAdultContent = detail.fullContent.content.nsfw || meta?.genres?.some(g => ['hentai', 'adult'].includes(g.toLowerCase()))}
        {@const shouldBlur = isAdultContent && (appConfig.data?.general?.blurAdultContent ?? true)}

        <div class="absolute top-0 inset-x-0 w-full h-[60vh] md:h-[75vh] overflow-hidden pointer-events-none" in:fade={{ duration: 800 }}>
            <div class="absolute inset-0 w-full h-full" style="mask-image: linear-gradient(to bottom, black 40%, transparent 100%);">
                <img src={meta?.bannerImage || meta?.coverImage} alt="Background" class="w-full h-full object-cover opacity-15 md:opacity-25 {meta?.bannerImage && !shouldBlur ? '' : 'blur-xl scale-110'} {shouldBlur ? 'blur-3xl scale-125 opacity-10' : ''}" />
            </div>
            <div class="absolute inset-0 bg-linear-to-b from-background/40 via-background/10 to-transparent"></div>
        </div>

        <main class="relative z-10 w-full max-w-[2000px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 pt-16 md:pt-28 lg:pt-36" in:fade={{ delay: 200 }}>
            <div class="grid grid-cols-1 xl:grid-cols-[280px_1fr] gap-6 lg:gap-12 items-start">

                <div class="order-2 xl:order-1">
                    <Sidebar
                            metadata={meta}
                            trackerMappings={detail.fullContent.trackerMappings}
                    />
                </div>

                <div class="flex flex-col w-full min-w-0 order-1 xl:order-2">
                    <div class="pt-4 mb-10">
                        <!-- Móvil -->
                        <div class="flex gap-4 lg:hidden mb-6 items-start">
                            <div class="w-24 sm:w-32 shrink-0 rounded-3xl overflow-hidden shadow-xl bg-muted relative">
                                <img src={meta?.coverImage} alt="Cover" class="w-full aspect-[2/3] object-cover {shouldBlur ? 'blur-2xl scale-110' : ''}" />
                            </div>
                            <div class="flex flex-col flex-1 py-0.5 gap-2.5">
                                <h1 class="text-xl sm:text-2xl font-black leading-tight line-clamp-2">{displayTitle}</h1>

                                {#if score}
                                    <Badge variant="outline" class="w-fit bg-green-500/10 text-green-500 font-bold">{score}%</Badge>
                                {/if}

                                <div class="flex flex-wrap gap-2 mt-2">
                                    <Button size="sm" onclick={() => detail.watchNow(detail.fullContent)} class="rounded-xl px-4 h-9 font-bold shadow-lg bg-primary">
                                        {#if detail.fullContent.content.contentType === 'anime'}
                                            <Play class="w-4 h-4 mr-1.5 fill-current" /> {i18n.t('content.watch_now')}
                                        {:else}
                                            <BookOpen class="w-4 h-4 mr-1.5 fill-current" /> {i18n.t('content.read_now')}
                                        {/if}
                                    </Button>

                                    <Button size="icon" variant="secondary" class="rounded-xl w-9 h-9" onclick={() => showListModal = true}>
                                        {#if detail.isEntryLoading}
                                            <Spinner class="w-4 h-4" />
                                        {:else if detail.hasEntry}
                                            <Check class="w-4 h-4 text-green-500" />
                                        {:else}
                                            <BookmarkPlus class="w-4 h-4" />
                                        {/if}
                                    </Button>

                                    <Button size="icon" variant="secondary" class="rounded-xl w-9 h-9" onclick={() => showTrackerModal = true}>
                                        <Link class="w-4 h-4" />
                                    </Button>

                                    <Button size="icon" variant="secondary" class="rounded-xl w-9 h-9" onclick={() => showExtensionModal = true}>
                                        <Plug class="w-4 h-4" />
                                    </Button>
                                </div>
                            </div>
                        </div>

                        <!-- Desktop -->
                        <div class="hidden lg:flex flex-col gap-5">
                            <h1 class="text-5xl xl:text-6xl font-black drop-shadow-2xl leading-[1.1]">{displayTitle}</h1>

                            <div class="flex flex-wrap items-center gap-3 text-sm font-bold">
                                {#if score} <Badge class="bg-green-500/20 text-green-500 border-green-500/30">{score}% Rating</Badge> {/if}
                                {#if meta?.releaseDate} <span class="text-muted-foreground">{meta.releaseDate.split('-')[0]}</span> {/if}
                                {#if meta?.epsOrChapters}
                                    <span class="text-muted-foreground">•
                                        {detail.fullContent.content.contentType === 'anime' ? i18n.t('content.eps_count', { count: meta.epsOrChapters }) : i18n.t('content.ch_count', { count: meta.epsOrChapters })}
                                    </span>
                                {/if}
                            </div>

                            {#if meta?.synopsis}
                                <p class="text-muted-foreground text-lg leading-relaxed max-w-3xl line-clamp-3">{@html meta.synopsis.replace(/<[^>]*>?/gm, '')}</p>
                            {/if}

                            <div class="flex items-center gap-3 pt-4">
                                <Button size="lg" onclick={() => detail.watchNow(detail.fullContent)} class="rounded-full px-8 h-12 font-bold shadow-lg flex-1 lg:flex-none">
                                    {#if detail.fullContent.content.contentType === 'anime'}
                                        <Play class="w-5 h-5 mr-2 fill-current" /> {i18n.t('content.watch_now')}
                                    {:else}
                                        <BookOpen class="w-5 h-5 mr-2 fill-current" /> {i18n.t('content.read_now')}
                                    {/if}
                                </Button>

                                <div class="flex gap-2">
                                    <Button size="icon" variant="secondary" class="rounded-2xl w-12 h-12 hover:bg-primary/10 transition-all" onclick={() => showListModal = true}>
                                        {#if detail.isEntryLoading}
                                            <Spinner class="w-5 h-5" />
                                        {:else if detail.hasEntry}
                                            <Check class="w-5 h-5 text-green-500" />
                                        {:else}
                                            <BookmarkPlus class="w-5 h-5" />
                                        {/if}
                                    </Button>

                                    <!-- Gestionar trackers -->
                                    <Button size="icon" variant="secondary" class="rounded-2xl w-12 h-12 hover:bg-primary/10 transition-all" onclick={() => showTrackerModal = true}>
                                        <Link class="w-5 h-5" />
                                    </Button>

                                    <!-- Gestionar extensiones -->
                                    <Button size="icon" variant="secondary" class="rounded-2xl w-12 h-12 hover:bg-primary/10 transition-all" onclick={() => showExtensionModal = true}>
                                        <Plug class="w-5 h-5" />
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Tabs y resto del contenido (sin cambios) -->
                    <div class="w-full">
                        {#if isMovie}
                            <div class="space-y-12 pb-12">
                                <CastAndStaff characters={meta?.characters || []} staff={meta?.staff || []} />
                                {#if detail.fullContent.relations.length > 0}
                                    <div class="pt-6 border-t border-border/20">
                                        <RelationsTab relations={detail.fullContent.relations} />
                                    </div>
                                {/if}
                            </div>
                        {:else}
                            <Tabs.Root value="overview" class="w-full">
                                <Tabs.List class="w-full flex justify-start gap-10 border-b border-border/40 bg-transparent h-12 p-0 mb-8 overflow-x-auto hide-scrollbar">
                                    <Tabs.Trigger value="overview" class="h-full rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none text-muted-foreground font-bold text-base transition-all hover:text-foreground px-1 bg-transparent">{i18n.t('content.overview')}</Tabs.Trigger>
                                    <Tabs.Trigger value="episodes" class="h-full rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-primary data-[state=active]:bg-transparent data-[state=active]:shadow-none text-muted-foreground font-bold text-base transition-all hover:text-foreground px-1 bg-transparent">
                                        {detail.fullContent.content.contentType === 'anime' ? i18n.t('content.episodes') : i18n.t('content.chapters')}
                                    </Tabs.Trigger>
                                </Tabs.List>

                                <Tabs.Content value="overview" class="space-y-12 pb-12">
                                    <CastAndStaff characters={meta?.characters || []} staff={meta?.staff || []} />
                                    {#if detail.fullContent.relations.length > 0}
                                        <div class="pt-6 border-t border-border/20">
                                            <RelationsTab relations={detail.fullContent.relations} />
                                        </div>
                                    {/if}
                                </Tabs.Content>

                                <Tabs.Content value="episodes">
                                    {#if detail.fullContent.content.contentType === 'anime'}
                                        <Episodes cid={detail.fullContent.content.cid} source={detail.source} sourceId={detail.id} epsOrChapters={meta?.epsOrChapters} contentUnits={detail.fullContent.contentUnits} />
                                    {:else}
                                        <Chapters cid={detail.fullContent.content.cid} contentType={detail.fullContent.content.contentType} source={detail.source} sourceId={detail.id} />
                                    {/if}
                                </Tabs.Content>
                            </Tabs.Root>
                        {/if}
                    </div>
                </div>
            </div>
        </main>

        <ListEditor bind:open={showListModal} cid={detail.fullContent.content.cid} title={displayTitle} contentType={detail.fullContent.content.contentType} coverImage={meta?.coverImage ?? undefined} />
        <TrackerManager
                bind:open={showTrackerModal}
                cid={detail.fullContent.content.cid}
                trackers={detail.fullContent.trackerMappings}
                metadata={meta}
        />
        <ExtensionManager
                bind:open={showExtensionModal}
                cid={detail.fullContent.content.cid}
                metadata={meta}
                isNsfw={isAdultContent}
                extensions={detail.fullContent.extensionSources}
                contentType={detail.fullContent.content.contentType}
        />
    {/if}
</div>