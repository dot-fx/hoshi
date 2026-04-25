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
    import ReaderImage from "@/components/readers/ReaderImage.svelte";
    import MangaReaderSettings from "@/components/readers/MangaReaderSettings.svelte";

    const readerState = new MangaReaderState();
</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

<Reader
        {readerState}
        contentType="manga"
        currentProgress={readerState.layout === "paged" ? readerState.currentProgress : null}
>
    {#snippet settings()}
        <MangaReaderSettings {readerState} />
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
                            <ReaderImage imgEntry={group[0]} {readerState} customClass="..." customStyle="..." />
                        {/if}
                        {#if group[1]}
                            <ReaderImage imgEntry={group[0]} {readerState} customClass="..." customStyle="..." />
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
                                <ReaderImage imgEntry={group[0]} {readerState} customClass="..." customStyle="..." />
                            {/if}
                            {#if group[1]}
                                <ReaderImage imgEntry={group[0]} {readerState} customClass="..." customStyle="..." />
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