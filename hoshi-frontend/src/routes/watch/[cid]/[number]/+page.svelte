<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade } from "svelte/transition";
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

    const cid = $derived(page.params.cid);
    const epNumber = $derived(Number(page.params.number));

    let animeTitle = $state("");
    let episodeTitle = $state("");
    let extensions = $state<string[]>([]);
    let selectedExtension = $state<string | null>(null);
    let servers = $state<string[]>([]);
    let supportsDub = $state(false);
    let selectedServer = $state<string | null>(null);
    let isDub = $state(false);
    let animeData = $state<any>(null);

    // Estados de sincronización y carga
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

        // Reset de flags de sincronización para el nuevo episodio
        lastSyncTime = 0;
        hasUpdatedList = false;

        try {
            // Buscamos progreso guardado para setear el tiempo inicial
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
            error = e?.message || "Error";
        } finally {
            isLoadingPlay = false;
        }
    }

    function handlePlayerProgress({ currentTime, duration }: { currentTime: number; duration: number }) {
        if (!appConfig.data) return;

        // Sincronizar progreso (throttle 10s)
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

        // Actualizar lista al 80%
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
            if (targetCid !== currentLoadedCid) {
                isLoadingMeta = true;
                const contentRes = await contentApi.get(targetCid);
                const meta = primaryMetadata(contentRes);
                animeTitle = meta?.title ?? "";
                animeData = contentRes;

                const globalExtensions = extensionsStore.anime.map(e => e.id);
                const contentExtensions = contentRes.extensionSources?.map((e: any) => e.extensionName) || [];

                extensions = contentExtensions.length > 0
                    ? contentExtensions.filter((e: string) => globalExtensions.includes(e))
                    : globalExtensions;

                currentLoadedCid = targetCid;
                if (extensions.length > 0) await selectExtension(extensions[0]);
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
    <div class="custom-top-bar absolute top-0 inset-x-0 z-50 p-4 md:p-6 flex flex-col xl:flex-row items-start xl:items-center justify-between gap-4 pointer-events-none bg-gradient-to-b from-black/80 via-black/40 to-transparent transition-opacity duration-300">
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
                            <Switch
                                    id="dub-switch"
                                    checked={isDub}
                                    onCheckedChange={(v) => { isDub = v; loadPlay(); }}
                                    disabled={isLoadingPlay}
                                    class="scale-90"
                            />
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
{/snippet}

<div class="absolute inset-0 bg-black flex items-center justify-center overflow-hidden">
    {#if error}
        <div class="flex flex-col items-center gap-5 p-6 z-10">
            <AlertCircle class="w-12 h-12 text-destructive" />
            <p class="text-white/90 text-lg font-bold">{error}</p>
            <Button variant="destructive" onclick={loadPlay}>{i18n.t('watch.retry')}</Button>
        </div>
    {:else if !isLoadingMeta && extensions.length === 0}
        <div class="absolute inset-0 z-10 flex flex-col items-stretch">
            {@render TopBar()}
            <div class="flex-1 flex items-center justify-center">
                <Empty.Root>
                    <Empty.Title>{i18n.t('watch.no_extensions')}</Empty.Title>
                    <Button variant="secondary" onclick={() => goto("/marketplace")}>{i18n.t('marketplace.title')}</Button>
                </Empty.Root>
            </div>
        </div>
    {:else}
        <div class="w-full h-full">
            <Player
                    src={m3u8Url ?? ""}
                    {animeTitle}
                    {episodeTitle}
                    {subtitles}
                    {chapters}
                    {cid}
                    episode={epNumber}
                    initialTime={initialTime}
                    onTimeUpdate={handlePlayerProgress}
                    onEnded={() => hasNext && goto(`/watch/${cid}/${epNumber + 1}`)}
            >
                {@render TopBar()}
            </Player>
        </div>
    {/if}
</div>

<style>
    :global(media-player:not([data-controls]) .custom-top-bar) {
        opacity: 0 !important;
        pointer-events: none !important;
    }
</style>