<script lang="ts">
    import { RotateCcw, RotateCw } from 'lucide-svelte';
    import { i18n } from '@/stores/i18n.svelte';

    interface Props {
        seconds: number; // positive = forward, negative = backward
        onclick: () => void;
    }

    let { seconds, onclick }: Props = $props();

    const isForward = seconds > 0;
</script>

<button
        class="flex items-center justify-center gap-1.5 h-9 px-2 rounded-md bg-transparent text-white/75 cursor-pointer transition-all duration-200 hover:bg-white/15 hover:text-white active:scale-95"
        onclick={(e) => {
        e.stopPropagation();
        onclick();
    }}
        title={`${isForward ? i18n.t("player.seek_forward") : i18n.t("player.seek_backward")} ${i18n.t("settings.player_section.seconds", { num: Math.abs(seconds) })}`}
        aria-label={`${isForward ? i18n.t("player.seek_forward") : i18n.t("player.seek_backward")} ${i18n.t("settings.player_section.seconds", { num: Math.abs(seconds) })}`}
>
    {#if isForward}
        <RotateCw class="w-4 h-4" />
    {:else}
        <RotateCcw class="w-4 h-4" />
    {/if}
</button>