<script lang="ts">
    import { fade } from "svelte/transition";
    import { Button } from "$lib/components/ui/button";
    import * as Drawer from "$lib/components/ui/drawer";
    import * as Sheet from "$lib/components/ui/sheet";
    import { Loader2, AlertCircle, ChevronLeft, Settings2 } from "lucide-svelte";
    import type { Snippet } from "svelte";

    let {
        isLoading = false,
        error = null,
        title = "",
        chapterTitle = "",
        cid = "",
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
        currentProgress?: string | null;
        showSettings: boolean;
        onRetry: () => void;
        children: Snippet;
        settings: Snippet;
    } = $props();

    let innerWidth = $state(0);
    let isMobile = $derived(innerWidth < 1024);
</script>

<svelte:window bind:innerWidth />

<div class="min-h-screen bg-background text-foreground flex flex-col h-screen overflow-hidden">

    <header class="z-40 bg-background/95 backdrop-blur-md border-b border-border/50 p-2 shadow-sm shrink-0 h-[56px] flex items-center">
        <div class="flex items-center justify-between gap-4 w-full px-2 lg:px-6">
            <div class="flex items-center gap-3 overflow-hidden">
                <Button variant="ghost" size="icon" href={cid ? `/content/${cid}` : '/'} class="rounded-full size-9 shrink-0">
                    <ChevronLeft class="size-5" />
                </Button>
                <div class="flex flex-col truncate">
                    <h1 class="font-bold text-sm leading-tight truncate">{title || 'Cargando...'}</h1>
                    <p class="text-xs text-muted-foreground truncate mt-0.5">{chapterTitle || 'Por favor espera'}</p>
                </div>
            </div>

            <div class="flex items-center gap-3 shrink-0">
                {#if currentProgress && !isLoading && !error}
                    <div class="text-xs font-medium text-muted-foreground bg-muted px-2.5 py-1 rounded-md border border-border/50">
                        {currentProgress}
                    </div>
                {/if}
                <Button variant={showSettings ? 'secondary' : 'ghost'} size="icon" disabled={isLoading || !!error} class="rounded-full size-9" onclick={() => showSettings = !showSettings}>
                    <Settings2 class="size-4" />
                </Button>
            </div>
        </div>
    </header>

    <div class="flex flex-1 overflow-hidden relative">

        {#if isLoading}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background">
                <Loader2 class="w-10 h-10 text-primary animate-spin" />
                <span class="text-muted-foreground font-medium tracking-wide">Cargando...</span>
            </div>
        {:else if error}
            <div transition:fade class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-background p-6 text-center">
                <AlertCircle class="w-12 h-12 text-destructive" />
                <p class="text-foreground text-lg font-medium">{error}</p>
                <Button variant="secondary" onclick={onRetry}>Reintentar</Button>
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
                    <Drawer.Header>
                        <Drawer.Title>Settings</Drawer.Title>
                    </Drawer.Header>
                    <div class="p-4 pb-8 max-h-[75vh] overflow-y-auto">
                        {@render settings()}
                    </div>
                </Drawer.Content>
            </Drawer.Root>
        {:else}
            <Sheet.Root bind:open={showSettings}>
                <Sheet.Content side="right" class="w-[340px] sm:w-[400px] overflow-y-auto bg-card/95 backdrop-blur-xl border-l border-border/50 shadow-2xl p-0">
                    <Sheet.Header class="p-6 pb-0">
                        <Sheet.Title class="text-left font-semibold text-lg border-b border-border/40 pb-4 mb-6">Settings</Sheet.Title>
                    </Sheet.Header>
                    <div class="px-6 pb-8">
                        {@render settings()}
                    </div>
                </Sheet.Content>
            </Sheet.Root>
        {/if}
    {/if}

</div>