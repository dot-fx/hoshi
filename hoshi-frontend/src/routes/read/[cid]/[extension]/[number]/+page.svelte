<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade, slide } from "svelte/transition";

    import { contentApi } from "$lib/api/content/content";
    import type { ContentUnit } from "$lib/api/content/types";

    import { Button } from "$lib/components/ui/button";
    import { Slider } from "$lib/components/ui/slider";
    import * as Drawer from "$lib/components/ui/drawer";
    import { Loader2, AlertCircle, ChevronLeft, Settings2, ArrowLeftRight, MonitorDown } from "lucide-svelte";

    const params = $derived(page.params as Record<string, string>);
    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));
    const PROXY_BASE = "/api/proxy";

    // --- DATA STATES ---
    let title = $state("");
    let chapterTitle = $state("");
    let images = $state<{url: string, headers?: Record<string,string>}[]>([]);
    let allChapters = $state<ContentUnit[]>([]);

    let isLoading = $state(true);
    let error = $state<string | null>(null);

    // --- RESPONSIVE STATE ---
    let innerWidth = $state(0);
    let isMobile = $derived(innerWidth < 1024); // lg de tailwind

    // --- READING STATES (UI) ---
    let showSettings = $state(false);

    // --- READER CONFIG ---
    let layout = $state<"scroll" | "paged">("scroll");
    let pagesPerView = $state<1 | 2>(1);
    let direction = $state<"ltr" | "rtl">("ltr");
    let fitMode = $state<"width" | "height">("width");

    // Gaps separados para X e Y
    let gapXArr = $state([0]);
    let gapX = $derived(gapXArr[0]);
    let gapYArr = $state([0]);
    let gapY = $derived(gapYArr[0]);

    // Derived: Navigation
    let hasNextChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber + 1));
    let hasPrevChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber - 1));

    // Agrupación de imágenes
    let groupedImages = $derived.by(() => {
        if (pagesPerView === 1) return images.map(img => [img]);

        let groups = [];
        for (let i = 0; i < images.length; i += 2) {
            const img1 = images[i];
            const img2 = images[i + 1] || null;

            if (direction === "rtl") {
                groups.push([img2, img1]);
            } else {
                groups.push([img1, img2]);
            }
        }
        return groups;
    });

    let currentGroupIndex = $state(0);

    $effect(() => {
        if (!isLoading) {
            localStorage.setItem("hoshi-reader-config", JSON.stringify({
                layout, pagesPerView, direction, fitMode, gapX: gapXArr[0], gapY: gapYArr[0]
            }));
        }
    });

    function proxifyImage(url: string, headers?: Record<string, string>): string {
        const params = new URLSearchParams({ url });
        if (headers) {
            if (headers["Referer"]) params.set("referer", headers["Referer"]);
            if (headers["Origin"]) params.set("origin", headers["Origin"]);
            if (headers["User-Agent"]) params.set("userAgent", headers["User-Agent"]);
        }
        return `${PROXY_BASE}?${params.toString()}`;
    }

    onMount(async () => {
        const savedConfig = localStorage.getItem("hoshi-reader-config");
        if (savedConfig) {
            try {
                const parsed = JSON.parse(savedConfig);
                if (parsed.layout) layout = parsed.layout;
                if (parsed.pagesPerView) pagesPerView = parsed.pagesPerView;
                if (parsed.direction) direction = parsed.direction;
                if (parsed.fitMode) fitMode = parsed.fitMode;

                if (parsed.gapX !== undefined) gapXArr = [parsed.gapX];
                else if (parsed.imageGap !== undefined) gapXArr = [parsed.imageGap];

                if (parsed.gapY !== undefined) gapYArr = [parsed.gapY];
                else if (parsed.imageGap !== undefined) gapYArr = [parsed.imageGap];

            } catch (e) {}
        }
        await loadChapter();
    });

    async function loadChapter() {
        isLoading = true;
        error = null;
        currentGroupIndex = 0;

        const mainContainer = document.getElementById("reader-main-container");
        if (mainContainer) mainContainer.scrollTop = 0;

        try {
            const [contentRes, playRes] = await Promise.all([
                contentApi.get(cid || ""),
                contentApi.play(cid || "", extension || "", chapterNumber)
            ]);
            title = contentRes.data.title ?? "";
            allChapters = (contentRes.data.contentUnits ?? []).filter(u => u.contentType === "chapter");
            const currentUnit = allChapters.find(u => u.unitNumber === chapterNumber);
            chapterTitle = currentUnit?.title ? `Ch. ${chapterNumber} - ${currentUnit.title}` : `Chapter ${chapterNumber}`;

            if (!playRes.success || playRes.type !== "reader") throw new Error("No reader data available");

            const data: any = playRes.data;
            const rawImages = Array.isArray(data) ? data : (data.pages || data.images || []);

            images = rawImages.map((img: any) => {
                if (typeof img === "string") return { url: proxifyImage(img) };
                return { url: proxifyImage(img.url, data.headers ?? img.headers) };
            });

            if (images.length === 0) throw new Error("No images found in chapter");
        } catch (e: any) {
            error = e?.message ?? "Failed to load chapter";
        } finally {
            isLoading = false;
        }
    }

    function turnPage(dir: "next" | "prev") {
        if (layout === "scroll") return;

        if (dir === "next") {
            if (currentGroupIndex < groupedImages.length - 1) {
                currentGroupIndex++;
            } else if (hasNextChapter) {
                goto(`/read/${cid}/${extension}/${chapterNumber + 1}`);
            }
        } else {
            if (currentGroupIndex > 0) {
                currentGroupIndex--;
            } else if (hasPrevChapter) {
                goto(`/read/${cid}/${extension}/${chapterNumber - 1}`);
            }
        }

        const mainContainer = document.getElementById("reader-main-container");
        if (mainContainer) mainContainer.scrollTop = 0;
    }

    function handleZoneClick(e: MouseEvent) {
        if (layout === "scroll" || isMobile) return;

        const readerEl = document.getElementById("reader-main-container");
        if (!readerEl) return;

        const rect = readerEl.getBoundingClientRect();
        const clickX = e.clientX - rect.left;
        const width = rect.width;
        const margin = width * 0.3;

        if (clickX < margin) {
            turnPage(direction === "rtl" ? "next" : "prev");
        } else if (clickX > width - margin) {
            turnPage(direction === "rtl" ? "prev" : "next");
        }
    }

    function handleMobileZoneClick(e: TouchEvent | MouseEvent) {
        if (layout === "scroll" || !isMobile) return;
        const width = window.innerWidth;
        const clickX = 'touches' in e ? e.touches[0].clientX : (e as MouseEvent).clientX;
        const margin = width * 0.3;

        if (clickX < margin) {
            turnPage(direction === "rtl" ? "next" : "prev");
        } else if (clickX > width - margin) {
            turnPage(direction === "rtl" ? "prev" : "next");
        }
    }
</script>

<svelte:window bind:innerWidth />

<svelte:head>
    <title>{chapterTitle} — {title}</title>
</svelte:head>

{#snippet SettingsContent()}
    <div class="space-y-6">
        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2"><MonitorDown class="size-4"/> Layout</span>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={layout === 'scroll' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => layout = 'scroll'}>Scroll</Button>
                <Button variant={layout === 'paged' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => layout = 'paged'}>Paged</Button>
            </div>
        </div>

        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2"><ArrowLeftRight class="size-4"/> Direction & Pages</span>
            <div class="grid grid-cols-2 gap-2 mb-2">
                <Button variant={pagesPerView === 1 ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => { pagesPerView = 1; currentGroupIndex = 0; }}>1 Page</Button>
                <Button variant={pagesPerView === 2 ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => { pagesPerView = 2; currentGroupIndex = 0; }}>2 Pages</Button>
            </div>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={direction === 'ltr' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => direction = 'ltr'}>LTR (➔)</Button>
                <Button variant={direction === 'rtl' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => direction = 'rtl'}>RTL (⬅)</Button>
            </div>
        </div>

        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Image Fit</span>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={fitMode === 'width' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => fitMode = 'width'}>Fit Width</Button>
                <Button variant={fitMode === 'height' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => fitMode = 'height'}>Fit Height</Button>
            </div>
        </div>

        <div class="space-y-5 pt-2 border-t border-border/40">
            <div>
                <div class="flex items-center justify-between mb-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Gap X (Horizontal)</span>
                    <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{gapX}px</span>
                </div>
                <Slider bind:value={gapXArr} max={100} step={2} class="w-full" />
            </div>

            {#if layout === 'scroll'}
                <div transition:slide>
                    <div class="flex items-center justify-between mb-3">
                        <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Gap Y (Vertical)</span>
                        <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{gapY}px</span>
                    </div>
                    <Slider bind:value={gapYArr} max={100} step={2} class="w-full" />
                </div>
            {/if}
        </div>
    </div>
{/snippet}


<div class="min-h-screen bg-background text-foreground flex flex-col h-screen overflow-hidden">

    <header class="z-40 bg-background/95 backdrop-blur-md border-b border-border/50 p-2 shadow-sm shrink-0 h-[56px] flex items-center">
        <div class="flex items-center justify-between gap-4 w-full px-2 lg:px-6">
            <div class="flex items-center gap-3 overflow-hidden">
                <Button variant="ghost" size="icon" href={cid ? `/content/${cid}` : '/'} class="rounded-full size-9 shrink-0">
                    <ChevronLeft class="size-5" />
                </Button>
                <div class="flex flex-col truncate">
                    <h1 class="font-bold text-sm leading-tight truncate">{title || 'Loading...'}</h1>
                    <p class="text-xs text-muted-foreground truncate mt-0.5">{chapterTitle || 'Please wait'}</p>
                </div>
            </div>

            <div class="flex items-center gap-3 shrink-0">
                {#if layout === 'paged' && !isLoading && !error}
                    <div class="text-xs font-medium text-muted-foreground bg-muted px-2.5 py-1 rounded-md border border-border/50">
                        {currentGroupIndex + 1} / {groupedImages.length}
                    </div>
                {/if}
                <Button variant={showSettings ? 'secondary' : 'ghost'} size="icon" disabled={isLoading || !!error} class="rounded-full size-9" onclick={() => showSettings = !showSettings}>
                    <Settings2 class="size-4" />
                </Button>
            </div>
        </div>
    </header>

    <div class="flex flex-1 overflow-hidden relative">

        {#if isLoading}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background">
                <Loader2 class="w-10 h-10 text-primary animate-spin" />
                <span class="text-muted-foreground font-medium tracking-wide">Loading pages...</span>
            </div>
        {:else if error}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                <AlertCircle class="w-12 h-12 text-destructive" />
                <p class="text-foreground text-lg font-medium">{error}</p>
                <Button variant="secondary" onclick={loadChapter}>Retry</Button>
            </div>
        {:else}
            <main
                    id="reader-main-container"
                    class="flex-1 bg-muted/10 overflow-y-auto overflow-x-hidden relative transition-all"
                    onclick={handleZoneClick}
                    onmouseup={handleMobileZoneClick}
                    aria-hidden="true"
            >
                {#if layout === "scroll"}
                    <div class="flex flex-col items-center w-full py-6 pb-24" style="row-gap: {gapY}px;">
                        {#each groupedImages as group}
                            <div class="flex justify-center w-full px-2 md:px-6" style="column-gap: {gapX}px;">
                                {#if group[0]}
                                    <img
                                            src={group[0].url}
                                            alt="Page"
                                            loading="lazy"
                                            class="select-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-56px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''} {fitMode === 'width' && pagesPerView === 1 ? 'max-w-[800px]' : ''}"
                                    />
                                {/if}
                                {#if group[1]}
                                    <img
                                            src={group[1].url}
                                            alt="Page"
                                            loading="lazy"
                                            class="select-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-56px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''}"
                                    />
                                {/if}
                            </div>
                        {/each}

                        <div class="w-full max-w-md mx-auto pt-16 px-4 flex justify-between gap-4">
                            <Button variant="outline" disabled={!hasPrevChapter} href={`/read/${cid}/${extension}/${chapterNumber - 1}`} class="flex-1 bg-background">Previous</Button>
                            <Button variant="default" disabled={!hasNextChapter} href={`/read/${cid}/${extension}/${chapterNumber + 1}`} class="flex-1">Next</Button>
                        </div>
                    </div>

                {:else}
                    {@const group = groupedImages[currentGroupIndex]}
                    <div class="flex items-center justify-center w-full min-h-full py-6 px-2 md:px-6" style="column-gap: {gapX}px;">
                        {#if group}
                            {#if group[0]}
                                <img
                                        src={group[0].url}
                                        alt="Page Left"
                                        class="select-none pointer-events-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-100px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''} {fitMode === 'width' && pagesPerView === 1 ? 'max-w-[1000px]' : ''}"
                                />
                            {/if}
                            {#if group[1]}
                                <img
                                        src={group[1].url}
                                        alt="Page Right"
                                        class="select-none pointer-events-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-100px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''}"
                                />
                            {/if}
                        {/if}
                    </div>
                {/if}
            </main>

            {#if !isMobile && showSettings}
                <aside
                        transition:slide={{axis: 'x', duration: 300}}
                        class="w-[320px] shrink-0 border-l border-border/50 bg-card overflow-y-auto shadow-2xl z-30"
                >
                    <div class="p-6">
                        <h2 class="font-semibold text-lg border-b border-border/40 pb-4 mb-6">Settings</h2>
                        {@render SettingsContent()}
                    </div>
                </aside>
            {/if}
        {/if}
    </div>

    {#if isMobile && !isLoading && !error}
        <Drawer.Root bind:open={showSettings}>
            <Drawer.Content class="bg-background/95 backdrop-blur-xl border-border/50">
                <Drawer.Header>
                    <Drawer.Title>Reader Settings</Drawer.Title>
                </Drawer.Header>
                <div class="p-4 pb-8">
                    {@render SettingsContent()}
                </div>
            </Drawer.Content>
        </Drawer.Root>
    {/if}

</div>