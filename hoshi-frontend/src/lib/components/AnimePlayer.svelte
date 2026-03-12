<script lang="ts">
    import 'vidstack/player/styles/default/theme.css';
    import 'vidstack/player/styles/default/layouts/video.css';
    import 'vidstack/player';
    import 'vidstack/player/layouts';
    import 'vidstack/player/ui';

    import { isTauri } from '@/api/client';
    import { createTauriLoader } from '@/api/proxy/tauri-hls-loader';
    import { appConfig } from '@/config.svelte'; //
    import { goto } from '$app/navigation';
    import type { Snippet } from 'svelte';

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
        nextRoute?: string | null; // Added to handle autoplay
    }

    let {
        src,
        animeTitle,
        episodeTitle,
        subtitles = [],
        chapters = [],
        children,
        nextRoute = null
    }: Props = $props();

    let player = $state<any>(null); // Reference to the Vidstack player

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

    // --- FEATURE: SEEK STEP ---
    $effect(() => {
        if (player && appConfig.data) {
            // Vidstack uses 'keyStep' to control arrow key skip intervals
            player.keyStep = appConfig.data.player.seekStep;
        }
    });

    // --- FEATURE: AUTO-SKIP INTRO/OUTRO ---
    function handleTimeUpdate(event: CustomEvent) {
        if (!appConfig.data || !player) return;

        const currentTime = event.detail.currentTime;
        const config = appConfig.data.player;

        // Find if we are inside a chapter that should be skipped
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

    // --- FEATURE: AUTOPLAY NEXT ---
    function handleEnded() {
        if (appConfig.data?.player.autoplayNextEpisode && nextRoute) {
            goto(nextRoute);
        }
    }

    function onHlsInstance(event: CustomEvent) {
        if (!isTauri()) return;
        const hls = event.detail;
        const TauriLoader = createTauriLoader();
        hls.config.loader    = TauriLoader;
        hls.config.pLoader   = TauriLoader;
        hls.config.fLoader   = TauriLoader;
    }
</script>

<media-player
        bind:this={player}
        on:time-update={handleTimeUpdate}
        on:ended={handleEnded}
        class="w-full h-full bg-black overflow-hidden"
        title={`${animeTitle} - ${episodeTitle}`}
        src={[{ src: src, type: 'application/vnd.apple.mpegurl' }]}
        playsInline
        crossOrigin="anonymous"
        on:hls-instance={onHlsInstance}
>
    <media-provider>
        {#each subtitles as sub}
            <track
                    kind="subtitles"
                    src={sub.url}
                    srclang={sub.id}
                    label={sub.language}
            />
        {/each}

        {#if chaptersTrackUrl}
            <track
                    kind="chapters"
                    src={chaptersTrackUrl}
                    srclang="es"
                    default
            />
        {/if}
    </media-provider>

    <media-video-layout></media-video-layout>
    {@render children?.()}
</media-player>