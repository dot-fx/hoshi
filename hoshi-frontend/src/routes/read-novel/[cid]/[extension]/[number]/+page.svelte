<script lang="ts">
    import { onMount, untrack } from "svelte";
    import { page } from "$app/state";
    import { contentApi } from "@/api/content/content";
    import { primaryMetadata } from "@/api/content/types";
    import { i18n } from '@/i18n/index.svelte.js';
    import { appConfig } from "@/stores/config.svelte.js";
    import type { NovelConfig, NovelTheme, FontFamily } from "@/api/config/types";
    import { progressApi } from "@/api/progress/progress";
    import { listApi } from "@/api/list/list";
    import type { CoreError } from "@/api/client";

    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Label } from "@/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Type, AlignLeft, AlignJustify, Palette, Expand, Baseline, Space } from "lucide-svelte";

    import Reader from "@/components/layout/Reader.svelte";
    import {contentCache} from "@/stores/contentCache.svelte.js";

    const params = $derived(page.params as Record<string, string>);
    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));

    let title = $state("");
    let chapterTitle = $state("");
    let contentHtml = $state<string>("");
    let allChapters = $state<any[]>([]);
    let isLoading = $state(true);
    let error = $state<CoreError | null>(null);
    let showSettings = $state(false);
    let isNsfw = $state(false)

    const novelConfig = $derived(appConfig.data?.novel);

    let theme = $derived(novelConfig?.theme ?? "dark");
    let fontFamily = $derived(novelConfig?.fontFamily ?? "sans");
    let textAlign = $derived(novelConfig?.textAlign ?? "left");

    let fontSize = $state(novelConfig?.fontSize ?? 18);
    let lineHeight = $state(novelConfig?.lineHeight ?? 1.6);
    let maxWidth = $state(novelConfig?.maxWidth ?? 800);
    let paragraphSpacing = $state(novelConfig?.paragraphSpacing ?? 1.5);
    let coverImage = $state<string | null>(null);

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

    async function updateNovelConfig(patch: Partial<NovelConfig>) {
        if (!appConfig.data?.novel) return;
        try {
            await appConfig.update({ novel: { ...appConfig.data.novel, ...patch } });
        } catch (err) {
            console.error("Error updating novel config:", err);
        }
    }

    let debounceTimer: any;
    $effect(() => {
        const currentSize = fontSize;
        const currentLine = lineHeight;
        const currentWidth = maxWidth;
        const currentSpacing = paragraphSpacing;

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
            fontSize = appConfig.data.novel.fontSize;
            lineHeight = appConfig.data.novel.lineHeight;
            maxWidth = appConfig.data.novel.maxWidth;
            paragraphSpacing = appConfig.data.novel.paragraphSpacing ?? 1.5;
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
            let contentPromise;

            if (contentCache.has(currentCid)) {
                contentPromise = Promise.resolve(contentCache.get(currentCid));
            } else {
                contentPromise = contentApi.get_by_cid(currentCid).then(res => {
                    contentCache.set(currentCid, res);
                    return res;
                });
            }

            const [contentRes, itemsRes, playRes] = await Promise.all([
                contentPromise,
                contentApi.getItems(currentCid, currentExt),
                contentApi.play(currentCid, currentExt, currentChapterNum)
            ]);

            console.log(playRes)

            isNsfw = contentRes.content.nsfw;

            const meta = primaryMetadata(contentRes);
            title = meta?.title ?? "";
            coverImage = meta?.coverImage ?? null;

            const rawItems: any[] = Array.isArray(itemsRes) ? itemsRes : (itemsRes as any)?.data ?? [];
            allChapters = rawItems.sort((a, b) => Number(a.number ?? a.unitNumber) - Number(b.number ?? b.unitNumber));

            const currentUnit = allChapters.find(u => Number(u.number ?? u.unitNumber) === currentChapterNum);
            chapterTitle = currentUnit?.title || "";

            if (playRes.type.toLowerCase() !== "novel" || !playRes.data) {
                throw { key: 'reader.no_data' } as CoreError;
            }

            const data: any = playRes.data;
            contentHtml = data.html || data.text || data.content || data;

            if (!contentHtml) {
                throw { key: 'reader.no_content' } as CoreError;
            }

            progressApi.updateChapterProgress({ cid: currentCid, chapter: currentChapterNum, completed: false })
                .catch(e => console.error("History sync failed", e));

        } catch (e: any) {
            console.log(e)
            error = e.key ? e : { key: 'errors.unknown_error' };
        } finally {
            isLoading = false;
        }
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
        {coverImage}
        {isNsfw} contentType="novel"
        currentChapter={chapterNumber}
        {allChapters}
        bind:showSettings
        onRetry={() => loadChapter(cid, extension, chapterNumber)}
>
    {#snippet settings()}
        <div class="space-y-6 px-1">
            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <Palette class="size-4"/> {i18n.t('reader.theme')}
                </Label>
                <div class="grid grid-cols-2 gap-2">
                    {#each Object.entries(themes) as [t, colors]}
                        <Button
                                variant={theme === t ? 'default' : 'outline'}
                                class="text-sm h-10 font-bold border-border/50 relative overflow-hidden"
                                style="background-color: {theme === t ? '' : colors.bg}; color: {theme === t ? '' : colors.text};"
                                onclick={() => updateNovelConfig({ theme: t as NovelTheme })}
                        >
                            <span class="capitalize">{t}</span>
                        </Button>
                    {/each}
                </div>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    <Type class="size-4"/> {i18n.t('reader.font_family')}
                </Label>
                <Tabs.Root value={fontFamily} onValueChange={(v) => updateNovelConfig({ fontFamily: v as FontFamily })} class="w-full">
                    <Tabs.List class="grid w-full grid-cols-3 rounded-xl h-11 p-1 bg-muted/50">
                        <Tabs.Trigger value="sans" class="rounded-lg font-sans font-bold h-9">Sans</Tabs.Trigger>
                        <Tabs.Trigger value="serif" class="rounded-lg font-serif font-bold h-9">Serif</Tabs.Trigger>
                        <Tabs.Trigger value="mono" class="rounded-lg font-mono font-bold h-9">Mono</Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="space-y-3">
                <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                    {i18n.t('reader.alignment')}
                </Label>
                <div class="grid grid-cols-2 gap-2 bg-muted/50 p-1 rounded-xl">
                    <Button variant={textAlign === 'left' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateNovelConfig({ textAlign: 'left' })}><AlignLeft class="size-4 mr-2"/> {i18n.t('reader.align_left')}</Button>
                    <Button variant={textAlign === 'justify' ? 'secondary' : 'ghost'} class="text-sm h-9 font-bold" onclick={() => updateNovelConfig({ textAlign: 'justify' })}><AlignJustify class="size-4 mr-2"/> {i18n.t('reader.align_justify')}</Button>
                </div>
            </div>

            <div class="space-y-5 pt-4 border-t border-border/40">
                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Type class="size-3.5"/> {i18n.t('reader.font_size')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{fontSize}px</span>
                    </div>
                    <Slider type="single" bind:value={fontSize} min={12} max={32} step={1} class="w-full" />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Baseline class="size-3.5"/> {i18n.t('reader.line_height')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{lineHeight}</span>
                    </div>
                    <Slider type="single" bind:value={lineHeight} min={1} max={3} step={0.1} class="w-full" />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Space class="size-3.5"/> {i18n.t('reader.paragraph_spacing')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{paragraphSpacing}em</span>
                    </div>
                    <Slider type="single" bind:value={paragraphSpacing} min={0.5} max={4} step={0.1} class="w-full" />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-3">
                        <Label class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1.5"><Expand class="size-3.5"/> {i18n.t('reader.content_width')}</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-0.5 rounded-md">{maxWidth}px</span>
                    </div>
                    <Slider type="single" bind:value={maxWidth} min={400} max={1200} step={50} class="w-full" />
                </div>
            </div>
        </div>
    {/snippet}

    <main
            id="novel-main-container"
            class="flex-1 overflow-y-auto overflow-x-hidden relative transition-colors duration-300"
            style="background-color: {themes[theme].bg}; color: {themes[theme].text};"
            onscroll={(e) => {
                const target = e.currentTarget;
                if (target.scrollHeight > 0) {
                    const ratio = (target.scrollTop + target.clientHeight) / target.scrollHeight;
                    handleProgress(ratio);
                }
            }}
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