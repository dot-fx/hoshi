<script lang="ts">
    import * as Dialog from "@/components/ui/dialog";
    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import { contentApi } from "@/api/content/content";
    import type { TrackerCandidate } from "@/api/content/types";
    import { Link, AlertCircle } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { i18n } from "@/i18n/index.svelte.js";
    import type { CoreError } from "@/api/client";

    let {
        open = $bindable(false),
        cid,
        candidates = []
    }: {
        open: boolean;
        cid: string;
        candidates: TrackerCandidate[];
    } = $props();

    let isLinking = $state(false);

    async function handleLink(candidate: TrackerCandidate) {
        isLinking = true;
        try {
            await contentApi.linkTracker(cid, {
                trackerName: candidate.trackerName,
                trackerId: candidate.trackerId
            });
            toast.success(i18n.t('content.link_success'));
            open = false;
            window.location.reload();
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            isLinking = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-md bg-card border-border/50">
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2">
                <AlertCircle class="w-5 h-5 text-primary" />
                {i18n.t('content.link_metadata')}
            </Dialog.Title>
            <Dialog.Description>
                {i18n.t('content.link_metadata_desc')}
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex flex-col gap-3 py-4 max-h-[60vh] overflow-y-auto pr-2 custom-scrollbar">
            {#each candidates.sort((a, b) => b.score - a.score) as cand}
                <button
                        class="flex items-start gap-3 w-full text-left p-2.5 rounded-xl border border-border/40 bg-muted/10 hover:bg-muted/30 hover:border-primary/50 transition-all group disabled:opacity-50"
                        onclick={() => handleLink(cand)}
                        disabled={isLinking}
                >
                    {#if cand.coverImage}
                        <img src={cand.coverImage} alt={cand.title} class="w-12 h-16 md:w-14 md:h-20 object-cover rounded-lg shadow-sm shrink-0" />
                    {:else}
                        <div class="w-12 h-16 md:w-14 md:h-20 bg-muted rounded-lg flex items-center justify-center shrink-0">
                            <Link class="w-4 h-4 text-muted-foreground" />
                        </div>
                    {/if}

                    <div class="flex flex-col justify-center min-w-0 py-1">
                        <span class="font-bold text-sm leading-tight line-clamp-2 group-hover:text-primary transition-colors">
                            {cand.title}
                        </span>
                        <div class="flex items-center gap-2 mt-1.5">
                            <span class="text-[10px] font-black uppercase tracking-widest text-muted-foreground bg-background/50 px-1.5 py-0.5 rounded border border-border/50">
                                {cand.trackerName}
                            </span>
                            <span class="text-xs font-medium text-muted-foreground">
                                {i18n.t('content.score')}: {Math.round(cand.score)}
                            </span>
                        </div>
                    </div>
                </button>
            {/each}
        </div>

        <Dialog.Footer class="sm:justify-start">
            <Button variant="ghost" class="w-full text-muted-foreground" onclick={() => open = false} disabled={isLinking}>
                {i18n.t('content.skip_now')}
            </Button>
        </Dialog.Footer>

        {#if isLinking}
            <div class="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center rounded-lg z-50">
                <div class="flex flex-col items-center gap-3">
                    <Spinner class="w-8 h-8 text-primary" />
                    <span class="text-sm font-medium">{i18n.t('content.linking')}</span>
                </div>
            </div>
        {/if}
    </Dialog.Content>
</Dialog.Root>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: hsl(var(--muted-foreground) / 0.3); border-radius: 4px; }
</style>