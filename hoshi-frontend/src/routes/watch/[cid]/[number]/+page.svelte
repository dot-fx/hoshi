<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { untrack } from "svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { extensions as extensionsStore } from "$lib/extensions.svelte";
    import { buildProxyUrl, proxyApi } from "$lib/api/proxy/proxy";
    import { isTauri } from "$lib/api/client";
    import Player from "$lib/components/Player.svelte";
    import type { Subtitle, Chapter } from "$lib/components/Player.svelte";

    import { progressApi } from '@/api/progress/progress';
    import { listApi } from '@/api/list/list';
    import { appConfig } from '@/config.svelte';

    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import * as Empty from "$lib/components/ui/empty";
    import { AlertCircle, PuzzleIcon, ChevronLeft, Settings2, Mic2, SkipBack, SkipForward } from "lucide-svelte";
    import { primaryMetadata } from "$lib/api/content/types";
    import {discordApi} from "@/api/discord/discord";
    import ExtensionManager from "@/components/modals/ExtensionManager.svelte";

    const cid = $derived(page.params.cid);
    const epNumber = $derived(Number(page.params.number));

    let animeTitle = $derived.by(() => {
        if (!animeData) return "";
        const meta = primaryMetadata(animeData, appConfig.data?.content?.preferredMetadataProvider);
        const pref = appConfig.data?.ui?.titleLanguage || 'romaji';

        return meta?.titleI18n?.[pref] || meta?.title || "";
    });
    let episodeTitle = $state("");
    let extensions = $state<string[]>([]);
    let selectedExtension = $state<string | null>(null);
    let servers = $state<string[]>([]);
    let supportsDub = $state(false);
    let selectedServer = $state<string | null>(null);
    let isDub = $state(false);
    let animeData = $state<any>(null);

    let m3u8Url = $state<string | null>(null);
    let subtitles = $state<Subtitle[]>([]);
    let chapters = $state<Chapter[]>([]);
    let isLoadingMeta = $state(true);
    let isLoadingPlay = $state(false);
    let error = $state<string | null>(null);

    let lastSyncTime = $state(0);
    let hasUpdatedList = $state(false);
    let initialTime = $state(0);
    let currentLoadedCid = $state<string | null>(null);
    let currentLoadedEp = $state<number | null>(null);
    let subtitleBlobUrls: string[] = [];

    let isPaused = $state(true);
    let currentDuration = $state(0);
    let lastCurrentTime = $state(0);
    let discordStatusUpdated = $state(false);

    let showExtensionManager = $state(false);

    const isMappingError = $derived(error?.toLowerCase().includes("not found") || error?.toLowerCase().includes("no results"));

    $effect(() => {
        return () => {
            discordApi.clearActivity().catch(() => {});
        };
    });

    async function syncDiscord(paused: boolean) {
        if (!animeData) return;

        isPaused = paused;
        const meta = primaryMetadata(animeData, appConfig.data?.content?.preferredMetadataProvider);
        const nowInSeconds = Math.floor(Date.now() / 1000);

        const startTime = !paused ? nowInSeconds - Math.floor(lastCurrentTime) : null;
        const endTime = !paused && currentDuration > 0 ? startTime! + Math.floor(currentDuration) : null;

        await discordApi.setActivity({
            title: animeTitle,
            details: episodeTitle,
            imageUrl: meta?.coverImage || null,
            startTime,
            endTime,
            isVideo: true,
            isNsfw: animeData.content.nsfw
        }).catch(() => {});
    }

    let totalEpisodes = $derived.by(() => {
        if (!animeData) return 0;
        const meta = primaryMetadata(animeData);
        return meta?.epsOrChapters || 0;
    });
    let hasNext = $derived(totalEpisodes > 0 && epNumber < totalEpisodes);
    let hasPrev = $derived(epNumber > 1);

    function revokeSubtitleBlobs() {
        subtitleBlobUrls.forEach(u => URL.revokeObjectURL(u));
        subtitleBlobUrls = [];
    }

    async function loadPlay() {
        if (!selectedExtension) return;
        isLoadingPlay = true;
        m3u8Url = null;
        error = null;
        revokeSubtitleBlobs();
        lastSyncTime = 0;
        hasUpdatedList = false;

        try {
            if (appConfig.data?.player.resumeFromLastPos) {
                try {
                    const res = await progressApi.getContentProgress(cid);
                    const prog = res.animeProgress.find(p => p.episode === epNumber);
                    initialTime = prog?.timestampSeconds ?? 0;
                } catch { initialTime = 0; }
            }

            const opts: { server?: string; category?: string } = {};
            if (selectedServer) opts.server = selectedServer;
            if (supportsDub && isDub) opts.category = "dub";

            const res = await contentApi.play(cid || "", selectedExtension, epNumber, opts);
            if (res.type !== "video") throw new Error(i18n.t('watch.no_stream'));
            const data = res.data as any;
            const headers = data.headers ?? {};

            m3u8Url = buildProxyUrl({ url: data.source.url, ...extractHeaders(headers) });
            subtitles = await Promise.all(
                (data.source.subtitles ?? []).map(async (s: any) => {
                    const proxyParams = { url: s.url, ...extractHeaders(headers) };
                    if (!isTauri()) return { ...s, url: buildProxyUrl(proxyParams) };
                    try {
                        const blob = await proxyApi.fetch(proxyParams);
                        const blobUrl = URL.createObjectURL(blob);
                        subtitleBlobUrls.push(blobUrl);
                        return { ...s, url: blobUrl };
                    } catch {
                        return { ...s, url: buildProxyUrl(proxyParams) };
                    }
                })
            );
            chapters = data.source.chapters ?? [];
        } catch (e: any) {
            error = e || "Error";
        } finally {
            isLoadingPlay = false;
        }
    }

    function handlePlayerProgress({ currentTime, duration }: { currentTime: number; duration: number }) {
        if (!appConfig.data) return;
        if (!discordStatusUpdated && duration > 0) {
            const meta = primaryMetadata(animeData, appConfig.data?.content?.preferredMetadataProvider);
            const coverImage = meta?.coverImage || "";

            const now = Math.floor(Date.now() / 1000);
            const start = now - Math.floor(currentTime);
            const end = start + Math.floor(duration);

            const isNsfwContent = animeData?.content?.nsfw ?? false;

            discordApi.setActivity({
                title: animeTitle,
                details: episodeTitle,
                imageUrl: coverImage,
                startTime: start,
                endTime: end,
                isVideo: true,
                isNsfw: isNsfwContent
            }).catch(() => {});

            discordStatusUpdated = true;
        }
        if (Math.abs(currentTime - lastSyncTime) >= 10 || (lastSyncTime === 0 && currentTime > 2)) {
            lastSyncTime = currentTime;
            progressApi.updateAnimeProgress({
                cid,
                episode: epNumber,
                timestampSeconds: Math.floor(currentTime),
                episodeDurationSeconds: duration > 0 ? Math.floor(duration) : undefined,
                completed: duration > 0 && (currentTime / duration) >= 0.9
            }).catch(() => {});
        }

        if (!hasUpdatedList && duration > 0 && appConfig.data.content.autoUpdateProgress) {
            if (currentTime / duration >= 0.8) {
                hasUpdatedList = true;
                const status = (totalEpisodes > 0 && epNumber >= totalEpisodes) ? "COMPLETED" : "CURRENT";
                listApi.upsert({ cid, status, progress: epNumber }).catch(() => {});
            }
        }
    }

    function extractHeaders(headers: Record<string, string>) {
        return {
            referer: headers["Referer"],
            origin: headers["Origin"],
            userAgent: headers["User-Agent"],
        };
    }

    async function selectExtension(ext: string) {
        selectedExtension = ext;
        servers = [];
        supportsDub = false;
        selectedServer = null;
        isDub = false;
        try {
            const s = await extensionsApi.getSettings(ext);
            servers = s.episodeServers ?? [];
            supportsDub = s.supportsDub ?? false;
            selectedServer = servers[0] ?? null;
        } catch {}
        await loadPlay();
    }

    async function loadPageData(targetCid: string, targetEp: number) {
        try {
            discordStatusUpdated = false;
            if (targetCid !== currentLoadedCid) {
                isLoadingMeta = true;
                const contentRes = await contentApi.get(targetCid);
                animeData = contentRes;

                const globalExtensions = extensionsStore.anime.map(e => e.id);

                const contentExtensions = contentRes.extensionSources?.map((e: any) => e.extensionName) || [];

                extensions = globalExtensions;

                currentLoadedCid = targetCid;

                if (extensions.length > 0) {
                    const initialExt = contentExtensions.find(e => globalExtensions.includes(e)) || extensions[0];
                    await selectExtension(initialExt);
                }
                currentLoadedCid = targetCid;
                isLoadingMeta = false;
            } else {
                currentLoadedEp = targetEp;
                await loadPlay();
            }
            updateEpisodeTitle(targetEp);
        } catch (e: any) {
            error = e?.message;
            isLoadingMeta = false;
        }
    }

    function updateEpisodeTitle(ep: number) {
        const unit = animeData?.contentUnits?.find((u: any) => u.unitNumber === ep);
        episodeTitle = unit?.title
            ? i18n.t('watch.episode_with_title', { num: ep, title: unit.title })
            : i18n.t('watch.episode_number', { num: ep });
    }

    $effect(() => {
        if (cid && epNumber && (cid !== currentLoadedCid || epNumber !== currentLoadedEp)) {
            untrack(() => loadPageData(cid, epNumber));
        }
    });

    $effect(() => () => revokeSubtitleBlobs());

</script>

<svelte:head>
    <title>{episodeTitle} - {animeTitle}</title>
</svelte:head>

{#snippet TopBar()}
    <div class="custom-top-bar absolute top-0 inset-x-0 z-50 p-4 md:p-6 hidden md:flex flex-col xl:flex-row items-start xl:items-center justify-between gap-4 pointer-events-none bg-gradient-to-b from-black/80 via-black/40 to-transparent transition-opacity duration-300">

        <div class="pointer-events-auto flex items-center gap-3 md:gap-4 text-left min-w-0 shrink-0">
            <Button variant="ghost" size="icon" href={`/content/${cid}`} class="rounded-xl bg-black/40 hover:bg-white/20 text-white border border-white/10 backdrop-blur-md h-11 w-11 shrink-0">
                <ChevronLeft class="size-6" />
            </Button>
            <div class="flex flex-col drop-shadow-lg min-w-0 max-w-[40vw]">
                <h1 class="font-black text-base md:text-lg leading-tight truncate text-white/95">{animeTitle || i18n.t('watch.loading')}</h1>
                <p class="text-xs md:text-sm font-bold text-white/60 truncate uppercase tracking-wider">{episodeTitle}</p>
            </div>
        </div>

        <div class="pointer-events-auto flex items-center gap-2 shrink-0">
            {#if !isLoadingMeta}
                <div class="flex items-center bg-black/40 border border-white/10 p-1 rounded-xl backdrop-blur-md">
                    <Button variant="ghost" size="icon" disabled={!hasPrev} href={`/watch/${cid}/${epNumber - 1}`} class="h-9 w-9 text-white"><SkipBack class="size-4" /></Button>
                    <div class="w-px h-5 bg-white/20 mx-1"></div>
                    <Button variant="ghost" size="icon" disabled={!hasNext} href={`/watch/${cid}/${epNumber + 1}`} class="h-9 w-9 text-white"><SkipForward class="size-4" /></Button>
                </div>

                <div class="flex items-center bg-black/40 border border-white/10 p-1.5 rounded-xl backdrop-blur-md shadow-lg shrink-0">
                    <Select.Root type="single" value={selectedExtension ?? ""} onValueChange={selectExtension}>
                        <Select.Trigger class="h-9 px-3 bg-transparent border-none text-white/90 hover:bg-white/10 rounded-lg flex items-center gap-2 font-semibold">
                            <PuzzleIcon class="size-4 text-white/50" />
                            <span class="truncate text-xs md:text-sm">{selectedExtension ?? i18n.t('watch.select_extension')}</span>
                        </Select.Trigger>
                        <Select.Content class="rounded-xl">
                            {#each extensions as ext}
                                <Select.Item value={ext} label={ext}>{ext}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>

                    <div class="w-px h-6 bg-white/20 mx-0.5"></div>

                    <Select.Root type="single" value={selectedServer ?? ""} onValueChange={(v) => { selectedServer = v; loadPlay(); }}>
                        <Select.Trigger class="h-9 px-3 bg-transparent border-none text-white/90 hover:bg-white/10 rounded-lg flex items-center gap-2 font-semibold">
                            <Settings2 class="size-4 text-white/50" />
                            <span class="truncate text-xs md:text-sm">{selectedServer ?? i18n.t('watch.auto_server')}</span>
                        </Select.Trigger>
                        <Select.Content class="rounded-xl">
                            {#each servers as srv}
                                <Select.Item value={srv} label={srv}>{srv}</Select.Item>
                            {/each}
                        </Select.Content>
                    </Select.Root>

                    {#if supportsDub}
                        <div class="w-px h-6 bg-white/20 mx-0.5"></div>
                        <div class="flex items-center gap-2 px-3 h-9">
                            <Mic2 class="size-4 text-white/50" />
                            <Label for="dub-switch" class="text-[10px] font-black uppercase tracking-widest text-white/70 cursor-pointer">{i18n.t('watch.dub')}</Label>
                            <Switch id="dub-switch" checked={isDub} onCheckedChange={(v) => { isDub = v; loadPlay(); }} disabled={isLoadingPlay} class="scale-90" />
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
{/snippet}

{#snippet MobileControls()}
    <div class="md:hidden flex flex-col p-4 gap-6 bg-background">
        <div class="flex items-start gap-3">
            <Button variant="secondary" size="icon" href={`/content/${cid}`} class="rounded-full shrink-0">
                <ChevronLeft class="size-5" />
            </Button>
            <div class="flex flex-col min-w-0 pt-1">
                <h1 class="font-bold text-xl leading-tight text-foreground">{animeTitle || i18n.t('watch.loading')}</h1>
                <p class="text-sm font-medium text-muted-foreground mt-1">{episodeTitle}</p>
            </div>
        </div>

        <div class="flex items-center justify-between p-1 bg-muted/50 rounded-2xl border border-border">
            <Button variant="ghost" disabled={!hasPrev} href={`/watch/${cid}/${epNumber - 1}`} class="flex-1 rounded-xl h-12 gap-2">
                <SkipBack class="size-4" /> {i18n.t('watch.previous')}
            </Button>
            <div class="w-px h-8 bg-border"></div>
            <Button variant="ghost" disabled={!hasNext} href={`/watch/${cid}/${epNumber + 1}`} class="flex-1 rounded-xl h-12 gap-2">
                {i18n.t('watch.next')} <SkipForward class="size-4" />
            </Button>
        </div>

        <div class="flex flex-col gap-3">
            <div class="space-y-1.5">
                <Label class="text-xs font-bold text-muted-foreground uppercase tracking-wider pl-1">{i18n.t('watch.select_extension')}</Label>
                <Select.Root type="single" value={selectedExtension ?? ""} onValueChange={selectExtension}>
                    <Select.Trigger class="w-full h-12 rounded-xl bg-card border-border">
                        <span class="truncate">{selectedExtension ?? i18n.t('watch.select_extension')}</span>
                    </Select.Trigger>
                    <Select.Content class="rounded-xl">
                        {#each extensions as ext}
                            <Select.Item value={ext} label={ext}>{ext}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            <div class="space-y-1.5">
                <Label class="text-xs font-bold text-muted-foreground uppercase tracking-wider pl-1">{i18n.t('watch.server')}</Label>
                <Select.Root type="single" value={selectedServer ?? ""} onValueChange={(v) => { selectedServer = v; loadPlay(); }}>
                    <Select.Trigger class="w-full h-12 rounded-xl bg-card border-border">
                        <span class="truncate">{selectedServer ?? i18n.t('watch.auto_server')}</span>
                    </Select.Trigger>
                    <Select.Content class="rounded-xl">
                        {#each servers as srv}
                            <Select.Item value={srv} label={srv}>{srv}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>
        </div>

        {#if supportsDub}
            <div class="flex items-center justify-between border border-border rounded-xl p-4 bg-card mt-2">
                <Label for="mobile-dub-switch" class="font-bold flex items-center gap-2">
                    <Mic2 class="size-5 text-primary"/> {i18n.t('watch.dub')}
                </Label>
                <Switch id="mobile-dub-switch" checked={isDub} onCheckedChange={(v) => { isDub = v; loadPlay(); }} disabled={isLoadingPlay} />
            </div>
        {/if}

        <div class="h-8"></div>
    </div>
{/snippet}


<div class="flex flex-col md:block w-full h-full md:bg-black bg-background" style="padding-top: env(safe-area-inset-top);">

    <div class="w-full aspect-video md:w-full md:h-full md:absolute md:inset-0 bg-black flex items-center justify-center relative z-10 shrink-0 shadow-lg md:shadow-none">

        {#if !isLoadingMeta}
            {@render TopBar()}
        {/if}

        {#if error}
            <div class="flex flex-col items-center gap-6 p-6 z-20 max-w-md text-center animate-in fade-in zoom-in duration-300">
                <div class="p-4 rounded-full bg-destructive/10 ring-1 ring-destructive/20">
                    <AlertCircle class="w-12 h-12 text-destructive" />
                </div>

                <div class="space-y-2">
                    <p class="text-white/90 text-xl font-black">
                        {isMappingError ? i18n.t('watch.mapping_error_title') : i18n.t('watch.error_playing')}
                    </p>
                    <p class="text-white/50 text-xs font-mono bg-white/5 p-3 rounded-lg border border-white/10 break-all max-w-xs mx-auto">
                        {error}
                    </p>
                </div>

                <div class="flex flex-wrap justify-center gap-3">
                    <Button variant="secondary" onclick={loadPlay} class="rounded-xl px-6 font-bold">
                        {i18n.t('watch.retry')}
                    </Button>

                    {#if isMappingError}
                        <Button
                                variant="outline"
                                class="rounded-xl px-6 font-bold bg-white/5 border-white/20 text-white hover:bg-white/10"
                                onclick={() => showExtensionManager = true}
                        >
                            <PuzzleIcon class="size-4 mr-2" />
                            {i18n.t('content.extension_manager.manage_extensions_title')}
                        </Button>
                    {/if}
                </div>
            </div>
        {:else if isLoadingPlay}
            <div class="flex flex-col items-center gap-4 z-10">
                <div class="relative">
                    <div class="size-12 border-4 border-primary/20 border-t-primary rounded-full animate-spin"></div>
                </div>
                <span class="text-white/60 text-sm font-bold tracking-widest uppercase">{i18n.t('watch.loading_stream')}</span>
            </div>
        {:else if !isLoadingMeta && extensions.length === 0}
            <div class="flex-1 flex items-center justify-center p-4">
                <Empty.Root>
                    <Empty.Title>{i18n.t('watch.no_extensions')}</Empty.Title>
                    <Button variant="secondary" onclick={() => goto("/marketplace")} class="mt-4">{i18n.t('marketplace.title')}</Button>
                </Empty.Root>
            </div>
        {:else if m3u8Url}
            <div class="w-full h-full">
                <Player
                        src={m3u8Url}
                        {animeTitle}
                        {episodeTitle}
                        {subtitles}
                        {chapters}
                        {cid}
                        episode={epNumber}
                        initialTime={initialTime}
                        onTimeUpdate={(data) => {
                        lastCurrentTime = data.currentTime;
                        currentDuration = data.duration;
                        isPaused = data.paused;
                        handlePlayerProgress(data);
                    }}
                        onPlay={() => syncDiscord(false)}
                        onPause={() => syncDiscord(true)}
                        onSeek={(time) => {
                        lastCurrentTime = time;
                        syncDiscord(isPaused);
                    }}
                        onEnded={() => {
                        discordApi.clearActivity();
                        if (hasNext) goto(`/watch/${cid}/${epNumber + 1}`);
                    }}
                >
                    {@render TopBar()}
                </Player>
            </div>
        {/if}
    </div>

    <div class="flex-1 overflow-y-auto md:hidden w-full relative">
        {@render MobileControls()}
    </div>
</div>


{#if animeData}
    <ExtensionManager
            bind:open={showExtensionManager}
            cid={cid}
            metadata={primaryMetadata(animeData)}
            isNsfw={animeData.content.nsfw}
            extensions={animeData.extensionSources || []}
            contentType="anime"
            onSuccess={async () => {
                showExtensionManager = false;
                currentLoadedCid = null;
                await loadPageData(cid, epNumber);
            }}
    />
{/if}

<style>
    :global(media-player:not([data-controls]) .custom-top-bar) {
        opacity: 0 !important;
        pointer-events: none !important;
    }

    :global(media-player) {
        --media-focus-ring: none;
    }
</style>