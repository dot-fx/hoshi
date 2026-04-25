<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import type { PlayerController } from './PlayerController.svelte.js';

    interface Props {
        src: string;
        controller: PlayerController;
    }

    let { src, controller }: Props = $props();

    let videoEl: HTMLVideoElement;


    onMount(() => {
        controller.attachVideo(videoEl);
    });

    $effect(() => {
        if (src) controller.loadSrc(src);
    });

    onDestroy(() => controller.destroy());
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video
        bind:this={videoEl}
        class="video-el"
        oncanplay={() => controller.onCanPlay()}
        ontimeupdate={() => controller.onTimeUpdate()}
        onended={() => controller.onEnded()}
        onprogress={() => controller.onProgress()}
        onwaiting={() => controller.onWaiting()}
        onplaying={() => controller.onPlaying()}
        playsinline
        crossorigin="anonymous"
>
    {#each controller.subtitleTracks as sub (sub.id)}
        <track
                kind="subtitles"
                src={sub.url}
                srclang={sub.srclang}
                label={sub.label}
        />
    {/each}
</video>

<style>
    .video-el {
        width: 100%;
        height: 100%;
        object-fit: contain;
        display: block;
        background: #000;
        object-position: center;
    }
</style>