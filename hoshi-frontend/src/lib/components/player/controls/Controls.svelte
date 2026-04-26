<script lang="ts">
    import Play from '@/components/player/controls/buttons/Play.svelte';
    import TimeBar from './TimeBar.svelte';
    import TimeDisplay from './TimeDisplay.svelte';
    import Seek from '@/components/player/controls/buttons/Seek.svelte';
    import Volume from '@/components/player/controls/buttons/Volume.svelte';
    import Fullscreen from '@/components/player/controls/buttons/Fullscreen.svelte';
    import Subtitles from '@/components/player/controls/buttons/Subtitles.svelte';
    import { Settings as SettingsIcon } from 'lucide-svelte';
    import { appConfig } from "@/stores/config.svelte";
    import type { Chapter } from '../types.js';
    import type { PlayerController } from '../PlayerController.svelte.js';
    import { layoutState } from "@/stores/layout.svelte";
    import Settings from "@/components/player/controls/buttons/Settings.svelte";
    import type {SubtitleSettings} from "@/components/player/subtitles/SubtitleSettings.svelte.js";

    interface Props {
        ctrl:               PlayerController;
        paused:             boolean;
        currentTime:        number;
        duration:           number;
        buffered:           number;
        chapters:           Chapter[];
        visible:            boolean;
        extensionItems:     { value: string; label: string }[];
        selectedExtension:  string | null;
        servers:            string[];
        serverItems:        { value: string; label: string }[];
        selectedServer:     string | null;
        supportsDub:        boolean;
        isDub:              boolean;
        isLoadingPlay:      boolean;
        subtitleSettings: SubtitleSettings;
        onExtensionChange:  (val: string) => void;
        onServerChange:     () => void;
        onDubChange:        (val: boolean) => void;
        onManageExtensions: () => void;
        onPlayPause:        () => void;
        onSeek:             (time: number) => void;
        onFullscreen:       () => void;
    }

    let {
        ctrl, paused, currentTime, duration, buffered, chapters, visible,
        extensionItems, selectedExtension = $bindable(), servers, serverItems,
        selectedServer = $bindable(), supportsDub, isDub = $bindable(),
        isLoadingPlay, subtitleSettings, onExtensionChange, onServerChange, onDubChange,
        onManageExtensions, onPlayPause, onSeek, onFullscreen,
    }: Props = $props();

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
                <Play {paused} onclick={onPlayPause} />
                <Volume
                        volume={ctrl.volume}
                        muted={ctrl.muted}
                        onVolumeChange={(v) => ctrl.setVolume(v)}
                        onToggleMute={() => ctrl.toggleMute()}
                />
                <TimeDisplay {currentTime} {duration} />
                <Seek
                        seconds={-(appConfig.data?.player?.seekStep ?? 10)}
                        onclick={() => ctrl.seekBy(-(appConfig.data?.player?.seekStep ?? 10))}
                />
                <Seek
                        seconds={appConfig.data?.player?.seekStep ?? 10}
                        onclick={() => ctrl.seekBy(appConfig.data?.player?.seekStep ?? 10)}
                />
            </div>

            <div class="relative flex items-center gap-2">
                {#if ctrl.subtitleTracks.length > 0}
                    <Subtitles {ctrl} />
                {/if}
                <button
                        class="flex items-center justify-center w-9 h-9 rounded-md bg-transparent text-white/75 cursor-pointer transition-colors duration-200 hover:bg-white/15 hover:text-white {settingsOpen ? 'bg-white/20 text-white' : ''}"
                        onclick={toggleSettings}
                        title="Settings"
                        aria-label="Stream settings"
                >
                    <SettingsIcon class="w-5 h-5" />
                </button>
                <Settings
                        {ctrl}
                        open={settingsOpen}
                        {extensionItems}
                        bind:selectedExtension
                        {servers}
                        {serverItems}
                        bind:selectedServer
                        {supportsDub}
                        bind:isDub
                        {isLoadingPlay}
                        {onExtensionChange}
                        {onServerChange}
                        {onDubChange}
                        {onManageExtensions}
                        onClose={() => settingsOpen = false}
                        subtitleSettings={subtitleSettings}
                />
                {#if !layoutState.isMobile}
                    <Fullscreen onclick={onFullscreen} />
                {/if}
            </div>
        </div>
    </div>
</div>