<script lang="ts">
    import { MangaReaderState } from "@/app/manga.svelte";
    import Reader from "@/components/layout/Reader.svelte";
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { ArrowLeftRight, GalleryVertical, BookOpen, Maximize } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { fly } from "svelte/transition";
    import type { MangaLayout } from "@/api/config/types";

    const readerState = new MangaReaderState();
</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

{#snippet imageWithPlaceholder(imgEntry, customClass, customStyle)}
    {@const status = readerState.imageStatus[imgEntry.id] || "loading"}
    <div class="relative flex items-center justify-center {customClass}" style={customStyle}>
        {#if status === "loading" || status === "error"}
            <div class="absolute inset-0 flex flex-col items-center justify-center bg-muted/10 animate-pulse rounded-lg">
                {#if status === "loading"}
                    <div class="size-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin"></div>
                {:else}
                    <div class="flex flex-col items-center gap-2 text-muted-foreground/50">
                        <span class="text-[10px] font-black uppercase tracking-tighter">{i18n.t("reader.error_loading")}</span>
                    </div>
                {/if}
            </div>
        {/if}
        <img
                src={imgEntry.url}
                alt={i18n.t("reader.page_alt")}
                draggable="false"
                loading="lazy"
                class="transition-all duration-500 {status === 'loaded' ? 'opacity-100 scale-100' : 'opacity-0 scale-95'} {customClass}"
                style={customStyle}
                onload={() => readerState.setImgStatus(imgEntry.id, "loaded")}
                onerror={() => readerState.setImgStatus(imgEntry.id, "error")}
                use:readerState.resolveBlobSrc={imgEntry}
                use:readerState.handleImgMount={imgEntry.id}
        />
    </div>
{/snippet}

<Reader
        {readerState}
        contentType="manga"
        currentProgress={readerState.layout === "paged" ? readerState.currentProgress : null}
>
    {#snippet settings()}
        <div class="space-y-6 px-1">
            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <GalleryVertical class="size-4"/> {i18n.t("reader.reading_mode")}
                </Label>
                <Tabs.Root value={readerState.layout} onValueChange={(v) => readerState.updateMangaConfig({ layout: v as MangaLayout })} class="w-full">
                    <Tabs.List class="grid w-full grid-cols-2 rounded-xl h-11 p-1 bg-muted/50">
                        <Tabs.Trigger value="scroll" class="rounded-lg gap-2 font-bold h-9"><GalleryVertical class="size-3"/>{i18n.t("reader.scroll")}</Tabs.Trigger>
                        <Tabs.Trigger value="paged" class="rounded-lg gap-2 font-bold h-9"><BookOpen class="size-3"/>{i18n.t("reader.paged")}</Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <ArrowLeftRight class="size-4"/> {i18n.t("reader.direction_and_pages")}
                </Label>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl mb-2">
                    <Button variant={readerState.pagesPerView === 1 ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => { readerState.updateMangaConfig({ pagesPerView: 1 }); readerState.currentGroupIndex = 0; }}>{i18n.t("reader.single_page")}</Button>
                    <Button variant={readerState.pagesPerView === 2 ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => { readerState.updateMangaConfig({ pagesPerView: 2 }); readerState.currentGroupIndex = 0; }}>{i18n.t("reader.double_page")}</Button>
                </div>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                    <Button variant={readerState.direction === "ltr" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ direction: "ltr" })}>LTR</Button>
                    <Button variant={readerState.direction === "rtl" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ direction: "rtl" })}>RTL</Button>
                </div>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <Maximize class="size-4"/> {i18n.t("reader.image_fit")}
                </Label>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                    <Button variant={readerState.fitMode === "width" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ fitMode: "width" })}>{i18n.t("reader.fit_width")}</Button>
                    <Button variant={readerState.fitMode === "height" ? "secondary" : "ghost"} class="text-sm h-9 font-bold" onclick={() => readerState.updateMangaConfig({ fitMode: "height" })}>{i18n.t("reader.fit_height")}</Button>
                </div>
            </div>

            <div class="space-y-5 pt-4 border-t border-border/40">
                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t("reader.gap_x")}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.gapX}px</span>
                    </div>
                    <Slider type="single" value={readerState.gapX} onValueChange={(v) => readerState.updateMangaConfig({ gapX: v })} max={100} step={2} class="w-full" />
                </div>
                {#if readerState.layout === "scroll"}
                    <div>
                        <div class="flex items-center justify-between mb-3">
                            <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground">{i18n.t("reader.gap_y")}</Label>
                            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{readerState.gapY}px</span>
                        </div>
                        <Slider type="single" value={readerState.gapY} onValueChange={(v) => readerState.updateMangaConfig({ gapY: v })} max={100} step={2} class="w-full" />
                    </div>
                {/if}
            </div>
        </div>
    {/snippet}

    <main
            id="reader-main-container"
            class="safe-reader flex-1 bg-muted/10 relative transition-all {readerState.layout === 'scroll' ? 'overflow-y-auto' : 'overflow-hidden'}"
            onclick={(e) => {
                readerState.handleZoneClick(e)
    if (readerState.layout === "scroll" || readerState.isSwiping) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const margin = rect.width * 0.4;

    if (clickX < margin) {
        readerState.turnPage(readerState.direction === "rtl" ? "next" : "prev");
    } else if (clickX > rect.width - margin) {
        readerState.turnPage(readerState.direction === "rtl" ? "prev" : "next");
    } else {
        readerState.showOverlay = !readerState.showOverlay;
    }
}}
            ontouchstart={(e) => readerState.handleTouchStart(e)}
            ontouchend={(e) => readerState.handleTouchEnd(e)}
    >
        {#if readerState.layout === "scroll"}
            <div class="flex flex-col items-center w-full py-6 pb-24" style="row-gap: {readerState.gapY}px;">
                {#each readerState.groupedImages as group}
                    <div class="flex justify-center items-center w-full px-2 md:px-6" style="column-gap: {readerState.gapX}px;">
                        {#if group[0]}
                            {@render imageWithPlaceholder(group[0],
                                `select-none object-contain shrink min-w-0
                                ${readerState.fitMode === "height" ? "max-h-[calc(100vh-60px)] w-auto" : ""}
                                ${readerState.fitMode === "width" && readerState.pagesPerView === 2 ? "flex-1 h-auto" : ""}
                                ${readerState.fitMode === "width" && readerState.pagesPerView === 1 ? "w-full max-w-[1000px] h-auto" : ""}`,
                                readerState.fitMode === "height" && readerState.pagesPerView === 2 ? `max-width: calc(50% - ${readerState.gapX / 2}px);` : ""
                            )}
                        {/if}
                        {#if group[1]}
                            {@render imageWithPlaceholder(group[1],
                                `select-none object-contain shrink min-w-0
                                ${readerState.fitMode === "height" ? "max-h-[calc(100vh-60px)] w-auto" : ""}
                                ${readerState.fitMode === "width" ? "flex-1 h-auto" : ""}`,
                                readerState.fitMode === "height" ? `max-width: calc(50% - ${readerState.gapX / 2}px);` : ""
                            )}
                        {/if}
                    </div>
                {/each}
            </div>
        {:else}
            <div class="grid w-full min-h-full place-items-center relative">
                {#key readerState.currentGroupIndex}
                    {@const group = readerState.groupedImages[readerState.currentGroupIndex]}
                    <div
                            class="col-start-1 row-start-1 flex items-center justify-center w-full min-h-full py-6 px-2 md:px-6"
                            style="column-gap: {readerState.gapX}px;"
                            in:fly={{ x: 150 * readerState.animDir, duration: 300 }}
                            out:fly={{ x: -150 * readerState.animDir, duration: 300 }}
                    >
                        {#if group}
                            {#if group[0]}
                                {@render imageWithPlaceholder(group[0],
                                    `select-none pointer-events-none object-contain shrink min-w-0
                                    ${readerState.fitMode === "height" ? "max-h-[85dvh] w-auto" : ""}
                                    ${readerState.fitMode === "width" && readerState.pagesPerView === 2 ? "flex-1 h-auto" : ""}
                                    ${readerState.fitMode === "width" && readerState.pagesPerView === 1 ? "w-full max-w-[1000px] h-auto" : ""}`,
                                    readerState.fitMode === "height" && readerState.pagesPerView === 2 ? `max-width: calc(50% - ${readerState.gapX / 2}px);` : ""
                                )}
                            {/if}
                            {#if group[1]}
                                {@render imageWithPlaceholder(group[1],
                                    `select-none pointer-events-none object-contain shrink min-w-0
                                    ${readerState.fitMode === "height" ? "max-h-[85dvh] w-auto" : ""}
                                    ${readerState.fitMode === "width" ? "flex-1 h-auto" : ""}`,
                                    readerState.fitMode === "height" ? `max-width: calc(50% - ${readerState.gapX / 2}px);` : ""
                                )}
                            {/if}
                        {/if}
                    </div>
                {/key}
            </div>
        {/if}
    </main>

    {#if readerState.layout === "paged" && readerState.showOverlay && readerState.groupedImages.length > 0}
        <div
                class="fixed bottom-6 left-6 right-6 z-[100] md:hidden pb-[env(safe-area-inset-bottom)]"
                transition:fly={{ y: 20, duration: 250 }}
                onclick={(e) => e.stopPropagation()}
        >
            <div class="bg-background/40 backdrop-blur-xl shadow-none border-none rounded-3xl p-4 flex flex-col gap-4">
                <div class="flex justify-center">
                    <span class="text-foreground/80 bg-foreground/5 backdrop-blur-md px-3 py-1 rounded-full font-mono text-xs font-bold">
                        {readerState.currentGroupIndex + 1} <span class="opacity-30 mx-0.5">/</span> {readerState.groupedImages.length}
                    </span>
                </div>
                <div class="px-2">
                    <Slider
                            value={[readerState.currentGroupIndex]}
                            min={0}
                            max={readerState.groupedImages.length - 1}
                            step={1}
                            dir={readerState.direction}
                            onValueChange={(v) => {
                            readerState.skipAnimation = false;
                            readerState.updatePageWithDir(v[0]);
                        }}
                            class="w-full"
                    />
                </div>
                <div class="flex justify-between px-2 -mt-2 text-[9px] font-black uppercase tracking-widest text-foreground/40">
                    <span>{readerState.direction === "rtl" ? i18n.t("reader.end") : i18n.t("reader.start")}</span>
                    <span>{readerState.direction === "rtl" ? i18n.t("reader.start") : i18n.t("reader.end")}</span>
                </div>
            </div>
        </div>
    {/if}
</Reader>

<style>
    .safe-reader {
        padding-top: calc(env(safe-area-inset-top) + 0.5rem) !important;
        padding-bottom: calc(env(safe-area-inset-bottom) + 2rem) !important;
        padding-left: env(safe-area-inset-left) !important;
        padding-right: env(safe-area-inset-right) !important;
        touch-action: pan-y pinch-zoom;
    }
</style>