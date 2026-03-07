<script lang="ts">
    import 'vidstack/player/styles/default/theme.css';
    import 'vidstack/player/styles/default/layouts/video.css';
    import 'vidstack/player';
    import 'vidstack/player/layouts';
    import 'vidstack/player/ui';

    import { isTauri } from '@/api/client';
    import { createTauriLoader } from '@/api/proxy/tauri-hls-loader';

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
    }

    import type { Snippet } from 'svelte';

    let { src, animeTitle, episodeTitle, subtitles = [], chapters = [], children }: Props = $props();

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

    // En Tauri, en cuanto hls.js está listo inyectamos el loader personalizado
    // que reemplaza fetch por invoke("proxy_fetch_bytes").
    function onHlsInstance(event: CustomEvent) {
        if (!isTauri()) return;
        const hls = event.detail;
        // hls.js permite cambiar el loader antes de que se cargue cualquier fuente.
        // config.loader es de solo lectura tras la construcción, pero podemos
        // usar pLoader (playlist loader) y fLoader (fragment loader) individualmente.
        const TauriLoader = createTauriLoader();
        hls.config.loader    = TauriLoader;
        hls.config.pLoader   = TauriLoader; // playlist (.m3u8)
        hls.config.fLoader   = TauriLoader; // fragments (.ts)
    }
</script>

<media-player
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