<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { untrack } from "svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { extensions as extensionsStore } from "$lib/extensions.svelte";
    import { buildProxyUrl, proxyApi } from "$lib/api/proxy/proxy";
    import { isTauri, type CoreError } from "$lib/api/client";
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
    import {contentCache} from "@/contentCache.svelte";

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
    let error = $state<CoreError | null>(null);

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

    const isMappingError = $derived(
        error?.key?.includes('no_match_found')
    );

    let playerContainer: HTMLElement | null = $state(null);

    $effect(() => {
        return () => {
            discordApi.clearActivity().catch(() => {});
        };
    });

    function formatAssTime(assTime: string) {
        let [hms, msPart = "00"] = assTime.trim().split('.');
        let [h, m, s] = hms.split(':');
        return `${h.padStart(2, '0')}:${m.padStart(2, '0')}:${s.padStart(2, '0')}.${msPart.padEnd(3, '0').substring(0, 3)}`;
    }

    function convertAssToVtt(assData: string) {
        const lines = assData.split(/\r?\n/);
        let vtt = "WEBVTT\n\n";
        let isEvents = false;
        let format: string[] = [];
        for (let line of lines) {
            line = line.trim();
            if (line === "[Events]") {
                isEvents = true;
                continue;
            }
            if (!isEvents) continue;
            if (line.startsWith("Format:")) {
                format = line.substring(7).split(",").map(s => s.trim());
                continue;
            }

            if (line.startsWith("Dialogue:")) {
                const parts = line.substring(9).split(",");
                const startIdx = format.indexOf("Start");
                const endIdx = format.indexOf("End");
                const textIdx = format.indexOf("Text");
                if (startIdx === -1 || endIdx === -1 || textIdx === -1) continue;

                const start = formatAssTime(parts[startIdx]);
                const end = formatAssTime(parts[endIdx]);

                let text = parts.slice(textIdx).join(",");
                text = text.replace(/\{[^}]+\}/g, "").replace(/\\N/gi, "\n");
                vtt += `${start} --> ${end}\n${text}\n\n`;
            }
        }
        return vtt;
    }

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

            if (res.type !== "video") throw { key: 'watch.no_stream' } as CoreError;

            const data = res.data as any;
            const headers = data.headers ?? {};

            m3u8Url = buildProxyUrl({ url: data.source.url, ...extractHeaders(headers) });
            subtitles = await Promise.all(
                (data.source.subtitles ?? []).map(async (s: any) => {
                    const proxyParams = { url: s.url, ...extractHeaders(headers) };
                    try {
                        const blob = await proxyApi.fetch(proxyParams);
                        const isAss = s.url.toLowerCase().endsWith('.ass') || s.url.toLowerCase().endsWith('.ssa');
                        let finalBlob = blob;
                        if (isAss) {
                            const textData = await blob.text();
                            const vttText = convertAssToVtt(textData);
                            finalBlob = new Blob([vttText], { type: 'text/vtt' });
                        }
                        const blobUrl = URL.createObjectURL(finalBlob);
                        subtitleBlobUrls.push(blobUrl);
                        return { ...s, url: blobUrl, type: 'vtt' };
                    } catch (err) {
                        return null;
                    }
                })
            ).then(subs => subs.filter(s => s !== null));
            chapters = data.source.chapters ?? [];
        } catch (e: any) {
            console.error(e)
            error = e.key ? e : { key: 'errors.unknown_error' };
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
                let contentRes;

                if (contentCache.has(targetCid)) {
                    contentRes = contentCache.get(targetCid);
                } else {
                    contentRes = await contentApi.get(targetCid);
                    contentCache.set(targetCid, contentRes);
                }

                animeData = contentRes;
                const globalExtensions = extensionsStore.anime.map(e => e.id);
                const contentExtensions = contentRes.extensionSources?.map((e: any) => e.extensionName) || [];
                extensions = globalExtensions;
                currentLoadedCid = targetCid;
                if (extensions.length > 0) {
                    const initialExt = contentExtensions.find(e => globalExtensions.includes(e)) || extensions[0];
                    await selectExtension(initialExt);
                }
                isLoadingMeta = false;
            } else {
                currentLoadedEp = targetEp;
                await loadPlay();
            }
            updateEpisodeTitle(targetEp);
        } catch (e: any) {
            error = e.key ? e : { key: 'errors.unknown_error', message: e.message };
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

    $effect(() => {
        if (typeof window === 'undefined' || !window.matchMedia('(pointer: coarse)').matches) return;

        const handleOrientationChange = () => {
            const isLandscape = window.screen.orientation?.type.startsWith('landscape') || window.innerWidth > window.innerHeight;

            if (isLandscape && !document.fullscreenElement) {
                const player = playerContainer?.querySelector('media-player') || playerContainer;

                if (player?.requestFullscreen) {
                    player.requestFullscreen().catch(() => {
                        console.log("Auto-fullscreen blocked by browser policy without user gesture.");
                    });
                }
            }
            else if (!isLandscape && document.fullscreenElement) {
                if (document.exitFullscreen) {
                    document.exitFullscreen().catch(() => {});
                }
            }
        };

        const handleFullscreenChange = () => {
            const screenOrientation = window.screen.orientation as any;

            if (document.fullscreenElement) {
                if (screenOrientation?.lock) {
                    screenOrientation.lock('landscape').catch(() => {});
                }
            } else {
                if (screenOrientation?.lock) {
                    screenOrientation.lock('portrait').catch(() => {}).finally(() => {
                        if (screenOrientation?.unlock) {
                            screenOrientation.unlock();
                        }
                    });
                } else if (screenOrientation?.unlock) {
                    screenOrientation.unlock();
                }
            }
        };

        window.addEventListener('orientationchange', handleOrientationChange);
        document.addEventListener('fullscreenchange', handleFullscreenChange);

        return () => {
            window.removeEventListener('orientationchange', handleOrientationChange);
            document.removeEventListener('fullscreenchange', handleFullscreenChange);
        };
    });

    $effect(() => {
        if (m3u8Url && 'mediaSession' in navigator && animeData) {
            const meta = primaryMetadata(animeData, appConfig.data?.content?.preferredMetadataProvider);
            const coverImage = meta?.coverImage || meta?.bannerImage || "";

            const setMediaSession = (artworkArray: any[]) => {
                navigator.mediaSession.metadata = new MediaMetadata({
                    title: episodeTitle || i18n.t('watch.episode'),
                    artist: animeTitle,
                    album: 'Hoshi',
                    artwork: artworkArray
                });
            };

            if (coverImage) {
                fetch(coverImage)
                    .then(res => res.blob())
                    .then(blob => {
                        const reader = new FileReader();
                        reader.onloadend = () => {
                            setMediaSession([{ src: reader.result as string }]);
                        };
                        reader.readAsDataURL(blob);
                    })
                    .catch(() => {
                        setMediaSession([{ src: coverImage }]);
                    });
            } else {
                setMediaSession([]);
            }
        }

        return () => {
            if ('mediaSession' in navigator) navigator.mediaSession.metadata = null;
        };
    });
</script>

<svelte:head>
    <title>{episodeTitle} - {animeTitle}</title>
</svelte:head>

{#snippet TopBar()}
    <div class="custom-top-bar absolute top-0 inset-x-0 z-50 p-4 lg:p-6 hidden lg:flex landscape:hidden lg:landscape:flex flex-col xl:flex-row items-start xl:items-center justify-between gap-4 pointer-events-none bg-gradient-to-b from-black/80 via-black/40 to-transparent transition-opacity duration-300"
         style="padding-top: max(2rem, calc(env(safe-area-inset-top) + 0.5rem)); padding-left: max(1rem, env(safe-area-inset-left)); padding-right: max(1rem, env(safe-area-inset-right));">

        <div class="pointer-events-auto flex items-center gap-3 lg:gap-4 text-left min-w-0 shrink-0">
            <Button variant="ghost" size="icon" href={`/content/${cid}`} class="rounded-xl bg-black/40 hover:bg-white/20 text-white border border-white/10 backdrop-blur-md h-11 w-11 shrink-0">
                <ChevronLeft class="size-6 text-primary" />
            </Button>
            <div class="flex flex-col drop-shadow-lg min-w-0 max-w-[40vw]">
                <h1 class="font-black text-base lg:text-lg leading-tight truncate text-white/95">{animeTitle || i18n.t('watch.loading')}</h1>
                <p class="text-xs lg:text-sm font-bold text-primary truncate uppercase tracking-wider">{episodeTitle}</p>
            </div>
        </div>

        <div class="pointer-events-auto flex items-center gap-2 shrink-0">
            {#if !isLoadingMeta}
                <div class="flex items-center bg-black/40 border border-white/10 p-1 rounded-xl backdrop-blur-md">
                    <Button variant="ghost" size="icon" disabled={!hasPrev} href={`/watch/${cid}/${epNumber - 1}`} class="h-9 w-9 text-white hover:text-primary"><SkipBack class="size-4" /></Button>
                    <div class="w-px h-5 bg-white/20 mx-1"></div>
                    <Button variant="ghost" size="icon" disabled={!hasNext} href={`/watch/${cid}/${epNumber + 1}`} class="h-9 w-9 text-white hover:text-primary"><SkipForward class="size-4" /></Button>
                </div>

                <div class="flex items-center bg-black/40 border border-white/10 p-1.5 rounded-xl backdrop-blur-md shadow-lg shrink-0">
                    <div class="relative flex items-center gap-2 bg-transparent hover:bg-white/10 px-3 h-9 rounded-lg transition-colors overflow-hidden cursor-pointer">
                        <PuzzleIcon class="size-4 text-primary shrink-0 pointer-events-none" />
                        <span class="truncate text-xs lg:text-sm text-white/90 font-semibold pointer-events-none">
                            {selectedExtension ?? i18n.t('watch.select_extension')}
                        </span>
                        <select class="absolute inset-0 w-full h-full opacity-0 cursor-pointer appearance-none" onchange={(e) => selectExtension(e.currentTarget.value)}>
                            {#if !selectedExtension}<option value="" disabled selected></option>{/if}
                            {#each extensions as ext}<option value={ext} selected={selectedExtension === ext} class="text-black bg-white">{ext}</option>{/each}
                        </select>
                    </div>

                    <div class="w-px h-6 bg-white/20 mx-0.5"></div>

                    <div class="relative flex items-center gap-2 bg-transparent hover:bg-white/10 px-3 h-9 rounded-lg transition-colors overflow-hidden cursor-pointer">
                        <Settings2 class="size-4 text-primary shrink-0 pointer-events-none" />
                        <span class="truncate text-xs lg:text-sm text-white/90 font-semibold pointer-events-none">
                            {selectedServer ?? i18n.t('watch.auto_server')}
                        </span>
                        <select class="absolute inset-0 w-full h-full opacity-0 cursor-pointer appearance-none" onchange={(e) => { selectedServer = e.currentTarget.value; loadPlay(); }}>
                            {#if !selectedServer}<option value="" disabled selected></option>{/if}
                            {#each servers as srv}<option value={srv} selected={selectedServer === srv} class="text-black bg-white">{srv}</option>{/each}
                        </select>
                    </div>

                    {#if supportsDub}
                        <div class="w-px h-6 bg-white/20 mx-0.5"></div>
                        <div class="flex items-center gap-2 px-3 h-9">
                            <Mic2 class="size-4 text-primary" />
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
    <div class="lg:hidden flex flex-col p-4 gap-6 bg-background"
         style="padding-bottom: max(1.5rem, env(safe-area-inset-bottom)); padding-left: max(1rem, env(safe-area-inset-left)); padding-right: max(1rem, env(safe-area-inset-right));">

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


<div class="flex flex-col landscape:block lg:block w-full h-full bg-background lg:bg-black landscape:bg-black">

    <div class="w-full aspect-video landscape:aspect-auto landscape:absolute landscape:inset-0 landscape:w-full landscape:h-full lg:aspect-auto lg:w-full lg:h-full lg:absolute lg:inset-0 bg-black flex items-center justify-center relative z-10 shrink-0 shadow-lg lg:shadow-none landscape:shadow-none"
         style="padding-left: env(safe-area-inset-left); padding-right: env(safe-area-inset-right);">

        {#if !isLoadingMeta && !m3u8Url}
            {@render TopBar()}
        {/if}

        {#if error}
            <div class="flex flex-col items-center gap-6 p-6 z-20 max-w-md text-center animate-in fade-in zoom-in duration-300">
                <div class="p-4 rounded-full bg-destructive/10 ring-1 ring-destructive/20">
                    <AlertCircle class="w-12 h-12 text-destructive" />
                </div>

                <div class="space-y-2">
                    <p class="text-white/90 text-xl font-black">
                        {i18n.t(error.key) || error.message || error.key}
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
            <div class="flex flex-col items-center justify-center p-6 gap-4 text-center z-20 animate-in fade-in zoom-in duration-300">
                <PuzzleIcon class="size-16 text-white/30" />
                <span class="text-white/90 text-xl font-bold">{i18n.t('watch.no_extensions')}</span>
            </div>

        {:else if m3u8Url}
            <div class="w-full h-full" bind:this={playerContainer}>
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

    <div class="flex-1 overflow-y-auto lg:hidden landscape:hidden w-full relative">
        {@render MobileControls()}
    </div>
</div>

{#if animeData}
    <ExtensionManager
            bind:open={showExtensionManager}
            {cid}
            metadata={primaryMetadata(animeData, appConfig.data?.content?.preferredMetadataProvider)}
            isNsfw={animeData.content?.nsfw ?? false}
            extensions={animeData.extensionSources ?? []}
            contentType="anime"
            onSuccess={() => loadPageData(cid, epNumber)}
    />
{/if}
<style>
    :global(media-player:not([data-controls]) .custom-top-bar) {
        opacity: 0 !important;
        pointer-events: none !important;
    }

    :global(div > .custom-top-bar) {
        opacity: 1 !important;
        pointer-events: auto !important;
    }

    :global(media-player) {
        --media-focus-ring: none;
    }
</style>