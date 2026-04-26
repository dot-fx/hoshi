<script lang="ts">
    import { Play, Pause } from 'lucide-svelte';
    import {i18n} from "@/stores/i18n.svelte";

    interface Props {
        paused: boolean;
        onclick: () => void;
    }

    let { paused, onclick }: Props = $props();
</script>

<button
        class="flex shrink-0 items-center justify-center w-10 h-10 rounded-full bg-transparent text-white transition-all duration-200 hover:bg-white/20 hover:scale-105 active:scale-95 cursor-pointer"
        onclick={(e) => {
        e.stopPropagation();
        onclick();
    }}
        aria-label={paused ? i18n.t("player.play") : i18n.t("player.pause")}
>
    {#if paused}
        <Play class="w-6 h-6 ml-0.5 fill-current" />
    {:else}
        <Pause class="w-6 h-6 fill-current" />
    {/if}
</button>