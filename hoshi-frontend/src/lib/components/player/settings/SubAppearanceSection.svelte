<script lang="ts">
    import {
        ChevronLeft, ChevronDown, ChevronRight, Check, RotateCcw,
        SlidersHorizontal, Type, Palette, Blend, Square, MoveVertical, Maximize2, Rows3, Eye, Layers, Pipette,
    } from 'lucide-svelte';
    import type { Component } from 'svelte';
    import { Button } from "@/components/ui/button";
    import { Slider } from "@/components/ui/slider";
    import { Separator } from "@/components/ui/separator";
    import { fly, slide } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { type SubtitleSettings, SUBTITLE_FONTS } from '../subtitles/SubtitleSettings.svelte.js';
    import {i18n} from "@/stores/i18n.svelte";

    interface SubtitleTrack { id: string; label: string; srclang: string; }

    interface Props {
        settings:      SubtitleSettings;
        tracks:        SubtitleTrack[];
        currentTrack:  string;
        onSelectTrack: (id: string) => void;
        onBack:        () => void;
    }

    let { settings, tracks, currentTrack, onSelectTrack, onBack }: Props = $props();

    let appearanceOpen = $state(false);

    let subSection = $state<string | null>(null);

    const allTracks = $derived([
        { id: '-1', label: 'Off', srclang: '' },
        ...tracks,
    ]);

    $effect(() => {
        void [
            settings.fontFamily, settings.fontSize, settings.fontWeight, settings.italic,
            settings.color, settings.outlineStyle, settings.outlineColor, settings.outlineWidth,
            settings.bgColor, settings.bgPadding, settings.bgRadius,
            settings.positionY, settings.maxWidth, settings.textAlign,
            settings.lineHeight, settings.letterSpacing, settings.opacity,
        ];
        settings.save();
    });

    const fontOptions = SUBTITLE_FONTS.map(f => ({ id: f.value, label: f.label }));

    const outlineOptions = [
        { id: 'none',        label: i18n.t('player.none')},
        { id: 'outline',     label: i18n.t('player.outline')     },
        { id: 'drop-shadow', label: i18n.t('player.drop_shadow') },
        { id: 'raised',      label: i18n.t('player.raised')      },
        { id: 'depressed',   label: i18n.t('player.depressed')  },
    ];

    const alignOptions = [
        { id: 'left',   label: i18n.t('player.left')   },
        { id: 'center', label: i18n.t('player.center') },
        { id: 'right',  label: i18n.t('player.right')  },
    ];

    const weightOptions = [
        { id: 'normal', label: i18n.t('player.normal') },
        { id: 'bold',   label: i18n.t('player.bold')  },
    ];

    const italicOptions = [
        { id: 'false', label: 'Normal' },
        { id: 'true',  label: 'Italic' },
    ];


    const currentFontLabel    = $derived(SUBTITLE_FONTS.find(f => f.value === settings.fontFamily)?.label ?? settings.fontFamily);
    const currentOutlineLabel = $derived(outlineOptions.find(o => o.id === settings.outlineStyle)?.label ?? '');
    const currentWeightLabel  = $derived(settings.fontWeight === 'bold' ? i18n.t('player.bold') : i18n.t('player.normal'));
</script>

{#snippet sliderRow(Icon: Component, label: string, badge: string, children: import('svelte').Snippet)}
    <div class="flex flex-col px-3 py-1.5">
        <div class="flex items-center justify-between mb-1.5">
            <div class="flex items-center gap-3">
                <div class="flex items-center justify-center w-8 h-8 rounded-sm bg-muted">
                    <Icon class="w-4 h-4 text-muted-foreground" />
                </div>
                <span class="text-sm font-medium">{label}</span>
            </div>
            <span class="text-xs text-muted-foreground tabular-nums">{badge}</span>
        </div>
        <div class="pl-11">
            {@render children()}
        </div>
    </div>
{/snippet}

{#snippet colorRow(Icon: Component, label: string, children: import('svelte').Snippet)}
    <div class="flex items-center justify-between w-full px-3 py-2 rounded-sm">
        <div class="flex items-center gap-3">
            <div class="flex items-center justify-center w-8 h-8 rounded-sm bg-muted">
                <Icon class="w-4 h-4 text-muted-foreground" />
            </div>
            <span class="text-sm font-medium">{label}</span>
        </div>
        {@render children()}
    </div>
{/snippet}

{#snippet drillRow(Icon: Component, label: string, value: string, id: string)}
    <Button
            variant="ghost"
            class="group flex items-center justify-between w-full px-3 py-2.5 h-auto
               rounded-sm text-foreground hover:bg-accent hover:text-accent-foreground"
            onclick={() => subSection = id}
    >
        <div class="flex items-center gap-3">
            <div class="flex items-center justify-center w-8 h-8 rounded-sm
                        bg-muted group-hover:bg-accent transition-colors duration-100">
                <Icon class="w-4 h-4 text-muted-foreground group-hover:text-accent-foreground" />
            </div>
            <span class="text-sm font-medium">{label}</span>
        </div>
        <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground font-normal">{value}</span>
            <ChevronRight class="w-4 h-4 text-muted-foreground/50 group-hover:text-muted-foreground transition-colors" />
        </div>
    </Button>
{/snippet}

{#snippet optionList(
    label: string,
    options: { id: string; label: string }[],
    current: string,
    onSelect: (id: string) => void
)}
    <div
            class="flex flex-col py-1"
            in:fly={{ x: 12, duration: 180, easing: cubicOut }}
    >
        <Button
                variant="ghost"
                class="flex items-center justify-start gap-2.5 w-full px-3 py-2.5 h-auto mb-1
                   rounded-sm border-b border-border text-foreground hover:bg-accent"
                onclick={() => subSection = null}
        >
            <ChevronLeft class="w-4 h-4 text-muted-foreground" />
            <span class="text-sm font-semibold">{label}</span>
        </Button>

        <div class="flex flex-col">
            {#each options as opt (opt.id)}
                {@const isActive = current === opt.id}
                <Button
                        variant="ghost"
                        class="flex items-center justify-start gap-3 w-full px-3 py-2.5 h-auto rounded-sm
                           {isActive ? 'text-primary hover:text-primary hover:bg-accent' : 'text-foreground hover:bg-accent'}"
                        onclick={() => { onSelect(opt.id); subSection = null; }}
                >
                    <div class="flex items-center justify-center w-5 h-5 shrink-0">
                        {#if isActive}<Check class="w-4 h-4 text-primary" />{/if}
                    </div>
                    <span class="flex-1 text-left text-sm {isActive ? 'font-medium' : 'text-muted-foreground'}">
                        {opt.label}
                    </span>
                </Button>
            {/each}
        </div>
    </div>
{/snippet}

<div
        class="flex flex-col py-1"
        in:fly={{ x: 12, duration: 180, easing: cubicOut }}
>
    {#if subSection === 'font'}
        {@render optionList(i18n.t('player.font'), fontOptions, settings.fontFamily, (v) => settings.fontFamily = v)}

    {:else if subSection === 'outline'}
        {@render optionList(i18n.t('player.outline'), outlineOptions, settings.outlineStyle, (v) => settings.outlineStyle = v as typeof settings.outlineStyle)}

    {:else if subSection === 'align'}
        {@render optionList(i18n.t('player.align'), alignOptions, settings.textAlign, (v) => settings.textAlign = v as typeof settings.textAlign)}

    {:else if subSection === 'weight'}
        {@render optionList(i18n.t('player.weigth'), weightOptions, settings.fontWeight, (v) => settings.fontWeight = v as typeof settings.fontWeight)}

    {:else if subSection === 'italic'}
        {@render optionList(i18n.t('player.style'), italicOptions, String(settings.italic), (v) => settings.italic = v === 'true')}

    {:else}
        <!-- Back header -->
        <Button
                variant="ghost"
                class="flex items-center justify-start gap-2.5 w-full px-3 py-2.5 h-auto mb-1
                   rounded-sm border-b border-border text-foreground hover:bg-accent"
                onclick={onBack}
        >
            <ChevronLeft class="w-4 h-4 text-muted-foreground" />
            <span class="text-sm font-semibold">{i18n.t('player.subtitles')}</span>
        </Button>

        <!-- Track list -->
        <div class="flex flex-col max-h-48 overflow-y-auto">
            {#each allTracks as track (track.id)}
                {@const isActive = currentTrack === track.id}
                <Button
                        variant="ghost"
                        class="flex items-center justify-start gap-3 w-full px-3 py-2.5 h-auto rounded-sm
                           {isActive ? 'text-primary hover:text-primary hover:bg-accent' : 'text-foreground hover:bg-accent'}"
                        onclick={() => onSelectTrack(track.id)}
                >
                    <div class="flex items-center justify-center w-5 h-5 shrink-0">
                        {#if isActive}<Check class="w-4 h-4 text-primary" />{/if}
                    </div>
                    <span class="flex-1 text-left text-sm {isActive ? 'font-medium' : 'text-muted-foreground'}">
                        {track.label}
                    </span>
                </Button>
            {/each}
        </div>

        <!-- Appearance accordion trigger -->
        <Separator class="my-1 opacity-60" />

        <Button
                variant="ghost"
                class="group flex items-center justify-between w-full px-3 py-2.5 h-auto
                   rounded-sm text-foreground hover:bg-accent hover:text-accent-foreground"
                onclick={() => appearanceOpen = !appearanceOpen}
        >
            <div class="flex items-center gap-3">
                <div class="flex items-center justify-center w-8 h-8 rounded-sm
                            bg-muted group-hover:bg-accent transition-colors duration-100">
                    <SlidersHorizontal class="w-4 h-4 text-muted-foreground group-hover:text-accent-foreground" />
                </div>
                <span class="text-sm font-medium">{i18n.t('player.appareance')}</span>
            </div>
            <ChevronDown
                    class="w-4 h-4 text-muted-foreground/50 transition-transform duration-200
                       {appearanceOpen ? 'rotate-180' : ''}"
            />
        </Button>

        {#if appearanceOpen}
            <div class="flex flex-col" transition:slide={{ duration: 180, easing: cubicOut }}>

                <!-- Reset -->
                <div class="flex justify-end px-3 py-1">
                    <Button
                            variant="ghost"
                            class="flex items-center gap-1.5 px-2 py-1 h-auto text-xs
                               text-muted-foreground hover:text-foreground hover:bg-accent"
                            onclick={() => settings.reset()}
                    >
                        <RotateCcw class="w-3 h-3" />
                        Reset defaults
                    </Button>
                </div>

                <!-- Font → drill-down -->
                {@render drillRow(Type, i18n.t('player.font'), currentFontLabel, 'font')}

                <!-- Font size → slider -->
                {@render sliderRow(Type, i18n.t('player.size'), `${settings.fontSize.toFixed(1)}×`, fontSizeSlider)}
                {#snippet fontSizeSlider()}
                    <Slider min={0.5} max={2.5} step={0.1}
                            value={[settings.fontSize]}
                            onValueChange={(v) => settings.fontSize = v[0]}
                            class="w-full"
                    />
                {/snippet}

                <!-- Weight → drill-down -->
                {@render drillRow(Type, i18n.t('player.weight'), currentWeightLabel, 'weight')}

                <Separator class="mx-3 my-1 opacity-40 w-auto" />

                <!-- Text colour -->
                {@render colorRow(Palette, i18n.t('player.text_color'), textColorInput)}
                {#snippet textColorInput()}
                    <input type="color" class="ctrl-color" bind:value={settings.color} />
                {/snippet}

                <!-- Opacity -->
                {@render sliderRow(Eye, i18n.t('player.opacity'), `${Math.round(settings.opacity * 100)}%`, opacitySlider)}
                {#snippet opacitySlider()}
                    <Slider min={0} max={1} step={0.05}
                            value={[settings.opacity]}
                            onValueChange={(v) => settings.opacity = v[0]}
                            class="w-full"
                    />
                {/snippet}

                <Separator class="mx-3 my-1 opacity-40 w-auto" />

                <!-- Outline → drill-down -->
                {@render drillRow(Layers, i18n.t('player.outline'), currentOutlineLabel, 'outline')}

                {#if settings.outlineStyle !== 'none'}
                    {@render colorRow(Pipette, i18n.t('player.outline_color'), outlineColorInput)}
                    {#snippet outlineColorInput()}
                        <input type="color" class="ctrl-color" bind:value={settings.outlineColor} />
                    {/snippet}

                    {@render sliderRow(Maximize2, i18n.t('player.width'), `${settings.outlineWidth}px`, outlineWidthSlider)}
                    {#snippet outlineWidthSlider()}
                        <Slider min={1} max={6} step={0.5}
                                value={[settings.outlineWidth]}
                                onValueChange={(v) => settings.outlineWidth = v[0]}
                                class="w-full"
                        />
                    {/snippet}
                {/if}

                <Separator class="mx-3 my-1 opacity-40 w-auto" />

                <!-- Background colour -->
                {@render colorRow(Square, i18n.t('player.background'), bgColorInput)}
                {#snippet bgColorInput()}
                    <input
                            type="color"
                            class="ctrl-color"
                            value={rgbaToHex(settings.bgColor)}
                            onchange={(e) => {
                            const alpha = rgbaAlpha(settings.bgColor);
                            settings.bgColor = hexToRgba((e.target as HTMLInputElement).value, alpha);
                        }}
                    />
                {/snippet}

                <!-- Background transparency -->
                {@render sliderRow(Blend, i18n.t('player.transparency'), `${Math.round(rgbaAlpha(settings.bgColor) * 100)}%`, bgAlphaSlider)}
                {#snippet bgAlphaSlider()}
                    <Slider min={0} max={1} step={0.05}
                            value={[rgbaAlpha(settings.bgColor)]}
                            onValueChange={(v) => settings.bgColor = hexToRgba(rgbaToHex(settings.bgColor), v[0])}
                            class="w-full"
                    />
                {/snippet}

                <!-- Bg padding -->
                {@render sliderRow(Square, i18n.t('player.padding'), `${settings.bgPadding}px`, bgPaddingSlider)}
                {#snippet bgPaddingSlider()}
                    <Slider min={0} max={24} step={1}
                            value={[settings.bgPadding]}
                            onValueChange={(v) => settings.bgPadding = v[0]}
                            class="w-full"
                    />
                {/snippet}

                <Separator class="mx-3 my-1 opacity-40 w-auto" />

                <!-- Vertical position -->
                {@render sliderRow(MoveVertical, i18n.t('player.vertical_position'), `${settings.positionY}%`, posYSlider)}
                {#snippet posYSlider()}
                    <Slider min={5} max={95} step={1}
                            value={[settings.positionY]}
                            onValueChange={(v) => settings.positionY = v[0]}
                            class="w-full"
                    />
                {/snippet}

                <Separator class="mx-3 my-1 opacity-40 w-auto" />

                <!-- Line height -->
                {@render sliderRow(Rows3, i18n.t('player.line_height'), settings.lineHeight.toFixed(1), lineHeightSlider)}
                {#snippet lineHeightSlider()}
                    <Slider min={1} max={2.5} step={0.1}
                            value={[settings.lineHeight]}
                            onValueChange={(v) => settings.lineHeight = v[0]}
                            class="w-full"
                    />
                {/snippet}
            </div>
        {/if}
    {/if}
</div>

<script lang="ts" module>
    export function rgbaToHex(rgba: string): string {
        const m = rgba.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
        if (!m) return rgba.startsWith('#') ? rgba.slice(0, 7) : '#000000';
        return '#' + [m[1], m[2], m[3]]
            .map(n => parseInt(n).toString(16).padStart(2, '0'))
            .join('');
    }

    export function rgbaAlpha(rgba: string): number {
        const m = rgba.match(/rgba\(\d+,\s*\d+,\s*\d+,\s*([\d.]+)\)/);
        return m ? parseFloat(m[1]) : rgba.startsWith('rgba') ? 0 : 1;
    }

    export function hexToRgba(hex: string, alpha: number): string {
        const r = parseInt(hex.slice(1, 3), 16);
        const g = parseInt(hex.slice(3, 5), 16);
        const b = parseInt(hex.slice(5, 7), 16);
        return `rgba(${r},${g},${b},${alpha})`;
    }
</script>

<style>
    .ctrl-color {
        width:         2rem;
        height:        1.75rem;
        border:        1px solid hsl(var(--border));
        border-radius: 0.25rem;
        padding:       0;
        cursor:        pointer;
        background:    none;
    }
</style>