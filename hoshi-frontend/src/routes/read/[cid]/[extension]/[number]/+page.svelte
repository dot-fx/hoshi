<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { fade } from "svelte/transition";

    import { contentApi } from "$lib/api/content/content";
    import type { ContentUnit } from "$lib/api/content/types";

    import { Button } from "$lib/components/ui/button";
    import { Loader2, AlertCircle, ChevronLeft, Settings2, SkipBack, SkipForward, ArrowLeft, ArrowRight } from "lucide-svelte";

    const params = $derived(page.params as Record<string, string>);

    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));
    const PROXY_BASE = "/api/proxy"; // Assuming you use the same proxy to bypass CORS

    // --- DATA STATES ---
    let title = $state("");
    let chapterTitle = $state("");
    let images = $state<{url: string, headers?: Record<string,string>}[]>([]);
    let allChapters = $state<ContentUnit[]>([]);

    let isLoading = $state(true);
    let error = $state<string | null>(null);

    // --- READING STATES (UI) ---
    let currentPage = $state(0);
    let showUI = $state(true);
    let showSettings = $state(false);

    // --- READER CONFIG (Saved in LocalStorage) ---
    // readMode: 'longstrip' (Webtoon/Manhwa) | 'paged-ltr' (Left to Right) | 'paged-rtl' (Classic Manga)
    let readMode = $state<"longstrip" | "paged-ltr" | "paged-rtl">("longstrip");
    // fitMode: 'width' | 'height' | 'original'
    let fitMode = $state<"width" | "height" | "original">("width");
    let imageGap = $state<"none" | "small" | "large">("none");

    // Navigation derived states
    let hasNextChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber + 1));
    let hasPrevChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber - 1));

    // Effect to sync config with LocalStorage
    $effect(() => {
        // Only save if we passed initial load
        if (!isLoading) {
            localStorage.setItem("hoshi-reader-config", JSON.stringify({
                readMode,
                fitMode,
                imageGap
            }));
        }
    });

    // Function to proxify images (vital for external manga sources)
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
        // Load saved config
        const savedConfig = localStorage.getItem("hoshi-reader-config");
        if (savedConfig) {
            try {
                const parsed = JSON.parse(savedConfig);
                if (parsed.readMode) readMode = parsed.readMode;
                if (parsed.fitMode) fitMode = parsed.fitMode;
                if (parsed.imageGap) imageGap = parsed.imageGap;
            } catch (e) {}
        }

        await loadChapter();
    });

    async function loadChapter() {
        isLoading = true;
        error = null;
        currentPage = 0;
        window.scrollTo(0, 0);

        try {
            // Load meta and pages in parallel
            const [contentRes, playRes] = await Promise.all([
                contentApi.get(cid || ""),
                contentApi.play(cid || "", extension || "", chapterNumber)
            ]);

            title = contentRes.data.title ?? "";

            // Filter chapters using your API types
            allChapters = (contentRes.data.contentUnits ?? []).filter(u => u.contentType === "chapter");

            const currentUnit = allChapters.find(u => u.unitNumber === chapterNumber);
            chapterTitle = currentUnit?.title ? `Ch. ${chapterNumber} - ${currentUnit.title}` : `Chapter ${chapterNumber}`;

            if (!playRes.success || playRes.type !== "reader") {
                throw new Error("No reader data available for this chapter");
            }

            // Assuming reader data comes in playRes.data.pages or playRes.data
            // Adapt this according to the exact structure your extension returns
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

    function toggleUI() {
        showUI = !showUI;
        showSettings = false;
    }

    // Paged mode navigation
    function turnPage(direction: "next" | "prev") {
        if (direction === "next" && currentPage < images.length - 1) {
            currentPage++;
        } else if (direction === "prev" && currentPage > 0) {
            currentPage--;
        } else if (direction === "next" && currentPage === images.length - 1 && hasNextChapter) {
            goto(`/read/${cid}/${extension}/${chapterNumber + 1}`);
        }
    }

    function handleZoneClick(e: MouseEvent) {
        if (readMode === "longstrip") {
            toggleUI();
            return;
        }

        const width = window.innerWidth;
        const clickX = e.clientX;
        const margin = width * 0.3; // 30% of the edge to change page

        if (clickX < margin) {
            // Left Click
            turnPage(readMode === "paged-rtl" ? "next" : "prev");
        } else if (clickX > width - margin) {
            // Right Click
            turnPage(readMode === "paged-rtl" ? "prev" : "next");
        } else {
            // Center Click
            toggleUI();
        }
    }
</script>

<svelte:head>
    <title>{chapterTitle} — {title}</title>
</svelte:head>

<div class="relative min-h-screen bg-black/95 text-foreground overflow-x-hidden" onclick={handleZoneClick}>

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
            <Button variant="ghost" href={`/content/${cid}`}>Back to content</Button>
        </div>
    {:else}

        <div class="fixed top-0 inset-x-0 z-40 p-4 bg-gradient-to-b from-black/90 to-transparent transition-transform duration-300 {showUI ? 'translate-y-0' : '-translate-y-full'}" onclick={(e) => e.stopPropagation()}>
            <div class="flex items-center justify-between gap-4 max-w-7xl mx-auto">
                <div class="flex items-center gap-3">
                    <Button variant="ghost" size="icon" href={`/content/${cid}`} class="rounded-full bg-black/40 hover:bg-white/10 text-white backdrop-blur-md size-10 shrink-0 border border-white/10">
                        <ChevronLeft class="size-5" />
                    </Button>
                    <div class="flex flex-col">
                        <h1 class="font-bold text-sm md:text-base leading-tight truncate text-white/90 max-w-[200px] md:max-w-[400px]">{title}</h1>
                        <p class="text-xs text-white/50 truncate mt-0.5">{chapterTitle}</p>
                    </div>
                </div>

                <Button variant="ghost" size="icon" class="rounded-full bg-black/40 text-white hover:bg-white/10 border border-white/10" onclick={() => showSettings = !showSettings}>
                    <Settings2 class="size-5" />
                </Button>
            </div>
        </div>

        {#if showSettings && showUI}
            <div transition:fade={{duration: 200}} class="fixed right-4 top-20 z-50 w-72 bg-popover/95 border border-border/50 backdrop-blur-xl rounded-2xl p-5 shadow-2xl flex flex-col gap-6" onclick={(e) => e.stopPropagation()}>

                <div class="space-y-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Reading Mode</span>
                    <div class="grid grid-cols-1 gap-2">
                        <Button variant={readMode === 'longstrip' ? 'secondary' : 'ghost'} class="justify-start text-sm h-9" onclick={() => readMode = 'longstrip'}>
                            Long Strip (Webtoon)
                        </Button>
                        <Button variant={readMode === 'paged-rtl' ? 'secondary' : 'ghost'} class="justify-start text-sm h-9" onclick={() => readMode = 'paged-rtl'}>
                            Manga (Right to Left)
                        </Button>
                        <Button variant={readMode === 'paged-ltr' ? 'secondary' : 'ghost'} class="justify-start text-sm h-9" onclick={() => readMode = 'paged-ltr'}>
                            Comic (Left to Right)
                        </Button>
                    </div>
                </div>

                <div class="space-y-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Image Fit</span>
                    <div class="grid grid-cols-3 gap-1 bg-muted/30 p-1 rounded-lg">
                        <Button variant={fitMode === 'width' ? 'secondary' : 'ghost'} size="sm" class="text-xs" onclick={() => fitMode = 'width'}>Width</Button>
                        <Button variant={fitMode === 'height' ? 'secondary' : 'ghost'} size="sm" class="text-xs" onclick={() => fitMode = 'height'}>Height</Button>
                        <Button variant={fitMode === 'original' ? 'secondary' : 'ghost'} size="sm" class="text-xs" onclick={() => fitMode = 'original'}>Original</Button>
                    </div>
                </div>

                {#if readMode === 'longstrip'}
                    <div class="space-y-3">
                        <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Spacing</span>
                        <div class="grid grid-cols-3 gap-1 bg-muted/30 p-1 rounded-lg">
                            <Button variant={imageGap === 'none' ? 'secondary' : 'ghost'} size="sm" class="text-xs" onclick={() => imageGap = 'none'}>0</Button>
                            <Button variant={imageGap === 'small' ? 'secondary' : 'ghost'} size="sm" class="text-xs" onclick={() => imageGap = 'small'}>S</Button>
                            <Button variant={imageGap === 'large' ? 'secondary' : 'ghost'} size="sm" class="text-xs" onclick={() => imageGap = 'large'}>L</Button>
                        </div>
                    </div>
                {/if}
            </div>
        {/if}

        <div class="w-full flex justify-center {readMode === 'longstrip' ? 'py-0' : 'min-h-screen items-center py-0'} transition-all">

            {#if readMode === "longstrip"}
                <div class="flex flex-col w-full items-center" style="gap: {imageGap === 'none' ? '0' : imageGap === 'small' ? '1rem' : '4rem'}; max-width: {fitMode === 'width' ? '1000px' : 'auto'};">
                    {#each images as img}
                        <img
                                src={img.url}
                                alt="Manga page"
                                loading="lazy"
                                class="select-none {fitMode === 'width' ? 'w-full h-auto' : fitMode === 'height' ? 'h-screen w-auto' : ''}"
                        />
                    {/each}

                    <div class="w-full max-w-md mx-auto py-12 px-4 flex justify-between gap-4" onclick={(e) => e.stopPropagation()}>
                        <Button variant="secondary" disabled={!hasPrevChapter} href={`/read/${cid}/${extension}/${chapterNumber - 1}`} class="flex-1">Previous Chapter</Button>
                        <Button variant="default" disabled={!hasNextChapter} href={`/read/${cid}/${extension}/${chapterNumber + 1}`} class="flex-1">Next Chapter</Button>
                    </div>
                </div>

            {:else}
                <div class="relative flex items-center justify-center w-full h-screen">
                    <img
                            src={images[currentPage].url}
                            alt={`Page ${currentPage + 1}`}
                            class="select-none object-contain max-w-full max-h-screen pointer-events-none {fitMode === 'width' ? 'w-full h-auto' : fitMode === 'height' ? 'h-[100vh] w-auto' : 'w-auto h-auto max-h-none max-w-none'}"
                    />
                </div>
            {/if}

        </div>

        <div class="fixed bottom-0 inset-x-0 z-40 p-4 md:p-6 bg-gradient-to-t from-black/90 to-transparent flex items-center justify-center pointer-events-none transition-transform duration-300 {showUI ? 'translate-y-0' : 'translate-y-full'}">

            <div class="pointer-events-auto flex items-center gap-4 bg-background/60 border border-border/40 px-4 py-2 rounded-2xl backdrop-blur-xl shadow-2xl">

                <Button variant="ghost" size="icon" disabled={!hasPrevChapter} href={`/read/${cid}/${extension}/${chapterNumber - 1}`} class="size-8 rounded-xl disabled:opacity-30 text-foreground">
                    <SkipBack class="size-4" />
                </Button>

                {#if readMode !== "longstrip"}
                    <div class="flex items-center gap-2 px-2 text-sm font-medium text-foreground min-w-[80px] justify-center">
                        {currentPage + 1} <span class="text-muted-foreground">/</span> {images.length}
                    </div>
                {/if}

                <Button variant="ghost" size="icon" disabled={!hasNextChapter} href={`/read/${cid}/${extension}/${chapterNumber + 1}`} class="size-8 rounded-xl disabled:opacity-30 text-foreground">
                    <SkipForward class="size-4" />
                </Button>
            </div>

        </div>

    {/if}
</div>