<script lang="ts">
    import PlayButton from './buttons/PlayButton.svelte';
    import TimeBar from './TimeBar.svelte';
    import TimeDisplay from './TimeDisplay.svelte';
    import SeekButton from './buttons/SeekButton.svelte';
    import PlayerSettings from './buttons/PlayerSettings.svelte';
    import VolumeControl from './VolumeControl.svelte';
    import FullscreenButton from "@/components/player/controls/buttons/FullscreenButton.svelte";
    import { Settings } from 'lucide-svelte';
    import SubtitleButton from './buttons/SubtitleButton.svelte';
    import {appConfig} from "@/stores/config.svelte";

    import type { Chapter } from '../types.js';
    import type { PlayerController } from '../PlayerController.svelte.js';

    interface Props {
        ctrl: PlayerController;
        paused: boolean;
        currentTime: number;
        duration: number;
        buffered: number;
        chapters: Chapter[];
        visible: boolean;
        onPlayPause: () => void;
        onSeek: (time: number) => void;
        onFullscreen: () => void;
    }

    let { ctrl, paused, currentTime, duration, buffered, chapters, visible, onPlayPause, onSeek, onFullscreen }: Props = $props();
    let settingsOpen = $state(false);

    function toggleSettings(e: MouseEvent) {
        e.stopPropagation();
        settingsOpen = !settingsOpen;
    }

    $effect(() => { if (!visible) settingsOpen = false; });
</script>

<div
        class="controls-root absolute inset-x-0 bottom-0 z-50 w-full transition-opacity duration-300 {visible ? 'visible opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}"
        onclick={(e) => e.stopPropagation()}
>
    <div class="absolute inset-0 pointer-events-none bg-gradient-to-t from-black/80 via-black/40 to-transparent"></div>

    <div class="relative flex flex-col gap-1 px-4 pb-3">
        <TimeBar {currentTime} {duration} {buffered} {chapters} {onSeek} />

        <div class="flex items-center justify-between gap-3 pt-2">
            <div class="flex items-center gap-3">
                <PlayButton {paused} onclick={onPlayPause} />
                <SeekButton
                        seconds={-(appConfig.data?.player?.seekStep ?? 10)}
                        onclick={() => ctrl.seekBy(-(appConfig.data?.player?.seekStep ?? 10))}
                />
                <SeekButton
                        seconds={appConfig.data?.player?.seekStep ?? 10}
                        onclick={() => ctrl.seekBy(appConfig.data?.player?.seekStep ?? 10)}
                />
                <VolumeControl
                        volume={ctrl.volume}
                        muted={ctrl.muted}
                        onVolumeChange={(v) => ctrl.setVolume(v)}
                        onToggleMute={() => ctrl.toggleMute()}
                />
                <TimeDisplay {currentTime} {duration} />
            </div>

            <div class="relative flex items-center gap-2">
                {#if ctrl.subtitleTracks.length > 0}
                    <SubtitleButton {ctrl} />
                {/if}
                <button
                        class="flex items-center justify-center w-9 h-9 rounded-md bg-transparent text-white/75 cursor-pointer transition-colors duration-200 hover:bg-white/15 hover:text-white {settingsOpen ? 'bg-white/20 text-white' : ''}"
                        onclick={toggleSettings}
                        title="Settings"
                        aria-label="Stream settings"
                >
                    <Settings class="w-5 h-5" />
                </button>
                <PlayerSettings {ctrl} open={settingsOpen} onClose={() => settingsOpen = false} />
                <FullscreenButton onclick={onFullscreen} />
            </div>
        </div>
    </div>
</div>