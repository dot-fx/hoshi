<script lang="ts">
    interface Props {
        currentTime: number;
        duration: number;
    }

    let { currentTime, duration }: Props = $props();

    function format(s: number): string {
        if (!isFinite(s) || s < 0) s = 0;
        const h = Math.floor(s / 3600);
        const m = Math.floor((s % 3600) / 60);
        const sec = Math.floor(s % 60);
        if (h > 0) {
            return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
        }
        return `${m}:${String(sec).padStart(2, '0')}`;
    }
</script>

<div class="time-display">
    <span class="current">{format(currentTime)}</span>
    <span class="separator">/</span>
    <span class="total">{format(duration)}</span>
</div>

<style>
    .time-display {
        display: flex;
        align-items: center;
        gap: 3px;
        font-variant-numeric: tabular-nums;
        font-size: 13px;
        font-weight: 500;
        letter-spacing: 0.02em;
        color: rgba(255, 255, 255, 0.9);
        user-select: none;
        white-space: nowrap;
    }

    .separator {
        color: rgba(255, 255, 255, 0.35);
        margin: 0 1px;
    }

    .total {
        color: rgba(255, 255, 255, 0.5);
    }
</style>