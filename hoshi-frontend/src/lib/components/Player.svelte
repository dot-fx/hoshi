<script lang="ts">
    import 'vidstack/player/styles/default/theme.css';
    import 'vidstack/player/styles/default/layouts/video.css';
    import 'vidstack/player';
    import 'vidstack/player/layouts';
    import 'vidstack/player/ui';

    import { isTauri } from '@/api/client';
    import { createTauriLoader } from '@/api/proxy/tauri-hls-loader';
    import { appConfig } from '@/config.svelte';
    import { goto } from '$app/navigation';
    import { untrack, type Snippet } from 'svelte';
    import {i18n} from "@/i18n/index.svelte";

    import { progressApi } from '@/api/progress/progress';
    import { listApi } from '@/api/list/list';
    import type {DefaultLayoutTranslations} from "vidstack";

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
        nextRoute?: string | null;
        cid: string;
        episode: number;
        totalEpisodes: number;
    }

    let {
        src,
        animeTitle,
        episodeTitle,
        subtitles = [],
        chapters = [],
        children,
        nextRoute = null,
        cid,
        episode,
        totalEpisodes
    }: Props = $props();

    let player = $state<any>(null);

    let lastSyncTime = $state(0);
    let hasUpdatedList = $state(false);
    let hasSeeked = $state(false);
    let playerTranslations = $derived(i18n.locale ? getPlayerTranslations() : getPlayerTranslations());

    $effect(() => {
        src;
        episode;
        untrack(() => {
            lastSyncTime = 0;
            hasUpdatedList = false;
            hasSeeked = false;
        });
    });

    let chaptersTrackUrl = $derived.by(() => {
        if (!chapters || chapters.length === 0) return null;
        let vtt = "WEBVTT\n\n";
        for (const ch of chapters) {
            vtt += `${formatVttTime(ch.start)} --> ${formatVttTime(ch.end)}\n${ch.title}\n\n`;
        }
        return URL.createObjectURL(new Blob([vtt], { type: "text/vtt" }));
    });

    function formatVttTime(seconds: number) {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = (seconds % 60).toFixed(3);
        return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${s.padStart(6, "0")}`;
    }

    $effect(() => {
        if (player && appConfig.data) {
            player.keyStep = appConfig.data.player.seekStep;
        }
    });

    function handleCanPlay() {
        if (!hasSeeked && appConfig.data?.player.resumeFromLastPos) {
            const urlParams = new URLSearchParams(window.location.search);
            const t = urlParams.get('t');

            if (t && !isNaN(Number(t))) {
                player.currentTime = Number(t);
                hasSeeked = true;
            } else {
                progressApi.getContentProgress(cid).then(res => {
                    const prog = res.animeProgress.find(p => p.episode === episode);
                    if (prog && prog.timestampSeconds > 0 && player && !hasSeeked) {
                        player.currentTime = prog.timestampSeconds;
                        hasSeeked = true;
                    }
                }).catch(() => {});
            }
        }
    }

    function handleTimeUpdate(event: Event) {
        if (!appConfig.data || !player) return;

        const currentTime = player.currentTime;
        const duration = player.duration || 0;
        const config = appConfig.data.player;

        const currentChapter = chapters.find(ch => currentTime >= ch.start && currentTime <= (ch.end - 1));
        if (currentChapter) {
            const title = currentChapter.title.toLowerCase();
            const isIntro = title.includes('op') || title.includes('intro') || title.includes('opening');
            const isOutro = title.includes('ed') || title.includes('outro') || title.includes('ending');
            if ((config.autoSkipIntro && isIntro) || (config.autoSkipOutro && isOutro)) {
                player.currentTime = currentChapter.end;
            }
        }

        if (Math.abs(currentTime - lastSyncTime) >= 10 || (lastSyncTime === 0 && currentTime > 2)) {
            lastSyncTime = currentTime;
            progressApi.updateAnimeProgress({
                cid,
                episode,
                timestampSeconds: Math.floor(currentTime),
                episodeDurationSeconds: Math.floor(duration) > 0 ? Math.floor(duration) : undefined,
                completed: duration > 0 && (currentTime / duration) >= 0.9
            }).catch(e => console.error("History sync failed", e));
        }

        if (!hasUpdatedList && duration > 0 && appConfig.data.content.autoUpdateProgress) {
            if (currentTime / duration >= 0.8) {
                hasUpdatedList = true;
                const status = (totalEpisodes > 0 && episode >= totalEpisodes) ? "COMPLETED" : "CURRENT";

                listApi.upsert({
                    cid,
                    status,
                    progress: episode
                }).catch(e => console.error("List sync failed", e));
            }
        }
    }

    function handleEnded() {
        if (appConfig.data?.player.autoplayNextEpisode && nextRoute) {
            goto(nextRoute);
        }
    }

    function onHlsInstance(e: Event) {
        if (!isTauri()) return;
        const event = e as CustomEvent;
        const hls = event.detail;

        const TauriLoader = createTauriLoader();
        hls.config.loader    = TauriLoader;
        hls.config.pLoader   = TauriLoader;
        hls.config.fLoader   = TauriLoader;
    }

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
</script>

<media-player
        bind:this={player}
        on:can-play={handleCanPlay}
        on:time-update={handleTimeUpdate}
        on:ended={handleEnded}
        on:hls-instance={onHlsInstance}
        class="w-full h-full bg-black overflow-hidden"
        title={`${animeTitle} - ${episodeTitle}`}
        src={[{ src: src, type: 'application/vnd.apple.mpegurl' }]}
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

    <media-video-layout translations={playerTranslations}></media-video-layout>

    {@render children?.()}
</media-player>