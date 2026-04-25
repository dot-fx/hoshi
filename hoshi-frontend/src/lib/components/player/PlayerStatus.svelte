<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { AlertCircle, PuzzleIcon, Loader2 } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte";
    import type { CoreError } from "@/api/client";

    interface Props {
        error: CoreError | null;
        isLoadingPlay: boolean;
        isLoadingMeta: boolean;
        noExtensions: boolean;
        isMappingError: boolean;
        onRetry: () => void;
        onManageExtensions: () => void;
    }

    let {
        error,
        isLoadingPlay,
        isLoadingMeta,
        noExtensions,
        isMappingError,
        onRetry,
        onManageExtensions,
    }: Props = $props();

    const visible = $derived(!!error || isLoadingPlay || (!isLoadingMeta && noExtensions));
</script>

{#if visible}
    <div class="status-overlay absolute inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onclick={(e) => e.stopPropagation()} role="status">

        {#if error}
            <div class="flex flex-col items-center gap-5 p-6 max-w-md text-center bg-background/95 border border-white/10 rounded-2xl shadow-2xl animate-in fade-in zoom-in-95 duration-300">
                <div class="p-4 rounded-full bg-destructive/10 ring-1 ring-destructive/20">
                    <AlertCircle class="w-10 h-10 text-destructive" />
                </div>

                <p class="m-0 text-lg font-bold leading-snug text-white/90">
                    {i18n.t(error.key) || error.message || error.key}
                </p>

                <div class="flex flex-wrap justify-center gap-3 mt-2">
                    <Button variant="secondary" onclick={onRetry} class="px-6 font-bold rounded-xl">
                        {i18n.t('watch.retry')}
                    </Button>

                    {#if isMappingError}
                        <Button
                                variant="outline"
                                class="px-6 font-bold text-white transition-colors bg-white/5 border-white/20 hover:bg-white/10 rounded-xl"
                                onclick={onManageExtensions}
                        >
                            <PuzzleIcon class="w-4 h-4 mr-2" />
                            {i18n.t('content.extension_manager.manage_extensions_title')}
                        </Button>
                    {/if}
                </div>
            </div>

        {:else if isLoadingPlay}
            <div class="flex flex-col items-center gap-4 animate-in fade-in duration-300">
                <Loader2 class="w-12 h-12 animate-spin text-primary" />
                <span class="text-sm font-bold tracking-widest uppercase text-white/70">
                    {i18n.t('watch.loading_stream')}
                </span>
            </div>

        {:else if !isLoadingMeta && noExtensions}
            <div class="flex flex-col items-center gap-5 p-8 text-center animate-in fade-in zoom-in-95 duration-300">
                <PuzzleIcon class="w-16 h-16 text-white/30" />
                <span class="text-xl font-bold text-white/90">{i18n.t('watch.no_extensions')}</span>
            </div>
        {/if}
    </div>
{/if}