<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import { ChevronLeft, Settings2, Mic2, SkipBack, SkipForward, PuzzleIcon } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte";

    interface Props {
        cid:          string;
        animeTitle:   string;
        episodeTitle: string;

        epNumber:     number;
        hasPrev:      boolean;
        hasNext:      boolean;

        extensionItems:    { value: string; label: string }[];
        selectedExtension: string | null;
        servers:           string[];
        serverItems:       { value: string; label: string }[];
        selectedServer:    string | null;
        supportsDub:       boolean;
        isDub:             boolean;
        isLoadingPlay:     boolean;

        visible: boolean;

        onExtensionChange:  (val: string) => void;
        onServerChange:     () => void;
        onDubChange:        (val: boolean) => void;
        onManageExtensions: () => void;
        onBack: () => void;
    }

    let {
        cid,
        animeTitle,
        episodeTitle,
        epNumber,
        hasPrev,
        hasNext,
        extensionItems,
        selectedExtension = $bindable(),
        servers,
        serverItems,
        selectedServer = $bindable(),
        supportsDub,
        isDub = $bindable(),
        isLoadingPlay,
        visible,
        onExtensionChange,
        onServerChange,
        onDubChange,
        onManageExtensions,
        onBack,
    }: Props = $props();
</script>

<div
        class="player-top-bar"
        class:visible
        onclick={(e) => e.stopPropagation()}
        role="toolbar"
        aria-label="Player controls"
>
    <div class="left-group">
        <Button
                variant="ghost"
                size="icon"
                class="back-btn"
                onclick={onBack}
        >
            <ChevronLeft class="size-6 text-primary" />
        </Button>

        <div class="title-block">
            <h1 class="anime-title">{animeTitle}</h1>
            <p class="episode-title">{episodeTitle}</p>
        </div>
    </div>

    <div class="right-group">
        <!-- Prev / Next episode -->
        <div class="pill">
            <Button
                    variant="ghost"
                    size="icon"
                    disabled={!hasPrev}
                    href={`/watch/${cid}/${epNumber - 1}`}
                    class="pill-btn"
            >
                <SkipBack class="size-4" />
            </Button>
            <div class="divider"></div>
            <Button
                    variant="ghost"
                    size="icon"
                    disabled={!hasNext}
                    href={`/watch/${cid}/${epNumber + 1}`}
                    class="pill-btn"
            >
                <SkipForward class="size-4" />
            </Button>
        </div>

        <!-- Extension / server / dub pill -->
        <div class="pill pill--scrollable">
            <!-- Settings gear -->
            <Button
                    variant="ghost"
                    size="icon"
                    class="pill-btn"
                    onclick={onManageExtensions}
                    title={i18n.t('content.extension_manager.manage_extensions_title')}
            >
                <Settings2 class="size-4" />
            </Button>

            <div class="divider"></div>

            <!-- Extension select -->
            <div class="select-row">
                <PuzzleIcon class="size-4 text-primary shrink-0" />
                <ResponsiveSelect
                        bind:value={selectedExtension}
                        items={extensionItems}
                        onValueChange={(val) => onExtensionChange(val)}
                        class="select-control"
                />
            </div>

            <!-- Server select (only if >1 server) -->
            {#if servers.length > 1}
                <div class="divider"></div>
                <div class="select-row">
                    <Settings2 class="size-4 text-primary shrink-0" />
                    <ResponsiveSelect
                            bind:value={selectedServer}
                            items={serverItems}
                            placeholder={i18n.t('watch.auto_server')}
                            onValueChange={() => onServerChange()}
                            class="select-control"
                    />
                </div>
            {/if}

            <!-- Dub toggle -->
            {#if supportsDub}
                <div class="divider"></div>
                <div class="dub-row">
                    <Mic2 class="size-4 text-primary" />
                    <Label for="dub-switch" class="dub-label">
                        {i18n.t('watch.dub')}
                    </Label>
                    <Switch
                            id="dub-switch"
                            checked={isDub}
                            onCheckedChange={(v) => onDubChange(v)}
                            disabled={isLoadingPlay}
                            class="scale-75 origin-left"
                    />
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .player-top-bar {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        z-index: 60;

        display: flex;
        flex-direction: column;
        gap: 0.75rem;

        padding: max(2rem, env(safe-area-inset-top))
        max(1rem, env(safe-area-inset-right))
        1.5rem
        max(1rem, env(safe-area-inset-left));

        background: linear-gradient(
                to bottom,
                rgba(0, 0, 0, 0.9),
                rgba(0, 0, 0, 0.4) 60%,
                transparent
        );

        /* Visibility — driven by the `visible` prop */
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.3s ease;

        @media (min-width: 640px) {
            flex-direction: row;
            align-items: center;
            justify-content: space-between;
        }
    }

    .player-top-bar.visible {
        opacity: 1;
        pointer-events: auto;
    }

    /* ── Left group ──────────────────────────────────────── */
    .left-group {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        min-width: 0;
        width: 100%;

        @media (min-width: 640px) {
            width: auto;
        }
    }

    :global(.back-btn) {
        border-radius: 0.75rem;
        background: rgba(0, 0, 0, 0.4);
        color: white;
        height: 2.5rem;
        width: 2.5rem;
        flex-shrink: 0;
    }
    :global(.back-btn:hover) { background: rgba(255, 255, 255, 0.2); }

    .title-block {
        display: flex;
        flex-direction: column;
        min-width: 0;
        filter: drop-shadow(0 1px 2px rgba(0,0,0,0.8));
    }

    .anime-title {
        font-weight: 700;
        font-size: 0.875rem;
        line-height: 1.2;
        color: white;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        margin: 0;

        @media (min-width: 640px) { font-size: 1.125rem; }
    }

    .episode-title {
        font-size: 0.625rem;
        font-weight: 700;
        color: var(--color-primary, hsl(var(--primary)));
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        margin: 0;

        @media (min-width: 640px) { font-size: 0.75rem; }
    }

    /* ── Right group ─────────────────────────────────────── */
    .right-group {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-shrink: 0;
    }

    /* Shared pill */
    .pill {
        display: flex;
        align-items: center;
        background: rgba(0, 0, 0, 0.4);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 0.75rem;
        padding: 0.25rem;
        backdrop-filter: blur(12px);
    }

    .pill--scrollable {
        overflow-x: auto;
        scrollbar-width: none;
    }
    .pill--scrollable::-webkit-scrollbar { display: none; }

    :global(.pill-btn) {
        height: 2rem;
        width: 2.25rem;
        color: white;
    }
    :global(.pill-btn:hover) { color: var(--color-primary, hsl(var(--primary))); }

    .divider {
        width: 1px;
        height: 1.25rem;
        background: rgba(255, 255, 255, 0.2);
        margin: 0 0.25rem;
        flex-shrink: 0;
    }

    /* Select rows */
    .select-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding-left: 0.5rem;
        flex-shrink: 0;
    }

    :global(.select-control) {
        height: 2rem;
        border: none;
        background: transparent;
        color: white;
        font-size: 0.75rem;
        font-weight: 600;
    }
    :global(.select-control:hover) { background: rgba(255, 255, 255, 0.1); }
    :global(.select-control:focus) { ring: 0; }

    /* Dub row */
    .dub-row {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0 0.75rem;
        height: 2rem;
        border-radius: 0.5rem;
        transition: background 0.15s;
        flex-shrink: 0;
    }
    .dub-row:hover { background: rgba(255, 255, 255, 0.1); }

    .dub-label {
        font-size: 0.625rem;
        font-weight: 900;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: rgba(255, 255, 255, 0.7);
        cursor: pointer;
        white-space: nowrap;
    }
</style>