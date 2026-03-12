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

    import { progressApi } from '@/api/progress/progress';
    import { listApi } from '@/api/list/list';

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

    // --- ESTADOS PARA CONTROL DE PROGRESO ---
    let lastSyncTime = $state(0);
    let hasUpdatedList = $state(false);
    let hasSeeked = $state(false); // Para asegurar que salte de tiempo una sola vez

    $effect(() => {
        src;
        episode;
        untrack(() => {
            lastSyncTime = 0;
            hasUpdatedList = false;
            hasSeeked = false; // Resetear el seek si cambian de episodio
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

    // --- NUEVO: AUTO-RESUME CUANDO EL REPRODUCTOR CARGUE ---
    function handleCanPlay() {
        if (!hasSeeked && appConfig.data?.player.resumeFromLastPos) {
            // 1. Prioridad principal: si viene el parámetro ?t= en la URL
            const urlParams = new URLSearchParams(window.location.search);
            const t = urlParams.get('t');

            if (t && !isNaN(Number(t))) {
                player.currentTime = Number(t);
                hasSeeked = true;
            } else {
                // 2. Fallback: Verificamos en el historial por si viene de la página general
                progressApi.getContentProgress(cid).then(res => {
                    const prog = res.animeProgress.find(p => p.episode === episode);
                    if (prog && prog.timestampSeconds > 0 && player && !hasSeeked) {
                        player.currentTime = prog.timestampSeconds;
                        hasSeeked = true;
                    }
                }).catch(() => {}); // Fallo silencioso si no hay historial
            }
        }
    }

    function handleTimeUpdate(event: CustomEvent) {
        if (!appConfig.data || !player) return;

        const currentTime = event.detail.currentTime;
        const duration = player.duration || 0;
        const config = appConfig.data.player;

        // Auto-skip Intro/Outro
        const currentChapter = chapters.find(ch => currentTime >= ch.start && currentTime <= (ch.end - 1));
        if (currentChapter) {
            const title = currentChapter.title.toLowerCase();
            const isIntro = title.includes('op') || title.includes('intro') || title.includes('opening');
            const isOutro = title.includes('ed') || title.includes('outro') || title.includes('ending');
            if ((config.autoSkipIntro && isIntro) || (config.autoSkipOutro && isOutro)) {
                player.currentTime = currentChapter.end;
            }
        }

        // --- WATCH HISTORY (HEARTBEAT) ---
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

        // --- AUTO-UPDATE TRACKING LIST ---
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
        on:can-play={handleCanPlay}
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
            <track kind="subtitles" src={sub.url} srclang={sub.id} label={sub.language} />
        {/each}
        {#if chaptersTrackUrl}
            <track kind="chapters" src={chaptersTrackUrl} srclang="es" default />
        {/if}
    </media-provider>
    <media-video-layout></media-video-layout>
    {@render children?.()}
</media-player>