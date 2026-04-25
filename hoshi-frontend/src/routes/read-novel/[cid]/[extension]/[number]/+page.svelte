<script lang="ts">
    import { NovelReaderState, NOVEL_THEMES } from "@/app/novel.svelte";
    import Reader from "@/components/layout/Reader.svelte";
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Type, AlignLeft, AlignJustify, Palette, Expand, Baseline, Space } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { NovelTheme, FontFamily } from "@/api/config/types";
    import NovelReaderSettings from "@/components/readers/NovelReaderSettings.svelte";

    const readerState = new NovelReaderState();
</script>

<svelte:head>
    <title>{readerState.chapterTitle} - {readerState.title}</title>
</svelte:head>

<Reader {readerState} contentType="novel">
    {#snippet settings()}
        <NovelReaderSettings {readerState} />
    {/snippet}

    <main
            id="novel-main-container"
            class="flex-1 overflow-y-auto overflow-x-hidden relative transition-colors duration-300"
            style="background-color: {readerState.themeColors.bg}; color: {readerState.themeColors.text};"
            onscroll={(e) => readerState.onScroll(e)}
    >
        <article
                class="mx-auto px-4 py-8 md:py-12 transition-all duration-300 {readerState.fontFamily === 'sans' ? 'font-sans' : readerState.fontFamily === 'serif' ? 'font-serif' : 'font-mono'}"
                style="
                max-width: {readerState.maxWidth}px;
                font-size: {readerState.fontSize}px;
                line-height: {readerState.lineHeight};
                text-align: {readerState.textAlign};
                --paragraph-spacing: {readerState.paragraphSpacing}em;
            "
        >
            <div class="prose max-w-none novel-content" style="color: inherit; text-align: inherit; line-height: inherit;">
                {@html readerState.contentHtml}
            </div>
            <div class="h-24 w-full"></div>
        </article>
    </main>
</Reader>

<style>
    :global(.novel-content *) {
        text-align: inherit;
        line-height: inherit;
        color: inherit;
    }
    :global(.novel-content p) {
        margin-bottom: var(--paragraph-spacing, 1.5em);
    }
</style>