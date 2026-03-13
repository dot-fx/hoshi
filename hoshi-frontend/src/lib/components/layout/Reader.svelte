<script lang="ts">
    import { fade } from "svelte/transition";
    import { Button } from "@/components/ui/button";
    import * as Drawer from "@/components/ui/drawer";
    import * as Sheet from "@/components/ui/sheet";
    import * as Popover from "@/components/ui/popover";
    import {
        Loader2, AlertCircle, ChevronLeft, Settings2,
        ArrowLeft, ArrowRight, Check, ChevronsUpDown
    } from "lucide-svelte";
    import type { Snippet } from "svelte";
    import { i18n } from '@/i18n/index.svelte.js';

    let {
        isLoading = false,
        error = null,
        title = "",
        chapterTitle = "",
        cid = "",
        extension = "",
        contentType = "manga",
        currentChapter = 0,
        allChapters = [],
        currentProgress = null,
        showSettings = $bindable(false),
        onRetry,
        children,
        settings
    }: {
        isLoading: boolean;
        error: string | null;
        title: string;
        chapterTitle: string;
        cid: string;
        extension: string;
        contentType: "manga" | "novel";
        currentChapter: number;
        allChapters: any[];
        currentProgress?: string | null;
        showSettings: boolean;
        onRetry: () => void;
        children: Snippet;
        settings: Snippet;
    } = $props();

    let innerWidth = $state(0);
    let isMobile = $derived(innerWidth < 1024);
    let chapterDropdownOpen = $state(false);

    let sortedChapters = $derived([...allChapters].sort((a, b) => Number(a.number ?? a.unitNumber) - Number(b.number ?? b.unitNumber)));
    let currentIndex = $derived(sortedChapters.findIndex(c => Number(c.number ?? c.unitNumber) === currentChapter));

    let prevChapter = $derived(currentIndex > 0 ? sortedChapters[currentIndex - 1] : null);
    let nextChapter = $derived(currentIndex >= 0 && currentIndex < sortedChapters.length - 1 ? sortedChapters[currentIndex + 1] : null);

    let baseRoute = $derived(contentType === "novel" ? "/read-novel" : "/read");

    function getChapterUrl(chap: any) {
        if (!chap) return "#";
        const num = chap.number ?? chap.unitNumber;
        return `${baseRoute}/${cid}/${extension}/${num}`;
    }
</script>

<svelte:window bind:innerWidth />

<div class="min-h-screen bg-background text-foreground flex flex-col h-screen overflow-hidden">
    <header class="z-40 bg-background/95 backdrop-blur-md border-b border-border/50 p-2 shadow-sm shrink-0 h-[60px] flex items-center justify-between gap-2">
        <div class="flex items-center gap-1.5 sm:gap-3 overflow-hidden flex-1">
            <Button variant="ghost" size="icon" href={cid ? `/content/${cid}` : '/'} class="rounded-full size-9 shrink-0">
                <ChevronLeft class="size-5" />
            </Button>

            <Popover.Root bind:open={chapterDropdownOpen}>
                <Popover.Trigger
                        class="flex flex-col items-start justify-center h-auto py-1 px-2.5 rounded-lg hover:bg-muted/50 transition-colors w-full max-w-[200px] sm:max-w-[300px] outline-none"
                        disabled={isLoading || !!error}
                >
                    <span class="font-bold text-[13px] sm:text-sm leading-tight truncate w-full text-left">{title || i18n.t('reader.loading')}</span>
                    <div class="flex items-center gap-1 mt-0.5 w-full">
                        <span class="text-[11px] sm:text-xs text-muted-foreground truncate text-left">{chapterTitle || i18n.t('reader.loading')}</span>
                        <ChevronsUpDown class="size-3 text-muted-foreground shrink-0 opacity-50" />
                    </div>
                </Popover.Trigger>

                <Popover.Content class="w-[280px] sm:w-[320px] p-0 flex flex-col max-h-[60vh] shadow-xl border-border/50" align="start">
                    <div class="px-4 py-3 border-b border-border/40 bg-muted/30">
                        <h4 class="font-bold text-sm">{i18n.t('reader.select_chapter')}</h4>
                        <p class="text-xs text-muted-foreground">{i18n.t('reader.chapters_available', { count: sortedChapters.length })}</p>
                    </div>
                    <div class="flex-1 overflow-y-auto custom-scrollbar p-1 flex flex-col gap-0.5">
                        {#each sortedChapters as chap}
                            {@const num = chap.number ?? chap.unitNumber}
                            {@const isCurrent = num === currentChapter}
                            <a
                                    href={getChapterUrl(chap)}
                                    onclick={() => chapterDropdownOpen = false}
                                    class="flex flex-col items-start px-3 py-2 text-sm rounded-md transition-colors {isCurrent ? 'bg-primary/10 text-primary font-bold' : 'hover:bg-muted'}"
                            >
                                <div class="flex items-center justify-between w-full">
                                    <span>{i18n.t('reader.chapter_number', { num })}</span>
                                    {#if isCurrent}<Check class="size-4 shrink-0" />{/if}
                                </div>
                                {#if chap.title}
                                    <span class="text-[10px] sm:text-xs opacity-70 truncate w-full font-normal">{chap.title}</span>
                                {/if}
                            </a>
                        {/each}
                    </div>
                </Popover.Content>
            </Popover.Root>
        </div>

        <div class="flex items-center gap-1 sm:gap-2 shrink-0">
            {#if currentProgress && !isLoading && !error}
                <div class="text-[10px] sm:text-xs font-mono font-bold text-muted-foreground bg-muted px-2 py-1 rounded-md border border-border/50 hidden md:block">
                    {currentProgress}
                </div>
            {/if}

            <div class="flex bg-muted/50 rounded-lg p-0.5 border border-border/50">
                <Button variant="ghost" size="icon" class="size-8 rounded-md" href={prevChapter ? getChapterUrl(prevChapter) : undefined} disabled={!prevChapter || isLoading}>
                    <ArrowLeft class="size-4" />
                </Button>
                <Button variant="ghost" size="icon" class="size-8 rounded-md" href={nextChapter ? getChapterUrl(nextChapter) : undefined} disabled={!nextChapter || isLoading}>
                    <ArrowRight class="size-4" />
                </Button>
            </div>

            <Button variant={showSettings ? 'secondary' : 'ghost'} size="icon" disabled={isLoading || !!error} class="rounded-full size-9" onclick={() => showSettings = !showSettings}>
                <Settings2 class="size-4" />
            </Button>
        </div>
    </header>

    <div class="flex flex-1 overflow-hidden relative">
        {#if isLoading}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background">
                <Loader2 class="w-10 h-10 text-primary animate-spin" />
                <span class="text-muted-foreground font-medium tracking-wide">{i18n.t('reader.loading')}</span>
            </div>
        {:else if error}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                <AlertCircle class="w-12 h-12 text-destructive" />
                <p class="text-foreground text-lg font-medium">{error}</p>
                <Button variant="secondary" onclick={onRetry}>{i18n.t('retry')}</Button>
            </div>
        {:else}
            <div class="flex-1 relative flex flex-col overflow-hidden">
                {@render children()}
            </div>
        {/if}
    </div>

    {#if !isLoading && !error}
        {#if isMobile}
            <Drawer.Root bind:open={showSettings}>
                <Drawer.Content class="bg-background/95 backdrop-blur-xl border-border/50">
                    <Drawer.Header><Drawer.Title>{i18n.t('reader.settings')}</Drawer.Title></Drawer.Header>
                    <div class="p-4 pb-8 max-h-[75vh] overflow-y-auto">{@render settings()}</div>
                </Drawer.Content>
            </Drawer.Root>
        {:else}
            <Sheet.Root bind:open={showSettings}>
                <Sheet.Content side="right" class="w-[340px] sm:w-[400px] overflow-y-auto bg-card/95 backdrop-blur-xl border-l border-border/50 shadow-2xl p-0">
                    <Sheet.Header class="p-6 pb-0"><Sheet.Title class="text-left font-semibold text-lg border-b border-border/40 pb-4 mb-6">{i18n.t('reader.settings')}</Sheet.Title></Sheet.Header>
                    <div class="px-6 pb-8">{@render settings()}</div>
                </Sheet.Content>
            </Sheet.Root>
        {/if}
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150,150,150,0.3); border-radius: 10px; }
</style>