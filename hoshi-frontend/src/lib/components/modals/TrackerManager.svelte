<script lang="ts">
    import { contentApi } from '@/api/content/content';
    import type { TrackerMapping } from '@/api/content/types';
    import * as Dialog from '@/components/ui/dialog';
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import * as Select from '@/components/ui/select';
    import { Trash2, Plus, Loader2, Pencil, X, Save } from 'lucide-svelte';
    import { toast } from "svelte-sonner";
    import { i18n } from "@/i18n/index.svelte.js";

    let {
        open = $bindable(false),
        cid,
        trackers
    }: {
        open: boolean;
        cid: string;
        trackers: TrackerMapping[];
    } = $props();

    let isLoading = $state(false);
    let isEditing = $state(false);
    let formName = $state("");
    let formId = $state("");

    const availableTrackers = [
        { value: 'anilist', label: 'AniList', domain: 'anilist.co' },
        { value: 'myanimelist', label: 'MyAnimeList', domain: 'myanimelist.net' },
        { value: 'simkl', label: 'Simkl', domain: 'simkl.com' },
        { value: 'kitsu', label: 'Kitsu', domain: 'kitsu.io' },
        { value: 'trakt', label: 'Trakt', domain: 'trakt.tv' },
        { value: 'animeplanet', label: 'Anime-Planet', domain: 'anime-planet.com' },
        { value: 'imdb', label: 'IMDb', domain: 'imdb.com' },
        { value: 'tmdb', label: 'TMDB', domain: 'themoviedb.org' },
        { value: 'tvdb', label: 'TVDB', domain: 'thethetvdb.com' },
    ];

    function getFavicon(trackerName: string) {
        const t = availableTrackers.find(x => x.value === trackerName.toLowerCase());
        const domain = t ? t.domain : `${trackerName}.com`;
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }

    function startEdit(tracker: TrackerMapping) {
        isEditing = true;
        formName = tracker.trackerName;
        formId = tracker.trackerId;
    }

    function cancelEdit() {
        isEditing = false;
        formName = "";
        formId = "";
    }

    async function handleSubmit() {
        if (!formName || !formId) return;
        isLoading = true;
        try {
            if (isEditing) {
                await contentApi.updateTrackerMapping(cid, { trackerName: formName, trackerId: formId });
                toast.success(i18n.t('content.mapping_updated'));
            } else {
                await contentApi.addTrackerMapping(cid, {
                    cid, trackerName: formName, trackerId: formId,
                    syncEnabled: false, createdAt: 0, updatedAt: 0
                });
                toast.success(i18n.t('content.mapping_added'));
            }
            window.location.reload();
        } catch (error) {
            toast.error(i18n.t('errors.network'));
            isLoading = false;
        }
    }

    async function handleDelete(trackerName: string) {
        isLoading = true;
        try {
            await contentApi.deleteTrackerMapping(cid, trackerName);
            toast.success(i18n.t('content.mapping_deleted'));
            window.location.reload();
        } catch (error) {
            toast.error(i18n.t('errors.network'));
            isLoading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="fixed left-1/2 top-1/2 z-50 w-[95vw] max-w-[425px] -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-border/40 bg-card p-6 shadow-2xl focus:outline-none">

        <Dialog.Header class="space-y-2">
            <Dialog.Title class="text-xl font-bold">{i18n.t('content.manage_trackers')}</Dialog.Title>
            <Dialog.Description class="text-sm text-muted-foreground">
                {i18n.t('content.manage_trackers_desc')}
            </Dialog.Description>
        </Dialog.Header>

        <div class="mt-4 space-y-6">
            <!-- Lista de Trackers -->
            <div class="space-y-2 max-h-[250px] overflow-y-auto pr-1">
                {#each trackers as tracker}
                    <div class="flex items-center justify-between p-3 rounded-xl border border-border/50 bg-muted/10">
                        <div class="flex items-center gap-3 min-w-0">
                            <img src={getFavicon(tracker.trackerName)} alt="" class="w-5 h-5 rounded-sm shrink-0" />
                            <div class="flex flex-col min-w-0">
                                <span class="font-bold text-sm truncate capitalize">{tracker.trackerName}</span>
                                <span class="text-[10px] text-muted-foreground font-mono truncate">{tracker.trackerId}</span>
                            </div>
                        </div>
                        <div class="flex items-center gap-1 shrink-0 ml-2">
                            <Button variant="ghost" size="icon" class="h-8 w-8" onclick={() => startEdit(tracker)} disabled={isLoading}>
                                <Pencil class="h-4 w-4" />
                            </Button>
                            <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:bg-destructive/10" onclick={() => handleDelete(tracker.trackerName)} disabled={isLoading}>
                                <Trash2 class="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                {:else}
                    <p class="text-center py-4 text-xs text-muted-foreground">{i18n.t('content.no_trackers')}</p>
                {/each}
            </div>

            <!-- Formulario -->
            <div class="space-y-4 pt-4 border-t border-border/20">
                <div class="flex items-center justify-between">
                    <span class="text-[10px] font-black uppercase tracking-widest text-muted-foreground">
                        {isEditing ? i18n.t('content.edit_tracker') : i18n.t('content.add_tracker')}
                    </span>
                    {#if isEditing}
                        <button class="text-[10px] font-bold uppercase text-primary" onclick={cancelEdit}>
                            {i18n.t('content.cancel')}
                        </button>
                    {/if}
                </div>

                <div class="flex items-end gap-2 w-full">
                    <div class="flex-1 min-w-0 space-y-1">
                        <label class="text-[10px] font-bold text-muted-foreground uppercase ml-1" for="prov">{i18n.t('content.provider')}</label>
                        {#if isEditing}
                            <div class="h-9 px-3 flex items-center bg-muted/30 border border-border/50 rounded-lg text-sm font-semibold opacity-60">
                                {formName}
                            </div>
                        {:else}
                            <Select.Root type="single" bind:value={formName}>
                                <Select.Trigger id="prov" class="h-9 text-sm rounded-lg">
                                    <span class="truncate">{availableTrackers.find(t => t.value === formName)?.label || i18n.t('content.select')}</span>
                                </Select.Trigger>
                                <Select.Content>
                                    {#each availableTrackers as t}
                                        <Select.Item value={t.value}>{t.label}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        {/if}
                    </div>

                    <div class="flex-1 min-w-0 space-y-1">
                        <label class="text-[10px] font-bold text-muted-foreground uppercase ml-1" for="idsl">{i18n.t('content.id_slug')}</label>
                        <Input id="idsl" class="h-9 text-sm rounded-lg" placeholder="ID / Slug" bind:value={formId} disabled={isLoading} />
                    </div>

                    <Button
                            size="icon"
                            variant={isEditing ? "default" : "secondary"}
                            class="h-9 w-9 shrink-0 rounded-lg"
                            disabled={!formName || !formId || isLoading}
                            onclick={handleSubmit}
                    >
                        {#if isLoading}
                            <Loader2 class="h-4 w-4 animate-spin" />
                        {:else if isEditing}
                            <Save class="h-4 w-4" />
                        {:else}
                            <Plus class="h-4 w-4" />
                        {/if}
                    </Button>
                </div>
            </div>
        </div>
    </Dialog.Content>
</Dialog.Root>