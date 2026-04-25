<script lang="ts">
    import { Maximize, Minimize } from 'lucide-svelte';

    interface Props {
        onclick: () => void;
    }

    let { onclick }: Props = $props();
    let isFullscreen = $state(!!document.fullscreenElement);

    $effect(() => {
        function onChange() {
            isFullscreen = !!document.fullscreenElement;
        }
        document.addEventListener('fullscreenchange', onChange);
        return () => document.removeEventListener('fullscreenchange', onChange);
    });
</script>

<button
        class="flex items-center justify-center w-9 h-9 rounded-md bg-transparent text-white/75 cursor-pointer transition-all duration-200 hover:bg-white/15 hover:text-white active:scale-95"
        onclick={(e) => {
        e.stopPropagation();
        onclick();
    }}
        title={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}
        aria-label={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}
>
    {#if isFullscreen}
        <Minimize class="w-5 h-5" />
    {:else}
        <Maximize class="w-5 h-5" />
    {/if}
</button>