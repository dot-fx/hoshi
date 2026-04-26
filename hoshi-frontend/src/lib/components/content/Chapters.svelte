<script lang="ts">
    import { contentApi } from "$lib/api/content/content";
    import { extensions } from "@/stores/extensions.svelte.js";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { BookOpen, SearchX, AlertCircle, BookOpenCheck } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { CoreError } from "@/api/client";
    import type { ChapterProgress } from "@/api/progress/types";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        cid,
        contentType,
        progress = [],
    }: {
        cid: string,
        contentType: string,
        progress?: ChapterProgress[],
    } = $props();

    // Build a map keyed by chapter number for O(1) lookup
    const progressMap = $derived(
        new Map(progress.map(p => [p.chapter, p]))
    );

    let availableExtensions = $derived(
        contentType === "manga" ? extensions.manga.map(e => e.id) :
            contentType === "novel" ? extensions.novel.map(e => e.id) : []
    );

    let selectedExtensionName = $state("");
    let chapters = $state<any[]>([]);
    let loading = $state(false);
    let error = $state<CoreError | null>(null);

    let currentPage = $state(1);
    const perPage = 14;
    const basePath = $derived(contentType === "novel" ? "/read-novel" : "/read");

    const enrichedChapters = $derived(
        chapters.map(ch => {
            const num = ch.number ?? ch.unitNumber;
            const prog = progressMap.get(num);
            return { ...ch, _isRead: prog?.completed ?? false };
        })
    );

    let paginatedChapters = $derived(
        enrichedChapters.slice((currentPage - 1) * perPage, currentPage * perPage)
    );

    $effect(() => {
        if (!selectedExtensionName && availableExtensions.length > 0) {
            selectedExtensionName = availableExtensions[0];
        }
    });

    $effect(() => {
        if (selectedExtensionName) {
            loadChapters(selectedExtensionName);
        }
    });

    let extensionItems = $derived(
        availableExtensions.map(ext => ({
            value: ext,
            label: ext
        }))
    );

    async function loadChapters(extName: string) {
        loading = true;
        error = null;
        currentPage = 1;
        try {
            const res = await contentApi.getItems(cid, extName);
            chapters = Array.isArray(res) ? res : [];
        } catch (e: any) {
            console.error("Failed to load chapters:", e);
            chapters = [];
            error = e.key ? e : { key: 'content.failed_load' };
        } finally {
            loading = false;
        }
    }
</script>

<div class="space-y-4">

    <!-- Header -->
    <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2">
            <h2 class="text-base font-bold tracking-tight">
                {i18n.t('content.chapters_title')}
            </h2>
            {#if chapters.length > 0}
                <span class="text-[11px] font-semibold text-muted-foreground/60 bg-muted/40 px-2 py-0.5 rounded-full">
                    {chapters.length}
                </span>
            {/if}
        </div>

        {#if availableExtensions.length > 1}
            <div class="w-[180px] shrink-0">
                <ResponsiveSelect
                        bind:value={selectedExtensionName}
                        items={extensionItems}
                        placeholder={i18n.t('content.select_extension')}
                        class="bg-muted/30 border-border/20 hover:bg-muted/50 transition-colors rounded-xl font-medium capitalize text-xs h-8"
                />
            </div>
        {/if}
    </div>

    <!-- No sources -->
    {#if availableExtensions.length === 0}
        <div class="flex flex-col items-center justify-center gap-3 py-14 rounded-2xl border border-border/20 bg-muted/5">
            <BookOpen class="w-8 h-8 text-muted-foreground/20" />
            <div class="text-center space-y-0.5">
                <p class="text-sm font-semibold text-muted-foreground/60">{i18n.t('content.no_sources')}</p>
                <p class="text-[11px] text-muted-foreground/40">{i18n.t('install_extension')}</p>
            </div>
        </div>

        <!-- Loading skeletons -->
    {:else if loading}
        <div class="flex flex-col gap-1.5">
            {#each Array(8) as _}
                <div class="flex items-center gap-3 px-3 py-2.5 rounded-xl border border-border/20 bg-muted/10">
                    <Skeleton class="shrink-0 w-9 h-9 rounded-lg" />
                    <div class="flex-1 space-y-1.5">
                        <Skeleton class="h-3.5 w-3/5 rounded-md" />
                        <Skeleton class="h-2.5 w-1/4 rounded-md opacity-40" />
                    </div>
                </div>
            {/each}
        </div>

        <!-- Error -->
    {:else if error}
        <div class="flex flex-col items-center justify-center gap-3 py-14 rounded-2xl border border-destructive/10 bg-destructive/5">
            <AlertCircle class="w-8 h-8 text-destructive/40" />
            <div class="text-center space-y-0.5">
                <p class="text-sm font-semibold text-muted-foreground/60">{i18n.t(error.key)}</p>
            </div>
            <button
                    onclick={() => loadChapters(selectedExtensionName)}
                    class="text-[11px] font-bold text-muted-foreground/40 hover:text-muted-foreground/70 uppercase tracking-widest transition-colors"
            >
                {i18n.t('content.retry')}
            </button>
        </div>

        <!-- Empty -->
    {:else if chapters.length === 0}
        <div class="flex flex-col items-center justify-center gap-3 py-14 rounded-2xl border border-border/20 bg-muted/5">
            <SearchX class="w-8 h-8 text-muted-foreground/20" />
            <div class="text-center space-y-0.5">
                <p class="text-sm font-semibold text-muted-foreground/60">{i18n.t('content.no_chapters')}</p>
                <p class="text-[11px] text-muted-foreground/40">{i18n.t('content.no_chapters_desc')}</p>
            </div>
        </div>

    {:else}
        {@const totalPages = Math.ceil(chapters.length / perPage)}

        <div class="flex flex-col gap-1.5 xl:max-h-[calc(100vh-12rem)] xl:overflow-y-auto xl:pr-1 hide-scrollbar">
            {#each paginatedChapters as chapter (chapter.id || chapter.number)}
                {@const num = chapter.number ?? chapter.unitNumber}
                {@const url = `${basePath}/${cid}/${selectedExtensionName}/${num}`}
                {@const title = chapter.title?.trim() ? chapter.title : null}
                {@const isRead = chapter._isRead}

                <a
                        href={url}
                        class="group flex items-center gap-3 px-3 py-2.5 rounded-xl border transition-all duration-200
                        {isRead
                            ? 'border-border/10 bg-muted/5 opacity-45 hover:opacity-70 hover:bg-muted/15 hover:border-border/30'
                            : 'border-border/20 bg-muted/10 hover:bg-muted/25 hover:border-border/50'}"
                >
                    <!-- Chapter number badge -->
                    <div class="shrink-0 w-9 h-9 rounded-lg bg-muted/40 flex items-center justify-center border border-border/20 group-hover:border-border/40 transition-colors">
                        <span class="text-xs font-black text-muted-foreground/50 group-hover:text-muted-foreground/80 transition-colors">{num}</span>
                    </div>

                    <!-- Title -->
                    <div class="flex-1 min-w-0">
                        {#if title}
                            <p class="font-semibold text-sm leading-snug line-clamp-1 group-hover:text-primary transition-colors duration-150 {isRead ? 'text-muted-foreground/60' : ''}">
                                {title}
                            </p>
                        {:else}
                            <p class="font-semibold text-sm text-muted-foreground/60 group-hover:text-muted-foreground transition-colors duration-150">
                                {i18n.t('content.chapter')} {num}
                            </p>
                        {/if}
                        {#if chapter.scanlator}
                            <p class="text-[10px] text-muted-foreground/35 truncate mt-0.5">{chapter.scanlator}</p>
                        {/if}
                    </div>

                    <!-- Read indicator -->
                    {#if isRead}
                        <BookOpenCheck class="w-3.5 h-3.5 text-primary/50 shrink-0" />
                    {/if}
                </a>
            {/each}
        </div>

        <!-- Pagination -->
        {#if totalPages > 1}
            <div class="flex items-center justify-between gap-2 pt-1">
                <button
                        onclick={() => currentPage = Math.max(1, currentPage - 1)}
                        disabled={currentPage === 1}
                        class="flex-1 h-9 rounded-xl border border-border/20 bg-muted/10 hover:bg-muted/25 hover:border-border/40 disabled:opacity-30 disabled:pointer-events-none transition-all text-xs font-semibold text-muted-foreground/70"
                >
                    ←
                </button>

                <span class="text-[11px] font-semibold text-muted-foreground/40 tabular-nums shrink-0 px-1">
                    {currentPage} / {totalPages}
                </span>

                <button
                        onclick={() => currentPage = Math.min(totalPages, currentPage + 1)}
                        disabled={currentPage === totalPages}
                        class="flex-1 h-9 rounded-xl border border-border/20 bg-muted/10 hover:bg-muted/25 hover:border-border/40 disabled:opacity-30 disabled:pointer-events-none transition-all text-xs font-semibold text-muted-foreground/70"
                >
                    →
                </button>
            </div>
        {/if}
    {/if}
</div>

<style>
    .hide-scrollbar::-webkit-scrollbar { display: none; }
    .hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>