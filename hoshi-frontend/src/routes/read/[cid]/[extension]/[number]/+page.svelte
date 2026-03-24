<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { contentApi } from "@/api/content/content";
    import { primaryMetadata } from "@/api/content/types";
    import { i18n } from '@/i18n/index.svelte.js';
    import { buildProxyUrl, proxyApi } from "@/api/proxy/proxy";
    import { isTauri } from "@/api/client";

    import { appConfig } from "@/config.svelte";
    import type { MangaConfig, MangaLayout } from "@/api/config/types";
    import { progressApi } from "@/api/progress/progress";
    import { listApi } from "@/api/list/list";

    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { ArrowLeftRight, GalleryVertical, BookOpen, Maximize } from "lucide-svelte";
    import Reader from "@/components/layout/Reader.svelte";

    const params = $derived(page.params as Record<string, string>);
    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));

    type ImageEntry = {
        url: string;
        proxyParams?: { url: string; referer?: string; origin?: string; userAgent?: string };
    };

    type ChapterItem = {
        number?: string | number;
        unitNumber?: string | number;
        title?: string;
        [key: string]: unknown;
    };

    let title = $state("");
    let chapterTitle = $state("");
    let images = $state<ImageEntry[]>([]);
    let allChapters = $state<ChapterItem[]>([]);

    let coverImage = $state<string | null>(null);

    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let showSettings = $state(false);

    let currentChapterIndex = $derived(allChapters.findIndex(c => Number(c.number ?? c.unitNumber) === chapterNumber));
    let nextChapterNum = $derived(currentChapterIndex >= 0 && currentChapterIndex < allChapters.length - 1 ? allChapters[currentChapterIndex + 1].unitNumber ?? allChapters[currentChapterIndex + 1].number : null);
    let prevChapterNum = $derived(currentChapterIndex > 0 ? allChapters[currentChapterIndex - 1].unitNumber ?? allChapters[currentChapterIndex - 1].number : null);

    const mangaConfig = $derived(appConfig.data?.manga);
    let layout = $derived(mangaConfig?.layout ?? "scroll");
    let pagesPerView = $derived(mangaConfig?.pagesPerView ?? 1);
    let direction = $derived(mangaConfig?.direction ?? "ltr");
    let fitMode = $derived(mangaConfig?.fitMode ?? "width");

    let gapXArr = $derived([mangaConfig?.gapX ?? 0]);
    let gapYArr = $derived([mangaConfig?.gapY ?? 0]);
    let gapX = $derived(mangaConfig?.gapX ?? 0);
    let gapY = $derived(mangaConfig?.gapY ?? 0);

    // --- ESTADOS DE PROGRESO ---
    let hasUpdatedList = $state(false);
    let hasMarkedCompleted = $state(false);

    $effect(() => {
        chapterNumber;
        untrack(() => {
            hasUpdatedList = false;
            hasMarkedCompleted = false;
        });
    });

    function handleProgress(ratio: number) {
        if (!hasMarkedCompleted && ratio >= 0.9) {
            hasMarkedCompleted = true;
            progressApi.updateChapterProgress({ cid, chapter: chapterNumber, completed: true })
                .catch(e => console.error("History completion sync failed", e));
        }

        if (!hasUpdatedList && ratio >= 0.8 && appConfig.data?.content.autoUpdateProgress) {
            hasUpdatedList = true;
            listApi.upsert({ cid, status: "CURRENT", progress: chapterNumber })
                .catch(e => console.error("List sync failed", e));
        }
    }

    async function updateMangaConfig(patch: Partial<MangaConfig>) {
        if (!appConfig.data?.manga) return;
        try {
            await appConfig.update({ manga: { ...appConfig.data.manga, ...patch } });
        } catch (err) {
            console.error("Error updating config:", err);
        }
    }

    let debounceTimer: ReturnType<typeof setTimeout>;
    $effect(() => {
        const currentX = gapX;
        const currentY = gapY;

        if (currentX !== mangaConfig?.gapX || currentY !== mangaConfig?.gapY) {
            clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {
                updateMangaConfig({ gapX: currentX, gapY: currentY });
            }, 500);
        }
    });

    let groupedImages = $derived.by(() => {
        if (pagesPerView === 1) return images.map(img => [img, null] as [ImageEntry, null]);
        const groups: [ImageEntry | null, ImageEntry | null][] = [];
        for (let i = 0; i < images.length; i += 2) {
            const img1 = images[i];
            const img2 = images[i + 1] ?? null;
            groups.push(direction === "rtl" ? [img2, img1] : [img1, img2]);
        }
        return groups;
    });

    let currentGroupIndex = $state(0);
    const blobCache = new Map<string, string>();

    // Paged progress tracking
    $effect(() => {
        if (layout === "paged" && groupedImages.length > 0) {
            const ratio = (currentGroupIndex + 1) / groupedImages.length;
            handleProgress(ratio);
        }
    });

    async function resolveImageUrl(img: ImageEntry): Promise<string> {
        if (!isTauri() || !img.proxyParams) return img.url;
        const key = img.proxyParams.url;
        if (blobCache.has(key)) return blobCache.get(key)!;
        const blob = await proxyApi.fetch(img.proxyParams);
        const blobUrl = URL.createObjectURL(blob);
        blobCache.set(key, blobUrl);
        return blobUrl;
    }

    function resolveBlobSrc(node: HTMLImageElement, img: ImageEntry) {
        if (isTauri() && img.proxyParams) {
            resolveImageUrl(img).then(url => { node.src = url; });
        }
        return {
            update(newImg: ImageEntry) {
                if (isTauri() && newImg.proxyParams) {
                    resolveImageUrl(newImg).then(url => { node.src = url; });
                }
            }
        };
    }

    function revokeBlobs() {
        blobCache.forEach(url => URL.revokeObjectURL(url));
        blobCache.clear();
    }

    function extractHeaders(headers: Record<string, string>) {
        return {
            referer:   headers["Referer"]    ?? undefined,
            origin:    headers["Origin"]     ?? undefined,
            userAgent: headers["User-Agent"] ?? undefined,
        };
    }

    $effect(() => () => revokeBlobs());

    onMount(async () => {
        if (appConfig.data?.manga) {
            gapX = appConfig.data.manga.gapX;
            gapY = appConfig.data.manga.gapY;
        }
    });

    let loadedId = $state("");
    $effect(() => {
        const currentId = `${cid}-${extension}-${chapterNumber}`;
        if (cid && extension && !isNaN(chapterNumber) && loadedId !== currentId) {
            loadedId = currentId;
            loadChapter(cid, extension, chapterNumber);
        }
    });

    async function loadChapter(currentCid: string, currentExt: string, currentChapterNum: number) {
        isLoading = true;
        error = null;
        currentGroupIndex = 0;
        revokeBlobs();
        document.getElementById("reader-main-container")?.scrollTo(0, 0);

        try {
            const [contentRes, itemsRes, playRes] = await Promise.all([
                contentApi.get(currentCid),
                contentApi.getItems(currentCid, currentExt),
                contentApi.play(currentCid, currentExt, currentChapterNum)
            ]);

            const meta = primaryMetadata(contentRes);
            title = meta?.title ?? "";
            coverImage = meta?.coverImage ?? null;

            const rawItems: any[] = Array.isArray(itemsRes) ? itemsRes : (itemsRes as any)?.data ?? [];
            allChapters = rawItems.sort((a, b) => Number(a.number ?? a.unitNumber) - Number(b.number ?? b.unitNumber));
            const currentUnit = allChapters.find(u => Number(u.number ?? u.unitNumber) === currentChapterNum);

            chapterTitle = currentUnit?.title
                ? i18n.t('reader.chapter_with_title', { num: currentChapterNum, title: currentUnit.title })
                : i18n.t('reader.chapter_number_fallback', { num: currentChapterNum });

            if (playRes.type !== "reader") throw new Error(i18n.t('reader.no_data'));

            const data: any = playRes.data;
            const rawImages = Array.isArray(data) ? data : (data.pages || data.images || []);
            const globalHeaders = data.headers ?? {};

            images = rawImages.map((img: string | { url: string; headers?: Record<string, string> }): ImageEntry => {
                const rawUrl = typeof img === "string" ? img : img.url;
                const headers = { ...globalHeaders, ...(typeof img !== "string" && img.headers ? img.headers : {}) };
                const proxyParams = { url: rawUrl, ...extractHeaders(headers) };

                return isTauri()
                    ? { url: "", proxyParams }
                    : { url: buildProxyUrl(proxyParams) };
            });

            if (images.length === 0) throw new Error(i18n.t('reader.no_images'));

            // REGISTRO INICIAL
            progressApi.updateChapterProgress({ cid: currentCid, chapter: currentChapterNum, completed: false })
                .catch(e => console.error("History sync failed", e));

        } catch (e: any) {
            error = e?.message;
        } finally {
            isLoading = false;
        }
    }

    function goToNextChapter() {
        if (nextChapterNum !== null) goto(`/read/${cid}/${extension}/${nextChapterNum}`);
    }

    function goToPrevChapter() {
        if (prevChapterNum !== null) goto(`/read/${cid}/${extension}/${prevChapterNum}`);
    }

    function turnPage(dir: "next" | "prev") {
        if (layout === "scroll") return;
        if (dir === "next") {
            if (currentGroupIndex < groupedImages.length - 1) currentGroupIndex++;
            else goToNextChapter();
        } else {
            if (currentGroupIndex > 0) currentGroupIndex--;
            else goToPrevChapter();
        }
        document.getElementById("reader-main-container")?.scrollTo(0, 0);
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
    <title>{chapterTitle} - {title}</title>
</svelte:head>

<Reader
        {isLoading}
        {error}
        {title}
        {chapterTitle}
        {cid}
        {extension}
        {coverImage} contentType="manga"
        currentChapter={chapterNumber}
        {allChapters}
        currentProgress={layout === 'paged' ? `${currentGroupIndex + 1} / ${groupedImages.length}` : null}
        bind:showSettings
        onRetry={() => loadChapter(cid, extension, chapterNumber)}
>
    {#snippet settings()}
        <div class="space-y-6 px-1">
            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <GalleryVertical class="size-4"/> {i18n.t('reader.reading_mode')}
                </Label>
                <Tabs.Root value={layout} onValueChange={(v) => updateMangaConfig({ layout: v as MangaLayout })} class="w-full">
                    <Tabs.List class="grid w-full grid-cols-2 rounded-xl h-11 p-1 bg-muted/50">
                        <Tabs.Trigger value="scroll" class="rounded-lg gap-2 font-bold h-9"><GalleryVertical class="size-3"/>{i18n.t('reader.scroll')}</Tabs.Trigger>
                        <Tabs.Trigger value="paged" class="rounded-lg gap-2 font-bold h-9"><BookOpen class="size-3"/>{i18n.t('reader.paged')}</Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <ArrowLeftRight class="size-4"/> {i18n.t('reader.direction_and_pages')}
                </Label>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl mb-2">
                    <Button variant={pagesPerView === 1 ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => { updateMangaConfig({ pagesPerView: 1 }); currentGroupIndex = 0; }}>{i18n.t('reader.single_page')}</Button>
                    <Button variant={pagesPerView === 2 ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => { updateMangaConfig({ pagesPerView: 2 }); currentGroupIndex = 0; }}>{i18n.t('reader.double_page')}</Button>
                </div>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                    <Button variant={direction === 'ltr' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateMangaConfig({ direction: 'ltr' })}>LTR</Button>
                    <Button variant={direction === 'rtl' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateMangaConfig({ direction: 'rtl' })}>RTL</Button>
                </div>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <Maximize class="size-4"/> {i18n.t('reader.image_fit')}
                </Label>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                    <Button variant={fitMode === 'width' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateMangaConfig({ fitMode: 'width' })}>{i18n.t('reader.fit_width')}</Button>
                    <Button variant={fitMode === 'height' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateMangaConfig({ fitMode: 'height' })}>{i18n.t('reader.fit_height')}</Button>
                </div>
            </div>

            <div class="space-y-5 pt-4 border-t border-border/40">
                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t('reader.gap_x')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{gapX}px</span>
                    </div>
                    <Slider type="single" bind:value={gapX} max={100} step={2} class="w-full" />
                </div>
                {#if layout === 'scroll'}
                    <div>
                        <div class="flex items-center justify-between mb-3">
                            <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t('reader.gap_y')}</Label>
                            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{gapY}px</span>
                        </div>
                        <Slider type="single" bind:value={gapY} max={100} step={2} class="w-full" />
                    </div>
                {/if}
            </div>
        </div>
    {/snippet}

    <main
            id="reader-main-container"
            class="flex-1 bg-muted/10 overflow-y-auto overflow-x-hidden relative transition-all"
            onclick={handleZoneClick}
            onmouseup={handleMobileZoneClick}
            aria-hidden="true"
            onscroll={(e) => {
                if (layout === "scroll") {
                    const target = e.currentTarget;
                    if (target.scrollHeight > 0) {
                        const ratio = (target.scrollTop + target.clientHeight) / target.scrollHeight;
                        handleProgress(ratio);
                    }
                }
            }}
    >
        {#if layout === "scroll"}
            <div class="flex flex-col items-center w-full py-6 pb-24" style="row-gap: {gapY}px;">
                {#each groupedImages as group}
                    <div class="flex justify-center items-center w-full px-2 md:px-6" style="column-gap: {gapX}px;">
                        {#if group[0]}
                            <img
                                    src={group[0].url}
                                    alt={i18n.t('reader.page_alt')}
                                    loading="lazy"
                                    class="select-none object-contain shrink min-w-0
                                {fitMode === 'height' ? 'max-h-[calc(100vh-60px)] w-auto' : ''}
                                {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 h-auto' : ''}
                                {fitMode === 'width' && pagesPerView === 1 ? 'w-full max-w-[1000px] h-auto' : ''}"
                                    style="{fitMode === 'height' && pagesPerView === 2 ? `max-width: calc(50% - ${gapX/2}px);` : ''}"
                                    use:resolveBlobSrc={group[0]}
                            />
                        {/if}
                        {#if group[1]}
                            <img
                                    src={group[1].url}
                                    alt={i18n.t('reader.page_alt')}
                                    loading="lazy"
                                    class="select-none object-contain shrink min-w-0
                                {fitMode === 'height' ? 'max-h-[calc(100vh-60px)] w-auto' : ''}
                                {fitMode === 'width' ? 'flex-1 h-auto' : ''}"
                                    style="{fitMode === 'height' ? `max-width: calc(50% - ${gapX/2}px);` : ''}"
                                    use:resolveBlobSrc={group[1]}
                            />
                        {/if}
                    </div>
                {/each}
            </div>
        {:else}
            {@const group = groupedImages[currentGroupIndex]}
            <div class="flex items-center justify-center w-full min-h-full py-6 px-2 md:px-6" style="column-gap: {gapX}px;">
                {#if group}
                    {#if group[0]}
                        <img
                                src={group[0].url}
                                alt={i18n.t('reader.page_left_alt')}
                                class="select-none pointer-events-none object-contain shrink min-w-0
                            {fitMode === 'height' ? 'max-h-[calc(100vh-100px)] w-auto' : ''}
                            {fitMode === 'width' && pagesPerView === 2 ? 'flex-1 h-auto' : ''}
                            {fitMode === 'width' && pagesPerView === 1 ? 'w-full max-w-[1000px] h-auto' : ''}"
                                style="{fitMode === 'height' && pagesPerView === 2 ? `max-width: calc(50% - ${gapX/2}px);` : ''}"
                                use:resolveBlobSrc={group[0]}
                        />
                    {/if}
                    {#if group[1]}
                        <img
                                src={group[1].url}
                                alt={i18n.t('reader.page_right_alt')}
                                class="select-none pointer-events-none object-contain shrink min-w-0
                            {fitMode === 'height' ? 'max-h-[calc(100vh-100px)] w-auto' : ''}
                            {fitMode === 'width' ? 'flex-1 h-auto' : ''}"
                                style="{fitMode === 'height' ? `max-width: calc(50% - ${gapX/2}px);` : ''}"
                                use:resolveBlobSrc={group[1]}
                        />
                    {/if}
                {/if}
            </div>
        {/if}
    </main>
</Reader>