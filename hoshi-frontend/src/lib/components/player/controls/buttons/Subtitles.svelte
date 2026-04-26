<script lang="ts">
    import { Captions, CaptionsOff } from 'lucide-svelte';
    import type { PlayerController } from '../../PlayerController.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';

    interface Props {
        ctrl: PlayerController;
    }

    let { ctrl }: Props = $props();

    const isOn = $derived(ctrl.currentSubtitle !== '-1');

    function toggle(e: MouseEvent) {
        e.stopPropagation();
        if (isOn) {
            ctrl.setSubtitleTrack('-1');
        } else {
            // Re-enable the first available track
            const first = ctrl.subtitleTracks[0];
            if (first) ctrl.setSubtitleTrack(first.id);
        }
    }
</script>

<button
        class="flex items-center justify-center w-9 h-9 rounded-md bg-transparent cursor-pointer transition-colors duration-200
        {isOn ? 'text-white hover:bg-white/15' : 'text-white/50 hover:bg-white/15 hover:text-white/75'}"
        onclick={toggle}
        title={isOn ? i18n.t("player.cc_off") : i18n.t("player.cc_on")}
        aria-label={isOn ? i18n.t("player.cc_off") : i18n.t("player.cc_on")}
>
    {#if isOn}
        <Captions class="w-5 h-5" />
    {:else}
        <CaptionsOff class="w-5 h-5" />
    {/if}
</button>