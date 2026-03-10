<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade } from "svelte/transition";
    import { untrack } from "svelte";
    import { i18n } from "$lib/i18n/index.svelte";

    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { buildProxyUrl, proxyApi } from "$lib/api/proxy/proxy";
    import { isTauri } from "$lib/api/client";
    import AnimePlayer from "$lib/components/AnimePlayer.svelte";
    import type { Subtitle, Chapter } from "$lib/components/AnimePlayer.svelte";

    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import * as Empty from "$lib/components/ui/empty";
    import { AlertCircle, Loader2, PuzzleIcon, ChevronLeft, Settings2, MonitorPlay, Mic2, SkipBack, SkipForward } from "lucide-svelte";
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

    let totalEpsFromUnits = $derived.by(() => {
        if (!animeData?.contentUnits) return null;
        return animeData.contentUnits.filter((u: any) => u.contentType === "episode").length;
    });

    let totalEpsFromExtension = $derived.by(() => {
        if (!animeData?.extensionSources || !selectedExtension) return null;
        const ext = animeData.extensionSources.find((e: any) => e.extensionName === selectedExtension);
        // El tipado de ExtensionSource no garantiza .metadata, así que usamos un cast seguro
        return (ext as any)?.metadata?.episodes || null;
    });

    // ACTUALIZADO: Buscar epsOrChapters en la metadata principal, no en la raíz
    let totalEpsFromMeta = $derived.by(() => {
        if (!animeData) return null;
        const meta = primaryMetadata(animeData);
        return meta?.epsOrChapters || null;
    });

    let totalEpisodes = $derived(totalEpsFromUnits ?? totalEpsFromMeta ?? totalEpsFromExtension ?? 0);

    let hasNext = $derived(totalEpisodes > 0 && epNumber < totalEpisodes);
    let hasPrev = $derived(epNumber > 1);

    let m3u8Url = $state<string | null>(null);
    let subtitles = $state<Subtitle[]>([]);
    let chapters = $state<Chapter[]>([]);
    let isLoadingMeta = $state(true);
    let isLoadingPlay = $state(false);
    let error = $state<string | null>(null);

    let currentLoadedCid = $state<string | null>(null);
    let currentLoadedEp = $state<number | null>(null);

    let subtitleBlobUrls: string[] = [];

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

        try {
            const opts: { server?: string; category?: string } = {};
            if (selectedServer) opts.server = selectedServer;
            if (supportsDub && isDub) opts.category = "dub";

            const res = await contentApi.play(cid || "", selectedExtension, epNumber, opts);

            if (res.type !== "video") throw new Error(i18n.t('no_reader_data'));

            const data = res.data as any;
            const headers = data.headers ?? {};
            if (!data.source?.url) throw new Error("No stream URL");

            m3u8Url = buildProxyUrl({ url: data.source.url, ...extractHeaders(headers) });

            subtitles = await Promise.all(
                (data.source.subtitles ?? []).map(async (s: any) => {
                    const proxyParams = { url: s.url, ...extractHeaders(headers) };
                    if (!isTauri()) {
                        return { ...s, url: buildProxyUrl(proxyParams) };
                    }
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
            console.error("Error en loadPlay:", e);
            error = typeof e === 'string'
                ? e
                : (e?.message ?? i18n.t('something_went_wrong'));
        } finally {
            isLoadingPlay = false;
        }
    }

    function extractHeaders(headers: Record<string, string>) {
        return {
            referer:   headers["Referer"]    ?? undefined,
            origin:    headers["Origin"]     ?? undefined,
            userAgent: headers["User-Agent"] ?? undefined,
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

    function onExtensionChange(value: string) {
        selectExtension(value);
    }

    function onServerChange(value: string) {
        selectedServer = value;
        loadPlay();
    }

    function onDubToggle(checked: boolean) {
        isDub = checked;
        loadPlay();
    }

    function updateEpisodeTitle(ep: number) {
        const unit = animeData?.contentUnits?.find(
            (u: any) => u.unitNumber === ep && u.contentType === "episode"
        );
        episodeTitle = unit?.title ? `${i18n.t('episode')} ${ep} - ${unit.title}` : `${i18n.t('episode')} ${ep}`;
    }

    $effect(() => {
        const _cid = cid;
        const _epNumber = epNumber;

        if (_cid && _epNumber && (_cid !== currentLoadedCid || _epNumber !== currentLoadedEp)) {
            untrack(() => {
                loadPageData(_cid, _epNumber);
            });
        }
    });

    $effect(() => {
        return () => revokeSubtitleBlobs();
    });

    async function loadPageData(targetCid: string, targetEp: number) {
        error = null;

        try {
            if (targetCid !== currentLoadedCid) {
                isLoadingMeta = true;
                const [contentRes, extRes] = await Promise.all([
                    contentApi.get(targetCid),
                    extensionsApi.getAnime(),
                ]);

                // ACTUALIZADO: Extraer el título usando primaryMetadata
                const meta = primaryMetadata(contentRes);
                animeTitle = meta?.title ?? "";
                animeData = contentRes;

                // ACTUALIZADO: Extraer los nombres de las extensiones vinculadas al contenido.
                // Si contentRes.extensionSources existe, usamos esos; si no, caemos en la respuesta global
                const contentExtensions = contentRes.extensionSources?.map(e => e.extensionName) || [];
                const globalExtensions = (extRes as any)?.extensions ?? extRes ?? [];

                // Priorizamos las extensiones que el usuario ya vinculó, pero nos aseguramos que estén instaladas
                extensions = contentExtensions.length > 0
                    ? contentExtensions.filter(e => globalExtensions.includes(e))
                    : globalExtensions;

                updateEpisodeTitle(targetEp);

                currentLoadedCid = targetCid;
                currentLoadedEp = targetEp;

                if (extensions.length > 0) {
                    await selectExtension(extensions[0]);
                }

                isLoadingMeta = false;
            } else {
                updateEpisodeTitle(targetEp);
                currentLoadedEp = targetEp;
                await loadPlay();
            }
        } catch (e: any) {
            console.error("Error en loadPageData:", e);
            error = typeof e === 'string'
                ? e
                : (e?.message ?? i18n.t('something_went_wrong'));
            isLoadingMeta = false;
        }
    }
</script>

{#snippet TopBar()}
    <div class="custom-top-bar absolute top-0 inset-x-0 z-50 p-4 md:p-6 flex flex-col xl:flex-row items-start xl:items-center justify-between gap-4 pointer-events-none bg-gradient-to-b from-black/80 via-black/40 to-transparent transition-opacity duration-300">

        <div class="pointer-events-auto flex items-center gap-3 md:gap-4 text-left min-w-0 shrink-0">
            <Button variant="ghost" size="icon" href={`/content/${cid}`} class="rounded-xl bg-black/40 hover:bg-white/20 text-white border border-white/10 backdrop-blur-md h-11 w-11 shrink-0">
                <ChevronLeft class="size-6" />
                <span class="sr-only">{i18n.t('back_to_anime')}</span>
            </Button>

            <div class="flex flex-col drop-shadow-lg min-w-0 max-w-[40vw] xl:max-w-[30vw]">
                <h1 class="font-black text-base md:text-lg leading-tight truncate text-white/95 tracking-tight">
                    {animeTitle || i18n.t('loading')}
                </h1>
                <p class="text-xs md:text-sm font-bold text-white/60 truncate mt-0.5 uppercase tracking-wider">
                    {episodeTitle}
                </p>
            </div>
        </div>

        <div class="pointer-events-auto flex items-center flex-wrap xl:flex-nowrap justify-end gap-2 shrink-0 max-w-full">

            {#if !isLoadingMeta}
                <div class="flex items-center bg-black/40 border border-white/10 p-1 rounded-xl backdrop-blur-md shadow-lg shrink-0">
                    <Button
                            variant="ghost"
                            size="icon"
                            disabled={!hasPrev}
                            href={`/watch/${cid}/${epNumber - 1}`}
                            class="h-9 w-9 rounded-lg disabled:opacity-30 hover:bg-white/20 text-white transition-colors"
                    >
                        <SkipBack class="size-4" />
                    </Button>
                    <div class="w-px h-5 bg-white/20 mx-1"></div>
                    <Button
                            variant="ghost"
                            size="icon"
                            disabled={!hasNext}
                            href={`/watch/${cid}/${epNumber + 1}`}
                            class="h-9 w-9 rounded-lg disabled:opacity-30 hover:bg-white/20 text-white transition-colors"
                    >
                        <SkipForward class="size-4" />
                    </Button>
                </div>
            {/if}

            {#if !isLoadingMeta && extensions.length > 0}
                <div class="flex items-center bg-black/40 border border-white/10 p-1.5 rounded-xl backdrop-blur-md shadow-lg overflow-x-auto hide-scrollbar shrink-0">

                    <Select.Root type="single" value={selectedExtension ?? ""} onValueChange={(v) => onExtensionChange(v)}>
                        <Select.Trigger class="h-9 px-3 bg-transparent border-none text-white/90 hover:bg-white/10 focus:ring-0 shadow-none transition-all rounded-lg flex items-center gap-2 max-w-[150px] font-semibold">
                            <PuzzleIcon class="size-4 text-white/50 shrink-0" />
                            <span class="truncate text-left text-xs md:text-sm">
                                {selectedExtension ?? i18n.t('select_extension')}
                            </span>
                        </Select.Trigger>
                        <Select.Content class="bg-popover border-border backdrop-blur-xl shadow-xl min-w-[200px] z-[60] rounded-xl">
                            <Select.Group>
                                {#each extensions as ext}
                                    <Select.Item value={ext} label={ext} class="no-check-item relative flex w-full cursor-pointer select-none items-center rounded-lg py-2 px-3 text-sm font-semibold outline-none transition-colors">
                                        {ext}
                                    </Select.Item>
                                {/each}
                            </Select.Group>
                        </Select.Content>
                    </Select.Root>

                    <div class="w-px h-6 bg-white/20 shrink-0 mx-0.5"></div>

                    <Select.Root type="single" value={selectedServer ?? ""} onValueChange={(v) => onServerChange(v)}>
                        <Select.Trigger class="h-9 px-3 bg-transparent border-none text-white/90 hover:bg-white/10 focus:ring-0 shadow-none transition-all rounded-lg flex items-center gap-2 max-w-[130px] font-semibold">
                            <Settings2 class="size-4 text-white/50 shrink-0" />
                            <span class="truncate text-left text-xs md:text-sm">
                                {selectedServer ?? i18n.t('auto')}
                            </span>
                        </Select.Trigger>
                        <Select.Content class="bg-popover border-border backdrop-blur-xl shadow-xl min-w-[170px] z-[60] rounded-xl">
                            {#if servers.length > 0}
                                <Select.Group>
                                    {#each servers as srv}
                                        <Select.Item value={srv} label={srv} class="no-check-item relative flex w-full cursor-pointer select-none items-center rounded-lg py-2 px-3 text-sm font-semibold outline-none transition-colors">
                                            {srv}
                                        </Select.Item>
                                    {/each}
                                </Select.Group>
                            {:else}
                                <div class="px-2 py-4 text-center text-[10px] uppercase tracking-widest text-muted-foreground font-bold">
                                    {i18n.t('default_server')}
                                </div>
                            {/if}
                        </Select.Content>
                    </Select.Root>

                    {#if supportsDub}
                        <div class="w-px h-6 bg-white/20 shrink-0 mx-0.5"></div>
                        <div
                                class="flex items-center gap-2 shrink-0 px-3 h-9 bg-white/5 rounded-lg border border-transparent hover:bg-white/10 transition-colors group cursor-pointer"
                                role="button"
                                tabindex="0"
                                onclick={() => onDubToggle(!isDub)}
                                onkeydown={(e) => e.key === 'Enter' && onDubToggle(!isDub)}
                        >
                            <div class="flex items-center gap-1.5 pointer-events-none">
                                <Mic2 class="size-4 text-white/50 group-hover:text-white transition-colors" />
                                <Label for="dub-toggle" class="text-[10px] md:text-xs font-black uppercase tracking-widest text-white/70 group-hover:text-white transition-colors cursor-pointer">
                                    {i18n.t('dub')}
                                </Label>
                            </div>
                            <div
                                    class="pointer-events-auto"
                                    role="presentation"
                                    onclick={(e) => e.stopPropagation()}
                                    onkeydown={(e) => e.stopPropagation()}
                            >
                                <Switch id="dub-toggle" checked={isDub} onCheckedChange={onDubToggle} disabled={isLoadingPlay} class="data-[state=checked]:bg-primary scale-90 shadow-none border-white/20" />
                            </div>
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
{/snippet}

<svelte:head>
    <title>{episodeTitle} - {animeTitle}</title>
</svelte:head>

<div class="relative w-full h-full bg-black overflow-hidden flex items-center justify-center">

    <div class="absolute inset-0 z-0 flex items-center justify-center w-full h-full bg-black">

        {#if isLoadingMeta || isLoadingPlay}
            <div class="absolute inset-0 flex flex-col items-center justify-center gap-4 z-30 bg-black">
                <Loader2 class="w-12 h-12 text-white/70 animate-spin" />
                <span class="text-white/70 text-sm font-bold tracking-wide">
                    {isLoadingMeta ? i18n.t('loading_info') : i18n.t('connecting')}
                </span>
            </div>

        {:else if error}
            <div in:fade class="flex flex-col items-center justify-center gap-5 p-6 z-10 max-w-md">
                <div class="p-4 bg-destructive/10 rounded-2xl border border-destructive/20">
                    <AlertCircle class="w-12 h-12 text-destructive" />
                </div>
                <p class="text-white/90 text-lg text-center font-bold tracking-tight">{error}</p>
                <Button variant="destructive" onclick={loadPlay} class="mt-2 h-11 rounded-xl font-bold px-8 shadow-sm">
                    {i18n.t('retry_connection')}
                </Button>
            </div>

        {:else if extensions.length === 0}
            <div in:fade class="absolute inset-0 z-10 flex flex-col items-stretch">
                {@render TopBar()}
                <div class="flex-1 flex items-center justify-center p-6">
                    <Empty.Root class="border border-white/10 bg-white/5 rounded-2xl backdrop-blur-sm max-w-lg">
                        <Empty.Media variant="icon">
                            <PuzzleIcon class="size-16 text-white/40" />
                        </Empty.Media>
                        <Empty.Header>
                            <Empty.Title class="text-white text-2xl font-black">{i18n.t('no_extensions_found')}</Empty.Title>
                            <Empty.Description class="text-white/60 font-medium">
                                {i18n.t('please_install_extension')}
                            </Empty.Description>
                        </Empty.Header>
                        <Empty.Content>
                            <Button variant="secondary" onclick={() => goto("/settings/extensions")} class="h-11 rounded-xl font-bold shadow-sm">
                                {i18n.t('go_to_extensions')}
                            </Button>
                        </Empty.Content>
                    </Empty.Root>
                </div>
            </div>

        {:else if m3u8Url}
            <div class="w-full h-full bg-black animate-in fade-in duration-300">
                <AnimePlayer
                        src={m3u8Url}
                        {animeTitle}
                        {episodeTitle}
                        {subtitles}
                        {chapters}
                >
                    {@render TopBar()}
                </AnimePlayer>
            </div>

        {:else}
            <div class="z-10 flex items-center gap-3 text-white/40 bg-white/5 px-6 py-4 rounded-2xl border border-white/10 backdrop-blur-sm">
                <MonitorPlay class="size-6" />
                <span class="font-bold tracking-tight">{i18n.t('select_source_to_play')}</span>
            </div>
        {/if}
    </div>
</div>

<style>
    :global(.no-check-item span:first-child) { display: none !important; }
    :global(.no-check-item) { padding-left: 0.75rem !important; }
    :global(.no-check-item[data-state="checked"]) {
        background-color: var(--primary) !important;
        color: var(--primary-foreground) !important;
    }

    :global(media-player:not([data-controls]) .custom-top-bar) {
        opacity: 0 !important;
        pointer-events: none !important;
    }
</style>