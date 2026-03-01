<script lang="ts">
    import { contentApi } from '$lib/api/content/content';
    import type { TrackerMapping } from '$lib/api/content/types';
    import * as Dialog from '$lib/components/ui/dialog';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import * as Select from '$lib/components/ui/select';
    import { Trash2, Plus, Loader2, Pencil, X, Save } from 'lucide-svelte';
    import { toast } from "svelte-sonner"; // AGREGADO: Importación de sonner

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

    // Estados para controlar el CRUD
    let isEditing = $state(false);
    let formName = $state("");
    let formId = $state("");

    const availableTrackers = [
        { value: 'anilist', label: 'AniList', domain: 'anilist.co' },
        { value: 'myanimelist', label: 'MyAnimeList', domain: 'myanimelist.net' },
        { value: 'simkl', label: 'Simkl', domain: 'simkl.com' },
        { value: 'kitsu', label: 'Kitsu', domain: 'kitsu.io' },
        { value: 'anidb', label: 'AniDB', domain: 'anidb.net' },
        { value: 'trakt', label: 'Trakt', domain: 'trakt.tv' },
        { value: 'animeplanet', label: 'Anime-Planet', domain: 'anime-planet.com' },
        { value: 'imdb', label: 'IMDb', domain: 'imdb.com' },
        { value: 'tmdb', label: 'TMDB', domain: 'themoviedb.org' },
        { value: 'tvdb', label: 'TVDB', domain: 'thetvdb.com' },
    ];

    function getFavicon(trackerName: string) {
        const t = availableTrackers.find(x => x.value === trackerName.toLowerCase().replace('trakttvslug', 'trakt'));
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
                await contentApi.updateTrackerMapping(cid, {
                    trackerName: formName,
                    trackerId: formId
                });
                toast.success(`Updated ${formName} mapping`); // ÉXITO AL ACTUALIZAR
            } else {
                await contentApi.addTrackerMapping(cid, {
                    cid,
                    trackerName: formName,
                    trackerId: formId,
                    syncEnabled: false,
                    createdAt: 0,
                    updatedAt: 0
                });
                toast.success(`Added ${formName} mapping`); // ÉXITO AL AÑADIR
            }
            window.location.reload();
        } catch (error) {
            console.error("Failed to save tracker", error);
            toast.error(`Failed to save ${formName} mapping`); // ERROR AL GUARDAR
            isLoading = false;
        }
    }

    async function handleDelete(trackerName: string) {
        isLoading = true;
        try {
            await contentApi.deleteTrackerMapping(cid, trackerName);
            toast.success(`Deleted ${trackerName} mapping`); // ÉXITO AL ELIMINAR
            window.location.reload();
        } catch (error) {
            console.error("Failed to delete tracker", error);
            toast.error(`Failed to delete ${trackerName} mapping`); // ERROR AL ELIMINAR
            isLoading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[425px] bg-card border-border/40">
        <Dialog.Header>
            <Dialog.Title>Manage Trackers</Dialog.Title>
            <Dialog.Description>
                Add or edit external database mappings for this content.
            </Dialog.Description>
        </Dialog.Header>

        <div class="space-y-4 py-4">
            <div class="space-y-2 max-h-[250px] overflow-y-auto pr-2">
                {#each trackers as tracker}
                    <div class="flex items-center justify-between p-2.5 rounded-lg border border-border/50 bg-muted/10 transition-colors {isEditing && formName === tracker.trackerName ? 'border-primary/50 bg-primary/5' : ''}">
                        <div class="flex items-center gap-3 overflow-hidden">
                            <img src={getFavicon(tracker.trackerName)} alt={tracker.trackerName} class="w-6 h-6 rounded-sm bg-white" />

                            <div class="flex flex-col overflow-hidden">
                                <span class="font-semibold text-sm capitalize truncate">{tracker.trackerName}</span>
                                <span class="text-xs text-muted-foreground font-mono truncate">{tracker.trackerId}</span>
                            </div>
                        </div>
                        <div class="flex items-center gap-1 shrink-0">
                            <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-primary" onclick={() => startEdit(tracker)} disabled={isLoading}>
                                <Pencil class="h-4 w-4" />
                            </Button>
                            <Button variant="ghost" size="icon" class="text-destructive hover:text-destructive hover:bg-destructive/10 h-8 w-8" onclick={() => handleDelete(tracker.trackerName)} disabled={isLoading}>
                                <Trash2 class="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                {:else}
                    <p class="text-sm text-muted-foreground text-center py-4">No trackers configured yet.</p>
                {/each}
            </div>

            <div class="flex flex-col gap-2 pt-4 border-t border-border/40">
                <div class="flex items-center justify-between mb-1">
                    <span class="text-sm font-semibold">{isEditing ? 'Edit Tracker' : 'Add New Tracker'}</span>
                    {#if isEditing}
                        <Button variant="ghost" size="sm" class="h-6 text-xs px-2 text-muted-foreground" onclick={cancelEdit} disabled={isLoading}>
                            <X class="h-3 w-3 mr-1" /> Cancel
                        </Button>
                    {/if}
                </div>

                <div class="flex items-end gap-2">
                    <div class="space-y-2 flex-1">
                        <label class="text-xs font-medium text-muted-foreground">Provider</label>
                        {#if isEditing}
                            <div class="h-9 px-3 flex items-center bg-muted/30 border rounded-md text-sm capitalize opacity-70 cursor-not-allowed">
                                {formName}
                            </div>
                        {:else}
                            <Select.Root type="single" bind:value={formName}>
                                <Select.Trigger class="w-full h-9 text-sm">
                                    {availableTrackers.find(t => t.value === formName)?.label || "Select..."}
                                </Select.Trigger>
                                <Select.Content>
                                    {#each availableTrackers as t}
                                        <Select.Item value={t.value}>{t.label}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        {/if}
                    </div>
                    <div class="space-y-2 flex-1">
                        <label class="text-xs font-medium text-muted-foreground">ID / Slug</label>
                        <Input class="h-9 text-sm" placeholder="e.g. 12345" bind:value={formId} disabled={isLoading} />
                    </div>
                    <Button size="icon" variant={isEditing ? "default" : "secondary"} class="h-9 w-9 shrink-0" disabled={!formName || !formId || isLoading} onclick={handleSubmit}>
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