<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade } from "svelte/transition";

    import { contentApi } from "$lib/api/content/content";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import AnimePlayer from "$lib/components/AnimePlayer.svelte";
    import type { Subtitle, Chapter } from "$lib/components/AnimePlayer.svelte";

    import { Button } from "$lib/components/ui/button";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Empty, EmptyContent, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from "$lib/components/ui/empty";
    import { AlertCircle, Loader2, PuzzleIcon, ChevronLeft, Settings2, MonitorPlay, Mic2, SkipBack, SkipForward } from "lucide-svelte";
    import type { ContentUnit } from "$lib/api/content/types";

    const PROXY_BASE = "/api/proxy";
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
    let extensionData = $state<any>(null);
    let totalEpsFromUnits = $derived.by(() => {
        if (!animeData?.contentUnits) return null;
        return animeData.contentUnits.filter((u: any) => u.contentType === "episode").length;
    });
    let totalEpsFromExtension = $derived.by(() => {
        if (!animeData?.extensionSources || !selectedExtension) return null;
        const ext = animeData.extensionSources.find((e: any) => e.extensionName === selectedExtension);
        return ext?.metadata?.episodes || null;
    });

    let totalEpsFromMeta = $derived(animeData?.epsOrChapters || null);
    let totalEpisodes = $derived(totalEpsFromUnits ?? totalEpsFromMeta ?? totalEpsFromExtension ?? 0);

    let hasNext = $derived(totalEpisodes > 0 && epNumber < totalEpisodes);
    let hasPrev = $derived(epNumber > 1);

    let m3u8Url = $state<string | null>(null);
    let subtitles = $state<Subtitle[]>([]);
    let chapters = $state<Chapter[]>([]);
    let isLoadingMeta = $state(true);
    let isLoadingPlay = $state(false);
    let error = $state<string | null>(null);

    function proxify(url: string, headers?: Record<string, string>): string {
        const params = new URLSearchParams({ url });

        if (headers) {
            if (headers["Referer"]) params.set("referer", headers["Referer"]);
            if (headers["Origin"]) params.set("origin", headers["Origin"]);
            if (headers["User-Agent"]) params.set("userAgent", headers["User-Agent"]);
        }

        return `${PROXY_BASE}?${params.toString()}#.m3u8`;
    }

    async function loadPlay() {
        if (!selectedExtension) return;
        isLoadingPlay = true;
        m3u8Url = null;
        error = null;

        try {
            const opts: { server?: string; category?: string } = {};
            if (selectedServer) opts.server = selectedServer;
            if (supportsDub && isDub) opts.category = "dub";

            const res = await contentApi.play(cid || "", selectedExtension, epNumber, opts);
            if (!res.success || res.type !== "video") throw new Error("No video stream available");

            const data = res.data as any;
            const headers = data.headers ?? {};
            if (!data.source?.url) throw new Error("No stream URL");

            m3u8Url = proxify(data.source.url, headers);
            subtitles = (data.source.subtitles ?? []).map((s: any) => ({
                ...s,
                url: proxify(s.url, headers),
            }));
            chapters = data.source.chapters ?? [];
        } catch (e: any) {
            error = e?.message ?? "Failed to load episode";
        } finally {
            isLoadingPlay = false;
        }
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

    onMount(async () => {
        isLoadingMeta = true;
        error = null;

        try {
            const [contentRes, extRes] = await Promise.all([
                contentApi.get(cid || ""),
                extensionsApi.getAnime(),
            ]);

            animeTitle = contentRes.data.title ?? "";
            animeData = contentRes.data;
            const unit = animeData.contentUnits?.find(
                (u: any) => u.unitNumber === epNumber && u.contentType === "episode"
            );
            episodeTitle = unit?.title ? `Episode ${epNumber} - ${unit.title}` : `Episode ${epNumber}`;

            extensions = extRes.extensions ?? [];
            if (extensions.length > 0) await selectExtension(extensions[0]);
        } catch (e: any) {
            error = e?.message ?? "Failed to load content";
        } finally {
            isLoadingMeta = false;
        }
    });
</script>

{#snippet TopBar()}
    <div class="custom-top-bar absolute top-0 inset-x-0 z-50 p-4 md:p-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 pointer-events-none bg-gradient-to-b from-black/70 to-transparent transition-opacity duration-300">

        <div class="pointer-events-auto flex items-center gap-3 md:gap-4 text-left">
            <Button variant="ghost" size="icon" href={`/content/${cid}`} class="rounded-full bg-black/40 hover:bg-white/10 text-white border border-white/10 backdrop-blur-md size-10 md:size-11 shrink-0">
                <ChevronLeft class="size-5 md:size-6" />
                <span class="sr-only">Back to anime</span>
            </Button>

            <div class="flex flex-col drop-shadow-md max-w-[40vw] sm:max-w-[50vw]">
                <h1 class="font-bold text-sm md:text-base leading-tight truncate text-white/90">
                    {animeTitle || 'Loading...'}
                </h1>
                <p class="text-xs text-white/50 truncate mt-0.5">
                    {episodeTitle}
                </p>
            </div>
        </div>

        <div class="pointer-events-auto flex items-center gap-2 flex-wrap justify-end">

            {#if !isLoadingMeta}
                <div class="flex items-center gap-1 bg-background/60 border border-border/40 p-1 rounded-2xl backdrop-blur-xl shadow-2xl transition-all hover:border-border/80 mr-2">
                    <Button
                            variant="ghost"
                            size="icon"
                            disabled={!hasPrev}
                            href={`/watch/${cid}/${epNumber - 1}`}
                            class="h-8 w-8 rounded-xl disabled:opacity-30"
                    >
                        <SkipBack class="size-4" />
                    </Button>
                    <div class="w-px h-4 bg-border/40"></div>
                    <Button
                            variant="ghost"
                            size="icon"
                            disabled={!hasNext}
                            href={`/watch/${cid}/${epNumber + 1}`}
                            class="h-8 w-8 rounded-xl disabled:opacity-30"
                    >
                        <SkipForward class="size-4" />
                    </Button>
                </div>
            {/if}

        </div>

        {#if !isLoadingMeta && extensions.length > 0}
            <div class="pointer-events-auto flex items-center gap-2 bg-background/60 border border-border/40 p-1.5 rounded-2xl backdrop-blur-xl shadow-2xl max-full overflow-x-auto custom-scrollbar transition-all hover:border-border/80">

                <Select.Root type="single" value={selectedExtension ?? ""} onValueChange={(v) => onExtensionChange(v)}>
                    <Select.Trigger class="h-9 px-3 bg-transparent border-none text-foreground hover:bg-muted/40 focus:ring-0 shadow-none transition-all rounded-lg flex items-center gap-2 min-w-[170px]">
                        <PuzzleIcon class="size-3.5 text-muted-foreground shrink-0" />
                        <span class="truncate text-left font-medium text-xs">
                            {selectedExtension ?? "Select Extension"}
                        </span>
                    </Select.Trigger>
                    <Select.Content class="bg-popover border-border backdrop-blur-xl shadow-xl min-w-[200px] z-[60]">
                        <Select.Group>
                            {#each extensions as ext}
                                <Select.Item value={ext} label={ext} class="no-check-item relative flex w-full cursor-pointer select-none items-center rounded-md py-2 px-3 text-sm outline-none transition-colors">
                                    {ext}
                                </Select.Item>
                            {/each}
                        </Select.Group>
                    </Select.Content>
                </Select.Root>

                <div class="w-px h-6 bg-border/30 shrink-0"></div>

                <Select.Root type="single" value={selectedServer ?? ""} onValueChange={(v) => onServerChange(v)}>
                    <Select.Trigger class="h-9 px-3 bg-transparent border-none text-foreground hover:bg-muted/40 focus:ring-0 shadow-none transition-all rounded-lg flex items-center gap-2 min-w-[140px]">
                        <Settings2 class="size-3.5 text-muted-foreground shrink-0" />
                        <span class="truncate text-left font-medium text-xs">
                            {selectedServer ?? "Auto"}
                        </span>
                    </Select.Trigger>
                    <Select.Content class="bg-popover border-border backdrop-blur-xl shadow-xl min-w-[170px] z-[60]">
                        {#if servers.length > 0}
                            <Select.Group>
                                {#each servers as srv}
                                    <Select.Item value={srv} label={srv} class="no-check-item relative flex w-full cursor-pointer select-none items-center rounded-md py-2 px-3 text-sm outline-none transition-colors">
                                        {srv}
                                    </Select.Item>
                                {/each}
                            </Select.Group>
                        {:else}
                            <div class="px-2 py-4 text-center text-[10px] uppercase tracking-widest text-muted-foreground font-bold">
                                Default Server
                            </div>
                        {/if}
                    </Select.Content>
                </Select.Root>

                {#if supportsDub}
                    <div class="w-px h-6 bg-border/30 shrink-0"></div>
                    <div class="flex items-center gap-3 shrink-0 px-3 h-9 bg-muted/20 rounded-lg border border-transparent hover:border-border/20 transition-all ml-1 group">
                        <div class="flex items-center gap-2">
                            <Mic2 class="size-3.5 text-muted-foreground group-hover:text-foreground transition-colors" />
                            <Label for="dub-toggle" class="text-[0.65rem] font-bold uppercase tracking-widest text-muted-foreground group-hover:text-foreground cursor-pointer transition-colors">
                                Dub
                            </Label>
                        </div>
                        <Switch id="dub-toggle" checked={isDub} onCheckedChange={onDubToggle} disabled={isLoadingPlay} class="data-[state=checked]:bg-primary scale-90 shadow-sm" />
                    </div>
                {/if}
            </div>
        {/if}
    </div>
{/snippet}

<svelte:head>
    <title>{episodeTitle} — {animeTitle}</title>
</svelte:head>

<div class="relative w-full h-screen bg-black overflow-hidden flex items-center justify-center">

    <div class="absolute inset-0 z-0 flex items-center justify-center w-full h-full bg-black">
        {#if isLoadingMeta || isLoadingPlay}
            <div transition:fade class="absolute inset-0 flex flex-col items-center justify-center gap-4 z-30 bg-black">
                <Loader2 class="w-12 h-12 text-white/70 animate-spin" />
                <span class="text-white/70 text-sm font-medium tracking-wide">
                    {isLoadingMeta ? "Loading info..." : "Connecting..."}
                </span>
            </div>

        {:else if error}
            <div transition:fade class="flex flex-col items-center justify-center gap-4 p-6 z-10 max-w-md">
                <div class="p-4 bg-destructive/20 rounded-full border border-destructive/30">
                    <AlertCircle class="w-12 h-12 text-destructive" />
                </div>
                <p class="text-white/80 text-base text-center font-medium">{error}</p>
                <Button variant="destructive" onclick={loadPlay} class="mt-4">Retry connection</Button>
            </div>

        {:else if extensions.length === 0}
            <div transition:fade class="absolute inset-0 z-10 flex flex-col items-stretch">
                {@render TopBar()}
                <div class="flex-1 flex items-center justify-center p-6">
                    <Empty>
                        <EmptyMedia>
                            <PuzzleIcon class="size-16 text-white/30" />
                        </EmptyMedia>
                        <EmptyHeader>
                            <EmptyTitle class="text-white text-xl">No extensions found</EmptyTitle>
                            <EmptyDescription class="text-white/60">
                                Please install an extension to start watching.
                            </EmptyDescription>
                        </EmptyHeader>
                        <EmptyContent>
                            <Button variant="secondary" onclick={() => goto("/settings/extensions")}>
                                Go to Extensions
                            </Button>
                        </EmptyContent>
                    </Empty>
                </div>
            </div>

        {:else if m3u8Url}
            <div class="w-full h-full" transition:fade={{ duration: 500 }}>
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
            <div class="z-10 flex items-center gap-2 text-white/40">
                <MonitorPlay class="size-5" />
                <span>Select a source to play</span>
            </div>
        {/if}
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { display: none; }
    .custom-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }

    :global(.no-check-item span:first-child) { display: none !important; }
    :global(.no-check-item) { padding-left: 0.75rem !important; }
    :global(.no-check-item[data-state="checked"]) {
        background-color: var(--accent) !important;
        color: var(--accent-foreground) !important;
        font-weight: 600;
    }

    :global(media-player:not([data-controls]) .custom-top-bar) {
        opacity: 0 !important;
        pointer-events: none !important;
    }
</style>