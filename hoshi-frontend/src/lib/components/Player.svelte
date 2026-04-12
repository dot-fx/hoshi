<script lang="ts">
    import 'vidstack/player/styles/default/theme.css';
    import 'vidstack/player/styles/default/layouts/video.css';
    import 'vidstack/player';
    import 'vidstack/player/layouts';
    import 'vidstack/player/ui';

    import { isTauri } from '@/api/client';
    import { createTauriLoader } from '@/api/proxy/tauri-hls-loader';
    import { appConfig } from '@/stores/config.svelte.js';
    import { untrack, type Snippet } from 'svelte';
    import { i18n } from "@/i18n/index.svelte";
    import type { DefaultLayoutTranslations } from "vidstack";
    import type { VideoState } from '@/api/watchparty/types';
    import { themeManager } from '@/stores/theme.svelte.js';

    export interface Subtitle {
        id: string;
        url: string;
        language: string;
        type: string;
    }

    export interface Chapter {
        start: number;
        end: number;
        title: string;
    }

    interface Props {
        src: string;
        animeTitle: string;
        episodeTitle: string;
        subtitles?: Subtitle[];
        chapters?: Chapter[];
        children?: Snippet;
        cid: string;
        episode: number;
        initialTime?: number;

        isHost?: boolean;
        syncState?: VideoState | null;
        onPlay?: () => void;
        onPause?: () => void;
        onSeek?: (time: number) => void;
        onTimeUpdate?: (data: { currentTime: number; duration: number; paused: boolean }) => void;

        onEnded?: () => void;
    }

    let {
        src,
        animeTitle,
        episodeTitle,
        subtitles = [],
        chapters = [],
        children,
        initialTime = 0,
        isHost = true,
        syncState = null,
        onPlay,
        onPause,
        onSeek,
        onTimeUpdate,
        onEnded
    }: Props = $props();

    let player = $state<any>(null);
    let hasSeeked = $state(false);
    let playerTranslations = $derived(i18n.locale ? getPlayerTranslations() : getPlayerTranslations());

    $effect(() => {
        if (player && src) {
            player.src = { src: src, type: 'application/vnd.apple.mpegurl' };
            untrack(() => {
                hasSeeked = false;
            });
        }
    });

    $effect(() => {
        if (player && appConfig.data) {
            player.keyStep = appConfig.data.player.seekStep;
        }
    });

    $effect(() => {
        if (!player || isHost || !syncState) return;

        const now = Date.now();
        let targetPos = syncState.position;

        if (syncState.status === 'playing') {
            targetPos += (now - syncState.updatedAt) / 1000;
        }

        if (Math.abs(player.currentTime - targetPos) > 2) {
            player.currentTime = targetPos;
        }

        if (syncState.status === 'playing' && player.paused) {
            player.play().catch(() => console.warn('Autoplay bloqueado por el navegador'));
        } else if (syncState.status === 'paused' && !player.paused) {
            player.pause();
        }
    });

    $effect(() => {
        if (!player) return;

        const handler = (e: Event) => {
            if (!isHost) {
                console.log("blocked request:", e.type);
                e.stopImmediatePropagation();
                e.preventDefault?.();
            }
        };

        const events = [
            "media-play-request",
            "media-pause-request",
            "media-seek-request",
            "media-rate-change-request"
        ];

        for (const ev of events) {
            player.addEventListener(ev, handler, { capture: true });
        }

        return () => {
            for (const ev of events) {
                player.removeEventListener(ev, handler, { capture: true });
            }
        };
    });

    function handleCanPlay() {
        if (!player) return;

        if (!hasSeeked) {
            if (initialTime > 0) {
                player.currentTime = initialTime;
            }
            hasSeeked = true;
        }

        if (!isHost && syncState) {
            const now = Date.now();
            let targetPos = syncState.position;

            if (syncState.status === 'playing') {
                targetPos += (now - syncState.updatedAt) / 1000;
            }

            player.currentTime = targetPos;

            if (syncState.status === 'playing') {
                player.play().catch(() => {});
            } else {
                player.pause();
            }
        }
    }

    function handleTimeUpdate() {
        if (!player) return;
        const currentTime = player.currentTime;
        const duration = player.duration || 0;

        onTimeUpdate?.({ currentTime, duration, paused: player.paused });

        const config = appConfig.data?.player;
        if (config) {
            const currentChapter = chapters.find(ch => currentTime >= ch.start && currentTime <= (ch.end - 1));
            if (currentChapter) {
                const title = currentChapter.title.toLowerCase();
                const isIntro = title.includes('op') || title.includes('intro') || title.includes('opening');
                const isOutro = title.includes('ed') || title.includes('outro') || title.includes('ending');
                if ((config.autoSkipIntro && isIntro) || (config.autoSkipOutro && isOutro)) {
                    player.currentTime = currentChapter.end;
                }
            }
        }
    }

    function onPlayEvent(e: Event) { if (isHost) onPlay?.(); }
    function onPauseEvent(e: Event) { if (isHost) onPause?.(); }
    function onSeekEvent(e: Event) { if (isHost && player) onSeek?.(player.currentTime); }

    function onHlsInstance(e: Event) {
        if (!isTauri()) return;
        const event = e as CustomEvent;
        const hls = event.detail;
        const TauriLoader = createTauriLoader();
        hls.config.loader = TauriLoader;
        hls.config.pLoader = TauriLoader;
        hls.config.fLoader = TauriLoader;
    }

    function formatVttTime(seconds: number) {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = (seconds % 60).toFixed(3);
        return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${s.padStart(6, "0")}`;
    }

    let chaptersTrackUrl = $derived.by(() => {
        if (!chapters || chapters.length === 0) return null;
        let vtt = "WEBVTT\n\n";
        for (const ch of chapters) {
            vtt += `${formatVttTime(ch.start)} --> ${formatVttTime(ch.end)}\n${ch.title}\n\n`;
        }
        return URL.createObjectURL(new Blob([vtt], { type: "text/vtt" }));
    });

    function getPlayerTranslations(): DefaultLayoutTranslations {
        return {
            'Caption Styles': i18n.t('player.caption_styles'),
            'Captions look like this': i18n.t('player.captions_preview'),
            'Closed-Captions Off': i18n.t('player.cc_off'),
            'Closed-Captions On': i18n.t('player.cc_on'),
            'Display Background': i18n.t('player.display_background'),
            'Enter Fullscreen': i18n.t('player.enter_fullscreen'),
            'Enter PiP': i18n.t('player.enter_pip'),
            'Exit Fullscreen': i18n.t('player.exit_fullscreen'),
            'Exit PiP': i18n.t('player.exit_pip'),
            'Google Cast': i18n.t('player.google_cast'),
            'Keyboard Animations': i18n.t('player.keyboard_animations'),
            'Seek Backward': i18n.t('player.seek_backward'),
            'Seek Forward': i18n.t('player.seek_forward'),
            'Skip To Live': i18n.t('player.skip_to_live'),
            'Text Background': i18n.t('player.text_background'),
            Accessibility: i18n.t('player.accessibility'),
            AirPlay: i18n.t('player.airplay'),
            Announcements: i18n.t('player.announcements'),
            Audio: i18n.t('player.audio'),
            Auto: i18n.t('player.auto'),
            Boost: i18n.t('player.boost'),
            Captions: i18n.t('player.captions'),
            Chapters: i18n.t('player.chapters'),
            Color: i18n.t('player.color'),
            Connected: i18n.t('player.connected'),
            Connecting: i18n.t('player.connecting'),
            Continue: i18n.t('player.continue'),
            Default: i18n.t('player.default'),
            Disabled: i18n.t('player.disabled'),
            Disconnected: i18n.t('player.disconnected'),
            Download: i18n.t('player.download'),
            Family: i18n.t('player.font_family'),
            Font: i18n.t('player.font'),
            Fullscreen: i18n.t('player.fullscreen'),
            LIVE: i18n.t('player.live'),
            Loop: i18n.t('player.loop'),
            Mute: i18n.t('player.mute'),
            Normal: i18n.t('player.normal'),
            Off: i18n.t('player.off'),
            Opacity: i18n.t('player.opacity'),
            Pause: i18n.t('player.pause'),
            PiP: i18n.t('player.pip'),
            Play: i18n.t('player.play'),
            Playback: i18n.t('player.playback'),
            Quality: i18n.t('player.quality'),
            Replay: i18n.t('player.replay'),
            Reset: i18n.t('player.reset'),
            Seek: i18n.t('player.seek'),
            Settings: i18n.t('player.settings'),
            Shadow: i18n.t('player.shadow'),
            Size: i18n.t('player.size'),
            Speed: i18n.t('player.speed'),
            Text: i18n.t('player.text'),
            Track: i18n.t('player.track'),
            Unmute: i18n.t('player.unmute'),
            Volume: i18n.t('player.volume'),
        };
    }

    $effect(() => {
        if (player && appConfig.data) {
            const prefAudio = appConfig.data.player.preferredDubLang;
            const prefText = appConfig.data.player.preferredSubLang;

            if (prefAudio) {
                player.audioTracks.preferredLanguages = prefAudio.split(',').map(l => l.trim().toLowerCase());
            }

            if (prefText) {
                player.textTracks.preferredLanguages = prefText.split(',').map(l => l.trim().toLowerCase());

                player.textTracks.preferredMode = 'showing';
            }
        }
    });
</script>

<media-player
        bind:this={player}
        on:can-play={handleCanPlay}
        on:time-update={handleTimeUpdate}
        on:play={onPlayEvent}
        on:pause={onPauseEvent}
        on:seeked={onSeekEvent}
        on:ended={() => onEnded?.()}
        on:hls-instance={onHlsInstance}

        class="w-full h-full bg-black overflow-hidden"
        title={`${animeTitle} - ${episodeTitle}`}
        playsInline
        crossOrigin="anonymous"
>
    <media-provider>
        {#each subtitles as sub}
            <track kind="subtitles" src={sub.url} srclang={sub.id} label={sub.language} />
        {/each}
        {#if chaptersTrackUrl}
            <track kind="chapters" src={chaptersTrackUrl} srclang="es" default />
        {/if}
    </media-provider>

    <media-gesture
            event="dblpointerup"
            action={`seek:-${appConfig.data?.player.seekStep || 10}`}
            class="absolute top-0 left-0 z-10 w-1/5 h-full"
    ></media-gesture>

    <media-gesture
            event="dblpointerup"
            action={`seek:${appConfig.data?.player.seekStep || 10}`}
            class="absolute top-0 right-0 z-10 w-1/5 h-full"
    ></media-gesture>

    <media-video-layout
            translations={playerTranslations}
            color-scheme={themeManager.theme === 'light' ? 'light' : 'dark'}
    ></media-video-layout>
    {@render children?.()}
</media-player>