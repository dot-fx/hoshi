<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { contentApi } from "$lib/api/content/content";
    import type { ContentUnit } from "$lib/api/content/types";
    import { i18n } from '$lib/i18n/index.svelte'; // <-- Importar i18n

    import { Button } from "$lib/components/ui/button";
    import { Slider } from "$lib/components/ui/slider";
    import { ArrowLeftRight, MonitorDown } from "lucide-svelte";

    // IMPORTAR EL LAYOUT
    import ReaderLayout from "$lib/components/ReaderLayout.svelte";

    const params = $derived(page.params as Record<string, string>);
    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));
    const PROXY_BASE = "/api/proxy";

    let title = $state("");
    let chapterTitle = $state("");
    let images = $state<{url: string, headers?: Record<string,string>}[]>([]);
    let allChapters = $state<ContentUnit[]>([]);

    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let showSettings = $state(false);

    // --- MANGA CONFIG ---
    let layout = $state<"scroll" | "paged">("scroll");
    let pagesPerView = $state<1 | 2>(1);
    let direction = $state<"ltr" | "rtl">("ltr");
    let fitMode = $state<"width" | "height">("width");
    let gapXArr = $state([0]);
    let gapX = $derived(gapXArr[0]);
    let gapYArr = $state([0]);
    let gapY = $derived(gapYArr[0]);

    let hasNextChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber + 1));
    let hasPrevChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber - 1));

    let groupedImages = $derived.by(() => {
        if (pagesPerView === 1) return images.map(img => [img]);
        let groups = [];
        for (let i = 0; i < images.length; i += 2) {
            const img1 = images[i];
            const img2 = images[i + 1] || null;
            if (direction === "rtl") groups.push([img2, img1]);
            else groups.push([img1, img2]);
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
        const p = new URLSearchParams({ url });
        if (headers) {
            if (headers["Referer"]) p.set("referer", headers["Referer"]);
            if (headers["Origin"]) p.set("origin", headers["Origin"]);
            if (headers["User-Agent"]) p.set("userAgent", headers["User-Agent"]);
        }
        return `${PROXY_BASE}?${p.toString()}`;
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

            if (!playRes.success || playRes.type !== "reader") throw new Error(i18n.t('no_reader_data'));

            const data: any = playRes.data;
            const rawImages = Array.isArray(data) ? data : (data.pages || data.images || []);

            images = rawImages.map((img: any) => {
                if (typeof img === "string") return { url: proxifyImage(img) };
                return { url: proxifyImage(img.url, data.headers ?? img.headers) };
            });

            if (images.length === 0) throw new Error(i18n.t('no_images_found'));
        } catch (e: any) {
            error = e?.message ?? i18n.t('failed_load_chapter');
        } finally {
            isLoading = false;
        }
    }

    function turnPage(dir: "next" | "prev") {
        if (layout === "scroll") return;
        if (dir === "next") {
            if (currentGroupIndex < groupedImages.length - 1) currentGroupIndex++;
            else if (hasNextChapter) goto(`/read/${cid}/${extension}/${chapterNumber + 1}`);
        } else {
            if (currentGroupIndex > 0) currentGroupIndex--;
            else if (hasPrevChapter) goto(`/read/${cid}/${extension}/${chapterNumber - 1}`);
        }
        const mainContainer = document.getElementById("reader-main-container");
        if (mainContainer) mainContainer.scrollTop = 0;
    }

    function handleZoneClick(e: MouseEvent) {
        if (layout === "scroll") return;
        const readerEl = document.getElementById("reader-main-container");
        if (!readerEl) return;
        const rect = readerEl.getBoundingClientRect();
        const clickX = e.clientX - rect.left;
        const margin = rect.width * 0.3;

        if (clickX < margin) turnPage(direction === "rtl" ? "next" : "prev");
        else if (clickX > rect.width - margin) turnPage(direction === "rtl" ? "prev" : "next");
    }

    function handleMobileZoneClick(e: TouchEvent | MouseEvent) {
        if (layout === "scroll" || window.innerWidth >= 1024) return;
        const clickX = 'touches' in e ? e.touches[0].clientX : (e as MouseEvent).clientX;
        const margin = window.innerWidth * 0.3;
        if (clickX < margin) turnPage(direction === "rtl" ? "next" : "prev");
        else if (clickX > window.innerWidth - margin) turnPage(direction === "rtl" ? "prev" : "next");
    }
</script>

<svelte:head>
    <title>{chapterTitle} — {title}</title>
</svelte:head>

{#snippet MangaSettings()}
    <div class="space-y-6">
        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2"><MonitorDown class="size-4"/> {i18n.t('layout')}</span>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={layout === 'scroll' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => layout = 'scroll'}>{i18n.t('scroll')}</Button>
                <Button variant={layout === 'paged' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => layout = 'paged'}>{i18n.t('paged')}</Button>
            </div>
        </div>

        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2"><ArrowLeftRight class="size-4"/> {i18n.t('direction_pages')}</span>
            <div class="grid grid-cols-2 gap-2 mb-2">
                <Button variant={pagesPerView === 1 ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => { pagesPerView = 1; currentGroupIndex = 0; }}>{i18n.t('page_1')}</Button>
                <Button variant={pagesPerView === 2 ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => { pagesPerView = 2; currentGroupIndex = 0; }}>{i18n.t('pages_2')}</Button>
            </div>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={direction === 'ltr' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => direction = 'ltr'}>{i18n.t('ltr')}</Button>
                <Button variant={direction === 'rtl' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => direction = 'rtl'}>{i18n.t('rtl')}</Button>
            </div>
        </div>

        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t('image_fit')}</span>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={fitMode === 'width' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => fitMode = 'width'}>{i18n.t('fit_width')}</Button>
                <Button variant={fitMode === 'height' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => fitMode = 'height'}>{i18n.t('fit_height')}</Button>
            </div>
        </div>

        <div class="space-y-5 pt-2 border-t border-border/40">
            <div>
                <div class="flex items-center justify-between mb-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t('gap_x')}</span>
                    <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{gapX}px</span>
                </div>
                <Slider bind:value={gapXArr} max={100} step={2} class="w-full" />
            </div>

            {#if layout === 'scroll'}
                <div>
                    <div class="flex items-center justify-between mb-3">
                        <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t('gap_y')}</span>
                        <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{gapY}px</span>
                    </div>
                    <Slider bind:value={gapYArr} max={100} step={2} class="w-full" />
                </div>
            {/if}
        </div>
    </div>
{/snippet}

<ReaderLayout
        {isLoading}
        {error}
        {title}
        {chapterTitle}
        {cid}
        currentProgress={layout === 'paged' ? `${currentGroupIndex + 1} / ${groupedImages.length}` : null}
        bind:showSettings
        onRetry={loadChapter}
        settings={MangaSettings}
>
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
                            <img src={group[0].url} alt="Page" loading="lazy" class="select-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-56px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''} {fitMode === 'width' && pagesPerView === 1 ? 'max-w-[800px]' : ''}" />
                        {/if}
                        {#if group[1]}
                            <img src={group[1].url} alt="Page" loading="lazy" class="select-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-56px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''}" />
                        {/if}
                    </div>
                {/each}

                <div class="w-full max-w-md mx-auto pt-16 px-4 flex justify-between gap-4">
                    <Button variant="outline" disabled={!hasPrevChapter} href={`/read/${cid}/${extension}/${chapterNumber - 1}`} class="flex-1 bg-background">{i18n.t('previous')}</Button>
                    <Button variant="default" disabled={!hasNextChapter} href={`/read/${cid}/${extension}/${chapterNumber + 1}`} class="flex-1">{i18n.t('next')}</Button>
                </div>
            </div>
        {:else}
            {@const group = groupedImages[currentGroupIndex]}
            <div class="flex items-center justify-center w-full min-h-full py-6 px-2 md:px-6" style="column-gap: {gapX}px;">
                {#if group}
                    {#if group[0]}
                        <img src={group[0].url} alt="Page Left" class="select-none pointer-events-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-100px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''} {fitMode === 'width' && pagesPerView === 1 ? 'max-w-[1000px]' : ''}" />
                    {/if}
                    {#if group[1]}
                        <img src={group[1].url} alt="Page Right" class="select-none pointer-events-none object-contain shrink min-w-0 {fitMode === 'height' ? 'h-[calc(100vh-100px)] w-auto' : 'w-full h-auto'} {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 basis-0' : ''}" />
                    {/if}
                {/if}
            </div>
        {/if}
    </main>
</ReaderLayout>