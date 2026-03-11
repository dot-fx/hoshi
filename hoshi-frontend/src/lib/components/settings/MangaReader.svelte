<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import { Slider } from "$lib/components/ui/slider";
    import { fade } from "svelte/transition";
    import {
        BookOpen, ArrowRightLeft, Maximize,
        GalleryVertical, Grid2X2, List, ChevronLeft, ChevronRight,
        Monitor, Info
    } from "lucide-svelte";
    import type { MangaLayout, ReadingDirection, FitMode, ChapterLayout } from "@/api/config/types";

    let {
        layout = $bindable(),
        direction = $bindable(),
        pagesPerView = $bindable(),
        fitMode = $bindable(),
        gapX = $bindable(),
        gapY = $bindable(),
        preloadPages = $bindable(),
        defaultChapterLayout = $bindable(),
        notifyNewChapters = $bindable(),
        onSave
    }: {
        layout: MangaLayout,
        direction: ReadingDirection,
        pagesPerView: 1 | 2,
        fitMode: FitMode,
        gapX: number,
        gapY: number,
        preloadPages: number,
        defaultChapterLayout: ChapterLayout,
        notifyNewChapters: boolean,
        onSave: () => Promise<void> | void
    } = $props();

    let localGapX = $state([gapX]);
    let localGapY = $state([gapY]);

    function handleCommitX(val: number[]) { gapX = val[0]; onSave(); }
    function handleCommitY(val: number[]) { gapY = val[0]; onSave(); }
</script>

<div class="flex flex-col xl:flex-row gap-8 items-start">

    <!-- COLUMNA IZQUIERDA: EL MINI-READER (PREVIEW DINÁMICA) -->
    <aside class="w-full xl:w-[450px] xl:sticky xl:top-24 space-y-4">
        <div class="flex items-center justify-between px-1">
            <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                <Monitor class="size-3"/> Live Reader Preview
            </Label>
        </div>

        <div class="relative aspect-[3/4] w-full bg-[#050505] rounded-2xl border-4 border-muted shadow-2xl overflow-hidden flex flex-col">
            <!-- Mockup Header -->
            <div class="h-6 w-full bg-background/80 border-b border-white/5 flex items-center px-3 justify-between z-10">
                <div class="h-1 w-10 bg-muted rounded-full"></div>
                <div class="size-2 bg-primary/20 rounded-full"></div>
            </div>

            <!-- Area de Contenido -->
            <div class="flex-1 overflow-hidden relative">
                {#if layout === 'scroll'}
                    <!-- SCROLL MODE: Ahora soporta Double Page -->
                    <div class="h-full w-full overflow-y-auto custom-scrollbar flex flex-col items-center p-4"
                         style="row-gap: {localGapY[0]/3}px">

                        {#each [1, 2, 3] as row}
                            <div class="w-full flex justify-center transition-all duration-300"
                                 style="column-gap: {localGapX[0]/3}px; flex-direction: {direction === 'rtl' ? 'row-reverse' : 'row'}">

                                <div class="flex-1 min-w-0 aspect-[2/3] bg-muted/20 border border-white/5 rounded-md flex items-center justify-center text-[8px] font-bold text-muted-foreground/40 shrink-0">
                                    PAGE
                                </div>

                                {#if pagesPerView === 2}
                                    <div class="flex-1 min-w-0 aspect-[2/3] bg-muted/20 border border-white/5 rounded-md flex items-center justify-center text-[8px] font-bold text-muted-foreground/40 shrink-0" in:fade>
                                        PAGE
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {:else}
                    <!-- PAGED MODE -->
                    <div class="w-full h-full flex items-center justify-center p-6 transition-all duration-500"
                         style="column-gap: {localGapX[0]/3}px; flex-direction: {direction === 'rtl' ? 'row-reverse' : 'row'}">

                        <div class="flex-1 min-w-0 h-full max-h-[85%] aspect-[2/3] bg-muted/20 border border-white/10 rounded-lg flex flex-col items-center justify-center shadow-lg relative overflow-hidden" in:fade>
                            <div class="absolute top-2 {direction === 'rtl' ? 'right-2' : 'left-2'} size-2 bg-primary/40 rounded-full"></div>
                            <span class="text-[10px] font-bold text-primary/60">P. {direction === 'rtl' ? (pagesPerView === 2 ? '2' : '1') : '1'}</span>
                        </div>

                        {#if pagesPerView === 2}
                            <div class="flex-1 min-w-0 h-full max-h-[85%] aspect-[2/3] bg-muted/20 border border-white/10 rounded-lg flex flex-col items-center justify-center shadow-lg relative overflow-hidden" in:fade>
                                <span class="text-[10px] font-bold text-primary/60">P. {direction === 'rtl' ? '1' : '2'}</span>
                            </div>
                        {/if}
                    </div>

                    <!-- Navegación Mockup -->
                    <div class="absolute inset-y-0 left-0 w-8 flex items-center justify-center text-muted-foreground/20"><ChevronLeft class="size-4"/></div>
                    <div class="absolute inset-y-0 right-0 w-8 flex items-center justify-center text-muted-foreground/20"><ChevronRight class="size-4"/></div>
                {/if}
            </div>

            <!-- Mockup Progress Footer -->
            <div class="h-6 w-full bg-background/40 border-t border-white/5 flex items-center px-8">
                <div class="h-0.5 w-full bg-white/10 rounded-full overflow-hidden">
                    <div class="h-full bg-primary/60 w-1/3"></div>
                </div>
            </div>
        </div>

        <div class="bg-muted/30 border border-border/50 rounded-xl p-3 flex gap-3">
            <Info class="size-4 text-muted-foreground shrink-0 mt-0.5"/>
            <p class="text-[11px] text-muted-foreground leading-relaxed italic">
                Preview: <b>{layout.toUpperCase()}</b> + <b>{pagesPerView === 1 ? 'SINGLE' : 'DOUBLE'}</b>.
                Los espaciados están a escala para visualización.
            </p>
        </div>
    </aside>

    <!-- COLUMNA DERECHA: CONFIGURACIÓN -->
    <div class="flex-1 w-full space-y-8">
        <div class="grid gap-8">

            <!-- Modo de Lectura -->
            <div class="space-y-4">
                <div class="space-y-1">
                    <Label class="text-base font-bold">Reading Mode</Label>
                    <p class="text-sm text-muted-foreground">Choose between vertical scroll or horizontal pages.</p>
                </div>
                <Tabs.Root value={layout} onValueChange={(v) => { layout = v; onSave(); }} class="w-full">
                    <Tabs.List class="grid w-full grid-cols-2 rounded-xl h-12 p-1 bg-muted/50">
                        <Tabs.Trigger value="scroll" class="rounded-lg gap-2 font-bold">
                            <GalleryVertical class="size-4"/> Scroll
                        </Tabs.Trigger>
                        <Tabs.Trigger value="paged" class="rounded-lg gap-2 font-bold">
                            <BookOpen class="size-4"/> Paged
                        </Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <!-- Dirección y Páginas -->
            <div class="grid sm:grid-cols-2 gap-6">
                <div class="space-y-3">
                    <Label class="font-bold">Direction</Label>
                    <div class="flex bg-muted/50 p-1 rounded-xl h-11">
                        <Button variant={direction === 'ltr' ? 'secondary' : 'ghost'} class="flex-1 rounded-lg font-bold" onclick={() => { direction = 'ltr'; onSave(); }}>LTR</Button>
                        <Button variant={direction === 'rtl' ? 'secondary' : 'ghost'} class="flex-1 rounded-lg font-bold" onclick={() => { direction = 'rtl'; onSave(); }}>RTL</Button>
                    </div>
                </div>

                <div class="space-y-3">
                    <Label class="font-bold">Pages per View</Label>
                    <Select.Root type="single" value={pagesPerView.toString()} onValueChange={(v) => { pagesPerView = parseInt(v) | 2; onSave(); }}>
                        <Select.Trigger class="h-11 rounded-xl bg-muted/50 border-none font-bold">
                            {pagesPerView === 1 ? 'Single Page' : 'Double Page'}
                        </Select.Trigger>
                        <Select.Content>
                            <Select.Item value="1">Single Page</Select.Item>
                            <Select.Item value="2">Double Page</Select.Item>
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <!-- Ajuste de Imagen -->
            <div class="space-y-3">
                <Label class="font-bold">Image Fit Mode</Label>
                <div class="grid grid-cols-2 gap-2">
                    <Button variant={fitMode === 'width' ? 'secondary' : 'outline'} class="h-12 rounded-xl gap-2 font-bold" onclick={() => { fitMode = 'width'; onSave(); }}>
                        <Maximize class="size-4 rotate-90"/> Fit Width
                    </Button>
                    <Button variant={fitMode === 'height' ? 'secondary' : 'outline'} class="h-12 rounded-xl gap-2 font-bold" onclick={() => { fitMode = 'height'; onSave(); }}>
                        <Maximize class="size-4"/> Fit Height
                    </Button>
                </div>
            </div>

            <!-- Sliders de Espaciado -->
            <div class="bg-muted/30 rounded-2xl p-6 border border-border/50 space-y-8">
                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <Label class="font-bold text-sm">Horizontal Gap (X)</Label>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localGapX[0]}px</span>
                    </div>
                    <Slider bind:value={localGapX} max={100} step={2} onValueCommit={handleCommitX} />
                </div>

                {#if layout === 'scroll'}
                    <div class="space-y-4" in:fade>
                        <div class="flex justify-between items-center">
                            <Label class="font-bold text-sm">Vertical Gap (Y)</Label>
                            <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localGapY[0]}px</span>
                        </div>
                        <Slider bind:value={localGapY} max={100} step={2} onValueCommit={handleCommitY} />
                    </div>
                {/if}
            </div>

            <!-- Otros Ajustes -->
            <div class="grid sm:grid-cols-2 gap-4">
                <div class="flex items-center justify-between p-4 rounded-xl border border-border/40 bg-muted/10">
                    <div class="space-y-0.5">
                        <Label class="font-bold text-sm">Units Layout</Label>
                        <p class="text-[10px] text-muted-foreground uppercase tracking-tight">Grid / List</p>
                    </div>
                    <div class="flex bg-background border rounded-lg p-1">
                        <Button variant={defaultChapterLayout === 'grid' ? 'secondary' : 'ghost'} size="icon" class="size-8" onclick={() => { defaultChapterLayout = 'grid'; onSave(); }}><Grid2X2 class="size-4"/></Button>
                        <Button variant={defaultChapterLayout === 'list' ? 'secondary' : 'ghost'} size="icon" class="size-8" onclick={() => { defaultChapterLayout = 'list'; onSave(); }}><List class="size-4"/></Button>
                    </div>
                </div>

                <div class="flex items-center justify-between p-4 rounded-xl border border-border/40 bg-muted/10">
                    <div class="space-y-0.5">
                        <Label class="font-bold text-sm" for="notify">Notifications</Label>
                        <p class="text-[10px] text-muted-foreground uppercase tracking-tight">New Chapters</p>
                    </div>
                    <Switch id="notify" bind:checked={notifyNewChapters} onCheckedChange={onSave} />
                </div>
            </div>

        </div>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 3px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 10px; }
</style>