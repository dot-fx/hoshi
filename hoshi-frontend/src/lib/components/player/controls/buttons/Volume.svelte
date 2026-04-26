<script lang="ts">
    import { VolumeX, Volume1, Volume2 } from 'lucide-svelte';
    import { i18n } from '@/stores/i18n.svelte';

    interface Props {
        volume: number;
        muted: boolean;
        onVolumeChange: (v: number) => void;
        onToggleMute: () => void;
    }

    let { volume, muted, onVolumeChange, onToggleMute }: Props = $props();

    let hovered = $state(false);
    let dragging = $state(false);

    const showSlider = $derived(hovered || dragging);
    let sliderValue = $state(volume);

    const effectiveVolume = $derived(muted ? 0 : volume);
    let fill = $derived(effectiveVolume * 100);

    $effect(() => {
        if (!dragging) sliderValue = effectiveVolume;
    });

    function onSliderInput(e: Event) {
        const val = parseFloat((e.currentTarget as HTMLInputElement).value);
        sliderValue = val;
        onVolumeChange(val);
        if (muted && val > 0) onToggleMute();
    }

    function onSliderMouseDown() { dragging = true; }
    function onSliderMouseUp()   { dragging = false; }
</script>

<div
        class="flex items-center overflow-hidden group"
        onmouseenter={() => hovered = true}
        onmouseleave={() => hovered = false}
>
    <button
            class="flex shrink-0 items-center justify-center w-9 h-9 rounded-md bg-transparent text-white/75 cursor-pointer transition-all duration-200 hover:bg-white/15 hover:text-white active:scale-95"
            onclick={(e) => {
                e.stopPropagation();
                onToggleMute();
            }}
            title={muted ? i18n.t("player.unmute") : i18n.t("player.mute")}
            aria-label={muted ? i18n.t("player.unmute") : i18n.t("player.mute")}
    >
        {#if effectiveVolume === 0}
            <VolumeX class="w-5 h-5" />
        {:else if effectiveVolume < 0.5}
            <Volume1 class="w-5 h-5" />
        {:else}
            <Volume2 class="w-5 h-5" />
        {/if}
    </button>

    <div class="overflow-hidden transition-all duration-300 ease-out flex items-center {showSlider ? 'w-[72px] opacity-100 ml-1.5' : 'w-0 opacity-0'}">
        <input
                type="range"
                class="volume-slider w-[72px] h-1.5 rounded-full appearance-none cursor-pointer outline-none"
                min="0"
                max="1"
                step="0.01"
                value={sliderValue}
                oninput={onSliderInput}
                onmousedown={onSliderMouseDown}
                onmouseup={onSliderMouseUp}
                aria-label="Volume"
                style="background: linear-gradient(to right, #fff {fill}%, rgba(255,255,255,0.2) {fill}%);"
        />
    </div>
</div>

<style>
    .volume-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #fff;
        cursor: pointer;
        transition: transform 0.15s ease;
    }
    .volume-slider::-webkit-slider-thumb:hover {
        transform: scale(1.25);
    }
    .volume-slider::-moz-range-thumb {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #fff;
        border: none;
        cursor: pointer;
        transition: transform 0.15s ease;
    }
    .volume-slider::-moz-range-thumb:hover {
        transform: scale(1.25);
    }
</style>