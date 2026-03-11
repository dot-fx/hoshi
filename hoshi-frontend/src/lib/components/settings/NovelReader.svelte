<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { Button } from "$lib/components/ui/button";
    import { Label } from "$lib/components/ui/label";
    import { Slider } from "$lib/components/ui/slider";
    import {
        Type,
        AlignLeft,
        AlignJustify,
        Palette,
        Expand,
        Monitor,
        Info,
        BookOpenText,
        ALargeSmall,
        Baseline
    } from "lucide-svelte";
    import type { NovelConfig, NovelTheme, FontFamily, TextAlign } from "@/api/config/types";
    import { i18n } from '$lib/i18n/index.svelte';

    let {
        theme = $bindable(),
        fontFamily = $bindable(),
        fontSize = $bindable(),
        lineHeight = $bindable(),
        maxWidth = $bindable(),
        textAlign = $bindable(),
        onSave
    }: {
        theme: NovelTheme,
        fontFamily: FontFamily,
        fontSize: number,
        lineHeight: number,
        maxWidth: number,
        textAlign: TextAlign,
        onSave: () => Promise<void> | void
    } = $props();

    // Estado local para sliders (vista previa instantánea)
    let localSize = $state([fontSize]);
    let localLine = $state([lineHeight]);
    let localWidth = $state([maxWidth]);

    const themes = {
        light: { bg: "#fdfdfd", text: "#1a1a1a", border: "#e5e7eb" },
        dark: { bg: "#1a1a1a", text: "#e0e0e0", border: "#262626" },
        sepia: { bg: "#f4ecd8", text: "#5b4636", border: "#e2d7bf" },
        oled: { bg: "#000000", text: "#d1d5db", border: "#171717" }
    };

    function handleCommitSize(val: number[]) { fontSize = val[0]; onSave(); }
    function handleCommitLine(val: number[]) { lineHeight = val[0]; onSave(); }
    function handleCommitWidth(val: number[]) { maxWidth = val[0]; onSave(); }
</script>

<div class="flex flex-col xl:flex-row gap-8 items-start">

    <!-- COLUMNA IZQUIERDA: PREVIEW -->
    <aside class="w-full xl:w-[450px] xl:sticky xl:top-24 space-y-4">
        <div class="flex items-center justify-between px-1">
            <Label class="text-xs font-black uppercase tracking-widest text-muted-foreground flex items-center gap-2">
                <Monitor class="size-3"/> {i18n.t('live_preview')}
            </Label>
        </div>

        <div
                class="relative aspect-[3/4] w-full rounded-2xl border-4 shadow-2xl overflow-hidden flex flex-col transition-colors duration-500"
                style="background-color: {themes[theme].bg}; border-color: {themes[theme].border}; color: {themes[theme].text};"
        >
            <div class="h-8 w-full border-b flex items-center px-4 justify-between opacity-40" style="border-color: {themes[theme].border};">
                <div class="h-1.5 w-16 bg-current rounded-full"></div>
                <div class="h-1.5 w-4 bg-current rounded-full"></div>
            </div>

            <div class="flex-1 overflow-hidden p-6 text-sm">
                <article
                        class="mx-auto h-full transition-all duration-300"
                        style="
                        max-width: 100%;
                        font-size: {localSize[0] * 0.6}px;
                        line-height: {localLine[0]};
                        text-align: {textAlign};
                        font-family: {fontFamily === 'sans' ? 'sans-serif' : fontFamily === 'serif' ? 'serif' : 'monospace'};
                    "
                >
                    <h3 class="font-bold mb-4 opacity-90" style="font-size: 1.4em;">{i18n.t('preview_title')}</h3>
                    <p class="mb-4">
                        Había una vez en un lugar muy lejano, un lector que buscaba la configuración perfecta.
                        Ajustó la fuente, cambió los colores y encontró la paz en cada línea.
                    </p>
                </article>
            </div>
            <div class="h-6 w-full flex items-center justify-center text-[10px] opacity-40 font-mono">
                — 42 —
            </div>
        </div>

        <div class="bg-muted/30 border border-border/50 rounded-xl p-3 flex gap-3">
            <Info class="size-4 text-muted-foreground shrink-0 mt-0.5"/>
            <p class="text-[11px] text-muted-foreground leading-relaxed italic">
                El ancho máximo es de <b>{localWidth[0]}px</b>.
            </p>
        </div>
    </aside>

    <!-- COLUMNA DERECHA: AJUSTES -->
    <div class="flex-1 w-full space-y-8">
        <div class="grid gap-8">
            <!-- Selector de Temas -->
            <div class="space-y-4">
                <div class="flex items-center gap-2">
                    <Palette class="size-5 text-primary"/>
                    <Label class="text-base font-bold">{i18n.t('theme')}</Label>
                </div>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                    {#each Object.entries(themes) as [t, colors]}
                        <Button
                                variant={theme === t ? 'default' : 'outline'}
                                class="h-12 rounded-xl flex flex-col gap-0.5 relative overflow-hidden"
                                style="background-color: {theme === t ? '' : colors.bg}; color: {theme === t ? '' : colors.text};"
                                onclick={() => { theme = t; onSave(); }}
                        >
                            <span class="text-xs font-bold capitalize">{t}</span>
                        </Button>
                    {/each}
                </div>
            </div>

            <!-- Tipografía y Alineación -->
            <div class="grid sm:grid-cols-2 gap-8">
                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2"><Type class="size-4 text-primary"/> {i18n.t('font_family')}</Label>
                    <Tabs.Root value={fontFamily} onValueChange={(v) => { fontFamily = v; onSave(); }}>
                        <Tabs.List class="grid w-full grid-cols-3 rounded-xl h-11 p-1 bg-muted/50">
                            <Tabs.Trigger value="sans" class="rounded-lg font-sans">Sans</Tabs.Trigger>
                            <Tabs.Trigger value="serif" class="rounded-lg font-serif">Serif</Tabs.Trigger>
                            <Tabs.Trigger value="mono" class="rounded-lg font-mono">Mono</Tabs.Trigger>
                        </Tabs.List>
                    </Tabs.Root>
                </div>

                <div class="space-y-4">
                    <Label class="font-bold flex items-center gap-2">{i18n.t('alignment')}</Label>
                    <div class="flex bg-muted/50 p-1 rounded-xl h-11">
                        <Button
                                variant={textAlign === 'left' ? 'secondary' : 'ghost'}
                                class="flex-1 rounded-lg"
                                onclick={() => { textAlign = 'left'; onSave(); }}
                        >
                            <AlignLeft class="size-4 mr-2"/> {i18n.t('left')}
                        </Button>
                        <Button
                                variant={textAlign === 'justify' ? 'secondary' : 'ghost'}
                                class="flex-1 rounded-lg"
                                onclick={() => { textAlign = 'justify'; onSave(); }}
                        >
                            <AlignJustify class="size-4 mr-2"/> {i18n.t('justify')}
                        </Button>
                    </div>
                </div>
            </div>

            <!-- Sliders Dinámicos -->
            <div class="bg-muted/30 rounded-2xl p-6 border border-border/50 space-y-10">
                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <ALargeSmall class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('font_size')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localSize[0]}px</span>
                    </div>
                    <Slider bind:value={localSize} min={12} max={32} step={1} onValueCommit={handleCommitSize} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Baseline class="size-4 text-primary"/> <!-- <--- Icono corregido -->
                            <Label class="font-bold text-sm">{i18n.t('line_height')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localLine[0]}</span>
                    </div>
                    <Slider bind:value={localLine} min={1} max={3} step={0.1} onValueCommit={handleCommitLine} />
                </div>

                <div class="space-y-4">
                    <div class="flex justify-between items-center">
                        <div class="flex items-center gap-2">
                            <Expand class="size-4 text-primary"/>
                            <Label class="font-bold text-sm">{i18n.t('content_width')}</Label>
                        </div>
                        <span class="text-xs font-mono font-bold text-primary bg-primary/10 px-2 py-1 rounded-md">{localWidth[0]}px</span>
                    </div>
                    <Slider bind:value={localWidth} min={400} max={1200} step={50} onValueCommit={handleCommitWidth} />
                </div>
            </div>
        </div>
    </div>
</div>