<script lang="ts">
    import VideoCore from './VideoCore.svelte';
    import Controls from './controls/Controls.svelte';
    import PlayerTopBar from './PlayerTopBar.svelte';
    import PlayerStatus from './PlayerStatus.svelte';
    import { PlayerController } from './PlayerController.svelte.js';
    import { SkipForward, Loader2 } from 'lucide-svelte';
    import type { Subtitle, Chapter } from './types.js';
    import type { PlayerState } from "@/app/watch.svelte.js";

    export type { Subtitle, Chapter };

    interface Props {
        playerState: PlayerState;
        onPlay?: () => void;
        onManageExtensions: () => void;
    }

    let { playerState, onPlay, onManageExtensions }: Props = $props();

    const ctrl = new PlayerController();

    $effect(() => ctrl.setCallbacks({
        onTimeUpdate: (data) => playerState.onTimeUpdate(data),
        onPlay,
        onPause: () => playerState.onPause(),
        onSeek: (time) => playerState.onSeek(time),
        onEnded: () => playerState.onEnded()
    }));

    $effect(() => ctrl.setSubtitles(playerState.subtitles));
    $effect(() => ctrl.setChapters(playerState.chapters));
    $effect(() => ctrl.setInitialTime(playerState.initialTime));
    $effect(() => { if (playerState.m3u8Url) ctrl.loadSrc(playerState.m3u8Url); });

    let rootEl: HTMLElement;
    const topBarVisible = $derived(!playerState.m3u8Url || ctrl.controlsVisible);

    export function getControlsVisible() { return ctrl.controlsVisible; }
    export function enterFullscreen() { ctrl.enterFullscreen(rootEl); }
</script>

<div
        bind:this={rootEl}
        class="player-root relative w-full h-full bg-black overflow-hidden select-none [&:not(:has(.controls-root.visible)):not(:has(.status-overlay))]:cursor-none"
        onmousemove={() => ctrl.nudgeControls()}
        ontouchstart={() => ctrl.nudgeControls()}
        onclick={() => { if (playerState.m3u8Url) ctrl.togglePlay(); }}
        role="presentation"
>
    {#if playerState.m3u8Url}
        <VideoCore src={playerState.m3u8Url} controller={ctrl} />
    {/if}

    <PlayerTopBar
            cid={playerState.cid}
            animeTitle={playerState.animeTitle}
            episodeTitle={playerState.episodeTitle}
            epNumber={playerState.epNumber}
            hasPrev={playerState.hasPrev}
            hasNext={playerState.hasNext}
            extensionItems={playerState.extensionItems}
            bind:selectedExtension={playerState.selectedExtension}
            servers={playerState.servers}
            serverItems={playerState.serverItems}
            bind:selectedServer={playerState.selectedServer}
            supportsDub={playerState.supportsDub}
            bind:isDub={playerState.isDub}
            isLoadingPlay={playerState.isLoadingPlay}
            visible={topBarVisible}
            onExtensionChange={(val) => playerState.selectExtension(val)}
            onServerChange={() => playerState.loadPlay()}
            onDubChange={(v) => { playerState.isDub = v; playerState.loadPlay(); }}
            {onManageExtensions}
    />

    <PlayerStatus
            error={playerState.error}
            isLoadingPlay={playerState.isLoadingPlay}
            isLoadingMeta={playerState.isLoadingMeta}
            noExtensions={!playerState.isLoadingMeta && playerState.extensions.length === 0}
            isMappingError={playerState.isMappingError}
            onRetry={() => playerState.loadPlay()}
            {onManageExtensions}
    />

    {#if ctrl.isBuffering && !playerState.isLoadingPlay}
        <div class="absolute inset-0 z-[45] flex items-center justify-center pointer-events-none">
            <Loader2 class="w-10 h-10 animate-spin text-white/80" />
        </div>
    {/if}

    {#if ctrl.showSkipButton}
        <button
                class="absolute bottom-20 right-5 z-[60] flex items-center gap-2 px-4 py-2 text-[13px] font-semibold tracking-wide text-white transition-all duration-200 border border-white/20 rounded-xl bg-black/65 backdrop-blur-md hover:bg-black/85 active:scale-95 animate-in slide-in-from-right-4 fade-in"
                onclick={(e) => { e.stopPropagation(); ctrl.skipChapter(); }}
        >
            <SkipForward class="w-4 h-4 fill-current" />
            {ctrl.skipLabel}
        </button>
    {/if}

    {#if playerState.m3u8Url}
        <Controls
                ctrl={ctrl}
                paused={ctrl.paused}
                currentTime={ctrl.currentTime}
                duration={ctrl.duration}
                buffered={ctrl.buffered}
                chapters={playerState.chapters}
                visible={ctrl.controlsVisible}
                onPlayPause={() => ctrl.togglePlay()}
                onSeek={(t) => ctrl.seek(t)}
                onFullscreen={() => ctrl.enterFullscreen(rootEl)}
        />
    {/if}
</div>