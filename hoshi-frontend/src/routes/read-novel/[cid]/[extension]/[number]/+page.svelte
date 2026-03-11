<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { contentApi } from "@/api/content/content";
    import { primaryMetadata, type ContentUnit } from "@/api/content/types";
    import { i18n } from '@/i18n/index.svelte.js';

    import { appConfig } from "@/config.svelte.js";
    import type { NovelConfig } from "@/api/config/types";

    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Type, AlignLeft, AlignJustify, Palette, Expand, Baseline, Space } from "lucide-svelte";

    import ReaderLayout from "@/components/ReaderLayout.svelte";

    const params = $derived(page.params as Record<string, string>);
    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));

    let title = $state("");
    let chapterTitle = $state("");
    let contentHtml = $state<string>("");
    let allChapters = $state<any[]>([]);
    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let showSettings = $state(false);

    const novelConfig = $derived(appConfig.data?.novel);

    let theme = $derived(novelConfig?.theme ?? "dark");
    let fontFamily = $derived(novelConfig?.fontFamily ?? "sans");
    let textAlign = $derived(novelConfig?.textAlign ?? "left");

    let fontSizeArr = $state([novelConfig?.fontSize ?? 18]);
    let fontSize = $derived(fontSizeArr[0]);
    let lineHeightArr = $state([novelConfig?.lineHeight ?? 1.6]);
    let lineHeight = $derived(lineHeightArr[0]);
    let maxWidthArr = $state([novelConfig?.maxWidth ?? 800]);
    let maxWidth = $derived(maxWidthArr[0]);
    let paragraphSpacingArr = $state([novelConfig?.paragraphSpacing ?? 1.5]);
    let paragraphSpacing = $derived(paragraphSpacingArr[0]);

    async function updateNovelConfig(patch: Partial<NovelConfig>) {
        try {
            await appConfig.update({ novel: patch });
        } catch (err) {
            console.error("Error updating novel config:", err);
        }
    }

    let debounceTimer: any;
    $effect(() => {
        const currentSize = fontSizeArr[0];
        const currentLine = lineHeightArr[0];
        const currentWidth = maxWidthArr[0];
        const currentSpacing = paragraphSpacingArr[0];

        if (
            currentSize !== novelConfig?.fontSize ||
            currentLine !== novelConfig?.lineHeight ||
            currentWidth !== novelConfig?.maxWidth ||
            currentSpacing !== novelConfig?.paragraphSpacing
        ) {
            clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {
                updateNovelConfig({
                    fontSize: currentSize,
                    lineHeight: currentLine,
                    maxWidth: currentWidth,
                    paragraphSpacing: currentSpacing
                });
            }, 500);
        }
    });

    const themes = {
        light: { bg: "#fdfdfd", text: "#1a1a1a", border: "#e5e7eb" },
        dark: { bg: "#1a1a1a", text: "#e0e0e0", border: "#262626" },
        sepia: { bg: "#f4ecd8", text: "#5b4636", border: "#e2d7bf" },
        oled: { bg: "#000000", text: "#d1d5db", border: "#171717" }
    };

    onMount(async () => {
        if (appConfig.data?.novel) {
            fontSizeArr = [appConfig.data.novel.fontSize];
            lineHeightArr = [appConfig.data.novel.lineHeight];
            maxWidthArr = [appConfig.data.novel.maxWidth];
            paragraphSpacingArr = [appConfig.data.novel.paragraphSpacing ?? 1.5];
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

        const mainContainer = document.getElementById("novel-main-container");
        if (mainContainer) mainContainer.scrollTop = 0;

        try {
            const [contentRes, itemsRes, playRes] = await Promise.all([
                contentApi.get(currentCid),
                contentApi.getItems(currentCid, currentExt),
                contentApi.play(currentCid, currentExt, currentChapterNum)
            ]);

            const meta = primaryMetadata(contentRes);
            title = meta?.title ?? "";

            const rawItems: any[] = Array.isArray(itemsRes) ? itemsRes : (itemsRes as any)?.data ?? [];
            allChapters = rawItems.sort((a, b) => Number(a.number ?? a.unitNumber) - Number(b.number ?? b.unitNumber));

            const currentUnit = allChapters.find(u => Number(u.number ?? u.unitNumber) === currentChapterNum);

            chapterTitle = currentUnit?.title
                ? `${i18n.t('chapter')} ${currentChapterNum} - ${currentUnit.title}`
                : `${i18n.t('chapter')} ${currentChapterNum}`;

            if (playRes.type !== "reader" || !playRes.data) {
                throw new Error(i18n.t('no_reader_data'));
            }

            const data: any = playRes.data;
            contentHtml = data.html || data.text || data.content || data;

            if (!contentHtml) throw new Error(i18n.t('no_content_available'));

        } catch (e: any) {
            error = e?.message ?? i18n.t('failed_load_chapter');
        } finally {
            isLoading = false;
        }
    }
</script>

<svelte:head>
    <title>{chapterTitle} — {title}</title>
</svelte:head>

<!-- CORRECCIÓN: El snippet `settings` se define DENTRO de las etiquetas del Layout -->
<ReaderLayout
        {isLoading}
        {error}
        {title}
        {chapterTitle}
        {cid}
        {extension}
        contentType="novel"
        currentChapter={chapterNumber}
        {allChapters}
        bind:showSettings
        onRetry={() => loadChapter(cid, extension, chapterNumber)}
>
    <!-- Snippet explícito de Svelte 5 para las propiedades -->
    {#snippet settings()}
        <div class="space-y-6 px-1">
            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <Palette class="size-4"/> {i18n.t('theme')}
                </Label>
                <div class="grid grid-cols-2 gap-2">
                    {#each Object.entries(themes) as [t, colors]}
                        <Button
                                variant={theme === t ? 'default' : 'outline'}
                                class="text-sm h-10 font-bold border-border/50 relative overflow-hidden"
                                style="background-color: {theme === t ? '' : colors.bg}; color: {theme === t ? '' : colors.text};"
                                onclick={() => updateNovelConfig({ theme: t as any })}
                        >
                            <span class="capitalize">{t}</span>
                        </Button>
                    {/each}
                </div>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <Type class="size-4"/> {i18n.t('font_family')}
                </Label>
                <Tabs.Root value={fontFamily} onValueChange={(v) => updateNovelConfig({ fontFamily: v as any })} class="w-full">
                    <Tabs.List class="grid w-full grid-cols-3 rounded-xl h-11 p-1 bg-muted/50">
                        <Tabs.Trigger value="sans" class="rounded-lg font-sans font-bold h-9">Sans</Tabs.Trigger>
                        <Tabs.Trigger value="serif" class="rounded-lg font-serif font-bold h-9">Serif</Tabs.Trigger>
                        <Tabs.Trigger value="mono" class="rounded-lg font-mono font-bold h-9">Mono</Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    {i18n.t('alignment')}
                </Label>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                    <Button variant={textAlign === 'left' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateNovelConfig({ textAlign: 'left' })}><AlignLeft class="size-4 mr-2"/> {i18n.t('left')}</Button>
                    <Button variant={textAlign === 'justify' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateNovelConfig({ textAlign: 'justify' })}><AlignJustify class="size-4 mr-2"/> {i18n.t('justify')}</Button>
                </div>
            </div>

            <div class="space-y-5 pt-4 border-t border-border/40">
                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Type class="size-3.5"/> {i18n.t('font_size')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{fontSize}px</span>
                    </div>
                    <Slider bind:value={fontSizeArr} min={12} max={32} step={1} class="w-full" />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Baseline class="size-3.5"/> {i18n.t('line_height')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{lineHeight}</span>
                    </div>
                    <Slider bind:value={lineHeightArr} min={1} max={3} step={0.1} class="w-full" />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Space class="size-3.5"/> Paragraph Spacing</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{paragraphSpacing}em</span>
                    </div>
                    <Slider bind:value={paragraphSpacingArr} min={0.5} max={4} step={0.1} class="w-full" />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Expand class="size-3.5"/> {i18n.t('content_width')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{maxWidth}px</span>
                    </div>
                    <Slider bind:value={maxWidthArr} min={400} max={1200} step={50} class="w-full" />
                </div>
            </div>
        </div>
    {/snippet}

    <!-- Contenido principal implícito (Svelte lo convierte en la prop "children") -->
    <main
            id="novel-main-container"
            class="flex-1 overflow-y-auto overflow-x-hidden relative transition-colors duration-300"
            style="background-color: {themes[theme].bg}; color: {themes[theme].text};"
    >
        <article
                class="mx-auto px-4 py-8 md:py-12 transition-all duration-300 {fontFamily === 'sans' ? 'font-sans' : fontFamily === 'serif' ? 'font-serif' : 'font-mono'}"
                style="
                    max-width: {maxWidth}px;
                    font-size: {fontSize}px;
                    line-height: {lineHeight};
                    text-align: {textAlign};
                    --paragraph-spacing: {paragraphSpacing}em;
                "
        >
            <div class="prose max-w-none novel-content" style="color: inherit; text-align: inherit; line-height: inherit;">
                {@html contentHtml}
            </div>

            <div class="h-24 w-full"></div>
        </article>
    </main>
</ReaderLayout>

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