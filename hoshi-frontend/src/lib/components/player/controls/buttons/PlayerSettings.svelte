<script lang="ts">
    import { Gauge, AudioLines, Captions, ChevronRight, ChevronLeft, Check } from 'lucide-svelte';
    import { fade, fly } from 'svelte/transition';
    import type { PlayerController } from '../../PlayerController.svelte.js';
    import {appConfig} from "@/stores/config.svelte";

    interface Props {
        ctrl: PlayerController;
        open: boolean;
        onClose: () => void;
    }

    let { ctrl, open, onClose }: Props = $props();
    let activeSection = $state<string | null>(null);

    function stopProp(e: MouseEvent) { e.stopPropagation(); }

    function onBackdropClick(e: MouseEvent) {
        if ((e.target as HTMLElement).classList.contains('settings-backdrop')) onClose();
    }

    $effect(() => { if (!open) activeSection = null; });

    const subtitleOptions = $derived([
        { id: '-1', label: 'Off' },
        ...ctrl.subtitleTracks
    ]);

    const sections = $derived([
        {
            id: 'quality',
            label: 'Quality',
            icon: Gauge,
            options: ctrl.qualityLevels,
            current: ctrl.currentQuality,
            onSelect: (id: string) => ctrl.setQuality(id),
            visible: ctrl.qualityLevels.length > 1,
        },
        {
            id: 'audio',
            label: 'Audio',
            icon: AudioLines,
            options: ctrl.audioTracks,
            current: ctrl.currentAudio,
            onSelect: (id: string) => ctrl.setAudioTrack(id),
            visible: ctrl.audioTracks.length > 1,
        },
        {
            id: 'subtitles',
            label: 'Subtitles',
            icon: Captions,
            options: subtitleOptions,
            current: ctrl.currentSubtitle,
            onSelect: (id: string) => ctrl.setSubtitleTrack(id),
            visible: ctrl.subtitleTracks.length > 0,
        },
    ]);

    function findPreferred(
        tracks: { id: string | number; srclang?: string; label: string }[],
        preference: string
    ): string | number | null {
        const langs = preference.split(',').map(l => l.trim().toLowerCase());
        for (const lang of langs) {
            const match = tracks.find(t =>
                t.srclang?.toLowerCase() === lang ||
                t.label.toLowerCase() === lang
            );
            if (match) return match.id;
        }
        return null;
    }

    let subAutoApplied = false;
    let audioAutoApplied = false;

    $effect(() => {
        const subPref = appConfig.data?.player?.preferredSubLang;
        if (subPref && ctrl.subtitleTracks.length > 0 && !subAutoApplied) {
            const id = findPreferred(ctrl.subtitleTracks, subPref);
            if (id !== null) ctrl.setSubtitleTrack(String(id));
            subAutoApplied = true;
        }
        if (ctrl.subtitleTracks.length === 0) subAutoApplied = false; // reset on new episode
    });

    $effect(() => {
        const audioPref = appConfig.data?.player?.preferredDubLang;
        if (audioPref && ctrl.audioTracks.length > 1 && !audioAutoApplied) {
            const id = findPreferred(ctrl.audioTracks, audioPref);
            if (id !== null) ctrl.setAudioTrack(Number(id));
            audioAutoApplied = true;
        }
        if (ctrl.audioTracks.length === 0) audioAutoApplied = false;
    });

    const visibleSections  = $derived(sections.filter(s => s.visible));
    const currentSection   = $derived(sections.find(s => s.id === activeSection) ?? null);
</script>

{#if open}
    <div
            class="settings-backdrop absolute inset-0 z-[55]"
            onclick={onBackdropClick}
            role="presentation"
            transition:fade={{ duration: 150 }}
    ></div>

    <div
            class="absolute bottom-14 right-0 z-[60] w-72 overflow-hidden rounded-xl border border-white/10 bg-black/90 shadow-2xl backdrop-blur-xl font-sans"
            onclick={stopProp}
            role="menu"
            transition:fly={{ y: 15, duration: 250, opacity: 0 }}
    >
        {#if currentSection === null}
            <div class="flex flex-col py-2">
                {#each visibleSections as section}
                    {@const label = section.options.find(o => o.id === section.current)?.label ?? ''}
                    <button
                            class="flex items-center justify-between w-full px-4 py-2.5 hover:bg-white/10 transition-colors duration-100"
                            onclick={() => activeSection = section.id}
                    >
                        <div class="flex items-center gap-3">
                            <section.icon class="w-5 h-5 text-white" />
                            <span class="text-sm font-medium text-white">{section.label}</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-sm text-white/60">{label}</span>
                            <ChevronRight class="w-4 h-4 text-white/50" />
                        </div>
                    </button>
                {/each}

                {#if visibleSections.length === 0}
                    <p class="text-sm text-white/50 text-center py-4 px-3">No options available.</p>
                {/if}
            </div>
        {:else}
            <div class="flex flex-col py-2">
                <button
                        class="flex items-center gap-3 w-full px-4 py-2.5 mb-1 border-b border-white/10 hover:bg-white/10 transition-colors duration-100"
                        onclick={() => activeSection = null}
                >
                    <ChevronLeft class="w-5 h-5 text-white" />
                    <span class="text-sm font-semibold text-white">{currentSection.label}</span>
                </button>

                <div class="flex flex-col max-h-64 overflow-y-auto">
                    {#each currentSection.options as opt (opt.id)}
                        {@const isActive = currentSection.current === opt.id}
                        <button
                                class="flex items-center gap-3 w-full px-4 py-2.5 hover:bg-white/10 transition-colors duration-100"
                                onclick={() => { currentSection.onSelect(opt.id); activeSection = null; }}
                        >
                            <div class="flex items-center justify-center w-5 h-5 shrink-0">
                                {#if isActive}
                                    <Check class="w-4 h-4 text-white" />
                                {/if}
                            </div>
                            <span class="flex-1 text-left text-sm {isActive ? 'text-white font-medium' : 'text-white/80'}">
                                {opt.label}
                            </span>
                        </button>
                    {/each}
                </div>
            </div>
        {/if}
    </div>
{/if}