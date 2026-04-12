<script lang="ts">
    import { contentApi } from "$lib/api/content/content";
    import { extensions } from "@/stores/extensions.svelte.js";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Pagination from "$lib/components/ui/pagination";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import { BookOpen, SearchX, AlertCircle, Clock, ChevronRight } from "lucide-svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import type { CoreError } from "@/api/client";

    let {
        cid,
        contentType
    }: {
        cid: string,
        contentType: string
    } = $props();

    let availableExtensions = $derived(
        contentType === "manga" ? extensions.manga.map(e => e.id) :
            contentType === "novel" ? extensions.novel.map(e => e.id) : []
    );

    let selectedExtensionName = $state("");
    let chapters = $state<any[]>([]);
    let loading = $state(false);
    let error = $state<CoreError | null>(null);

    let currentPage = $state(1);
    const perPage = 10;
    const basePath = $derived(contentType === "novel" ? "/read-novel" : "/read");

    let paginatedChapters = $derived(
        chapters.slice((currentPage - 1) * perPage, currentPage * perPage)
    );

    const formatDate = (dateString: string | null) => {
        if (!dateString) return i18n.t('content.unknown_date');
        return new Intl.DateTimeFormat(i18n.locale, {
            year: 'numeric', month: 'short', day: 'numeric'
        }).format(new Date(dateString));
    };

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

    async function loadChapters(extName: string) {
        loading = true;
        error = null;
        currentPage = 1;
        try {
            const res = await contentApi.getItems(cid, extName);
            chapters = Array.isArray(res) ? res : [];

            // Si la extensión responde con éxito pero no hay capítulos,
            // no lo tratamos como error, simplemente mostraremos el Empty State.
        } catch (e: any) {
            console.error("Failed to load chapters:", e);
            chapters = [];
            error = e.key ? e : { key: 'content.failed_load' };
        } finally {
            loading = false;
        }
    }
</script>

<div class="space-y-6 animate-in fade-in duration-500">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-2 border-b border-border/20">
        <h3 class="text-2xl font-black tracking-tight flex items-center gap-2">
            <BookOpen class="w-6 h-6 text-primary" />
            {i18n.t('content.chapters_title')}
        </h3>

        {#if availableExtensions.length > 0}
            <div class="w-full sm:w-[280px]">
                <Select.Root type="single" bind:value={selectedExtensionName}>
                    <Select.Trigger class="w-full bg-muted/50 border-border/50 hover:bg-muted transition-colors rounded-xl font-medium capitalize">
                        {selectedExtensionName || i18n.t('content.select_extension')}
                    </Select.Trigger>
                    <Select.Content class="rounded-xl border-border/50 bg-card/95 backdrop-blur-xl">
                        {#each availableExtensions as extName}
                            <Select.Item value={extName} class="capitalize font-medium rounded-lg cursor-pointer">{extName}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>
        {/if}
    </div>

    {#if availableExtensions.length === 0}
        <Empty.Root class="border border-border/20 bg-muted/5 rounded-2xl py-16">
            <Empty.Header>
                <div class="p-4 bg-primary/10 rounded-full mb-4 inline-flex">
                    <BookOpen class="w-10 h-10 text-primary" />
                </div>
                <Empty.Title class="text-xl font-bold">{i18n.t('content.no_sources')}</Empty.Title>
                <Empty.Description class="max-w-md mx-auto text-base">
                    {i18n.t('install_extension')}
                </Empty.Description>
            </Empty.Header>
        </Empty.Root>

    {:else if loading}
        <div class="space-y-3">
            {#each Array(5) as _}
                <div class="flex items-center justify-between p-4 rounded-xl border border-border/20 bg-muted/10">
                    <div class="space-y-2 flex-1">
                        <Skeleton class="h-5 w-1/3 rounded-lg" />
                        <Skeleton class="h-4 w-1/4 rounded-lg opacity-50" />
                    </div>
                    <Skeleton class="h-9 w-24 rounded-full ml-4" />
                </div>
            {/each}
        </div>

    {:else if error}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center bg-destructive/5 rounded-2xl border border-destructive/10">
            <div class="p-4 bg-destructive/10 rounded-full mb-4">
                <AlertCircle class="w-10 h-10 text-destructive" />
            </div>
            <h3 class="text-xl font-bold text-foreground mb-2">{i18n.t(error.key)}</h3>
            <Button variant="outline" class="mt-4 rounded-xl font-bold" onclick={() => loadChapters(selectedExtensionName)}>
                {i18n.t('content.retry')}
            </Button>
        </div>

    {:else if chapters.length === 0}
        <Empty.Root class="border border-border/20 bg-muted/5 rounded-2xl py-16">
            <Empty.Header>
                <div class="p-4 bg-muted rounded-full mb-4 inline-flex">
                    <SearchX class="w-10 h-10 text-muted-foreground" />
                </div>
                <Empty.Title class="text-xl font-bold">{i18n.t('content.no_chapters')}</Empty.Title>
                <Empty.Description class="text-base">
                    {i18n.t('content.no_chapters_desc')}
                </Empty.Description>
            </Empty.Header>
        </Empty.Root>

    {:else}
        <div class="grid gap-3">
            {#each paginatedChapters as chapter (chapter.id || chapter.number)}
                {@const num = chapter.number ?? chapter.unitNumber}
                {@const url = `${basePath}/${cid}/${selectedExtensionName}/${num}`}

                <a href={url} class="group flex flex-col sm:flex-row sm:items-center justify-between p-4 rounded-xl border border-border/30 bg-card hover:bg-muted/30 hover:border-primary/30 transition-all duration-200 gap-4 shadow-sm hover:shadow-md">
                    <div class="flex items-center gap-4 min-w-0">
                        <div class="flex items-center justify-center w-12 h-12 rounded-lg bg-primary/10 text-primary font-black text-lg shrink-0">
                            {num}
                        </div>
                        <div class="flex flex-col min-w-0 gap-1">
                            <span class="font-bold text-foreground line-clamp-1 group-hover:text-primary transition-colors">
                                {chapter.title?.trim() ? chapter.title : `${i18n.t('content.chapter')} ${num}`}
                            </span>
                            <div class="flex items-center gap-2 text-xs font-medium text-muted-foreground">
                                <span class="flex items-center gap-1">
                                    <Clock class="w-3.5 h-3.5" />
                                    {formatDate(chapter.releaseDate)}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div class="flex items-center gap-3 shrink-0 sm:ml-auto">
                        <Button size="sm" variant="secondary" class="w-full sm:w-auto rounded-full font-bold bg-primary/10 text-primary hover:bg-primary hover:text-primary-foreground transition-all">
                            {i18n.t('content.read')}
                            <ChevronRight class="w-4 h-4 ml-1" />
                        </Button>
                    </div>
                </a>
            {/each}
        </div>

        {#if chapters.length > perPage}
            <div class="mt-8 flex justify-center pb-4">
                <Pagination.Root count={chapters.length} {perPage} bind:page={currentPage}>
                    {#snippet children({ pages, currentPage })}
                        <Pagination.Content class="bg-card border border-border/40 p-1 rounded-2xl shadow-sm">
                            <Pagination.Item>
                                <Pagination.PrevButton class="rounded-xl hover:bg-muted" />
                            </Pagination.Item>
                            {#each pages as page (page.key)}
                                {#if page.type === "ellipsis"}
                                    <Pagination.Item>
                                        <Pagination.Ellipsis />
                                    </Pagination.Item>
                                {:else}
                                    <Pagination.Item>
                                        <Pagination.Link {page} isActive={currentPage === page.value} class="rounded-xl {currentPage === page.value ? 'bg-primary text-primary-foreground font-bold' : 'hover:bg-muted'}">
                                            {page.value}
                                        </Pagination.Link>
                                    </Pagination.Item>
                                {/if}
                            {/each}
                            <Pagination.Item>
                                <Pagination.NextButton class="rounded-xl hover:bg-muted" />
                            </Pagination.Item>
                        </Pagination.Content>
                    {/snippet}
                </Pagination.Root>
            </div>
        {/if}
    {/if}
</div>