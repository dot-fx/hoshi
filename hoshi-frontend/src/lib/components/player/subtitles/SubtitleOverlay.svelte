<script lang="ts">
    import type { PlayerController } from '../PlayerController.svelte.js';
    import type { SubtitleSettings } from './SubtitleSettings.svelte.js';

    interface Props {
        controller: PlayerController;
        settings:   SubtitleSettings;
    }

    let { controller, settings }: Props = $props();

    let overlayEl: HTMLDivElement;

    $effect(() => {
        if (overlayEl) controller.attachSubtitleOverlay(overlayEl);
    });
</script>

<div
        class="subtitle-overlay"
        bind:this={overlayEl}
        style={settings.wrapperStyle}
        aria-live="polite"
        aria-atomic="true"
        role="status"
>
</div>

<style>
    .subtitle-overlay {
        position:       absolute;
        left:           50%;
        transform:      translateX(-50%);
        z-index:        50;
        width:          100%;
        pointer-events: none;
        word-break:     break-word;
        line-height:    normal;
    }
</style>