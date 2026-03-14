<script lang="ts">
    import { integrationsApi } from "@/api/tracker/tracker";
    import type { TrackerInfo } from "@/api/tracker/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from '$lib/i18n/index.svelte';

    import { Loader2, RefreshCw, Trash2, Plus, AlertTriangle, ExternalLink, User, Tags, Settings2 } from "lucide-svelte";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Badge } from "$lib/components/ui/badge";
    import { Switch } from "$lib/components/ui/switch";

    let trackers = $state<TrackerInfo[]>([]);
    let loading = $state(true);

    let showRemoveTrackerAlert = $state(false);
    let trackerToRemove = $state<string | null>(null);
    let removingTracker = $state(false);

    let showAddTrackerDialog = $state(false);
    let newTrackerName = $state("");
    let newTrackerDisplayName = $state("");
    let newTrackerToken = $state("");
    let newTrackerAuth = $state<any>(null);
    let addingTracker = $state(false);

    $effect(() => {
        loadTrackers();
    });

    async function loadTrackers() {
        loading = true;
        try {
            trackers = await integrationsApi.getAll() || [];
        } catch (error) {
            toast.error(i18n.t('errors.network'));
        } finally {
            loading = false;
        }
    }

    async function handleToggleSync(trackerName: string, enabled: boolean) {
        try {
            await integrationsApi.setSyncEnabled(trackerName, enabled);

            const index = trackers.findIndex(t => t.name === trackerName);
            if (index !== -1) {
                trackers[index].syncEnabled = enabled;
            }
            toast.success(i18n.t('settings.changes_updated'));
        } catch (error) {
            toast.error(i18n.t('errors.network'));
            await loadTrackers();
        }
    }

    function confirmRemoveTracker(trackerName: string) {
        trackerToRemove = trackerName;
        showRemoveTrackerAlert = true;
    }

    async function handleRemoveTracker() {
        if (!trackerToRemove) return;
        removingTracker = true;
        try {
            await integrationsApi.remove(trackerToRemove);
            toast.success(i18n.t('settings.changes_updated'));
            await loadTrackers();
        } catch (error) {
            toast.error(i18n.t('errors.network'));
        } finally {
            removingTracker = false;
            showRemoveTrackerAlert = false;
            trackerToRemove = null;
        }
    }

    function openAddTrackerDialog(tracker: TrackerInfo) {
        newTrackerName = tracker.name;
        newTrackerDisplayName = tracker.displayName;
        newTrackerAuth = tracker.auth;
        newTrackerToken = "";
        showAddTrackerDialog = true;
    }

    async function handleAddTracker(e: Event) {
        e.preventDefault();
        if (!newTrackerToken) {
            toast.error(i18n.t('settings.token_required'));
            return;
        }

        addingTracker = true;
        try {
            await integrationsApi.add({
                trackerName: newTrackerName,
                accessToken: newTrackerToken,
            } as any);
            toast.success(i18n.t('settings.connected_successfully', {name: newTrackerDisplayName}));
            showAddTrackerDialog = false;
            await loadTrackers();
        } catch (error: any) {
            toast.error(error?.message);
        } finally {
            addingTracker = false;
        }
    }
</script>

<div class="space-y-16 w-full">
    <section>
        <div class="mb-2">
            <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.trackers_title')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.trackers_desc')}</p>
        </div>

        {#if loading}
            <div in:fade class="flex justify-center py-12 text-muted-foreground border-y border-border/40 mt-6">
                <Loader2 class="h-8 w-8 animate-spin text-primary" />
            </div>
        {:else}
            <div in:fade class="mt-6 border-t border-border/40">
                {#each trackers as tracker}
                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">

                        <div class="flex items-center gap-4 pr-4">
                            <Avatar.Root class="h-12 w-12 border border-border/50 shadow-sm shrink-0">
                                {#if tracker.iconUrl}
                                    <Avatar.Image src={tracker.iconUrl} alt={tracker.displayName} class="object-cover" />
                                {/if}
                                <Avatar.Fallback class="bg-primary/10 text-primary font-bold uppercase">
                                    {tracker.displayName.slice(0, 2)}
                                </Avatar.Fallback>
                            </Avatar.Root>

                            <div class="space-y-1">
                                <div class="flex items-center gap-2">
                                    <Label class="text-base font-bold capitalize text-foreground">{tracker.displayName}</Label>
                                    {#if tracker.connected}
                                        <Badge variant="default" class="text-[10px] h-4">{i18n.t('settings.connected')}</Badge>
                                    {/if}
                                </div>

                                {#if tracker.connected}
                                    <div class="flex flex-wrap items-center gap-x-3 gap-y-1 pt-0.5 text-xs text-muted-foreground">
                                        {#if tracker.trackerUserId}
                                            <span class="flex items-center gap-1" title={i18n.t('settings.userId')}>
                                                <User class="h-3.5 w-3.5" />
                                                {tracker.trackerUserId}
                                            </span>
                                        {/if}

                                        {#if tracker.supportedTypes?.length}
                                            <span class="flex items-center gap-1" title={i18n.t('settings.supported_types')}>
                                                <Tags class="h-3.5 w-3.5" />
                                                <span class="capitalize">{tracker.supportedTypes.join(', ')}</span>
                                            </span>
                                        {/if}

                                        {#if tracker.syncEnabled !== null}
                                            <div class="flex items-center gap-1.5 ml-1">
                                                <Switch
                                                        id={`sync-${tracker.name}`}
                                                        checked={tracker.syncEnabled}
                                                        onCheckedChange={(v) => handleToggleSync(tracker.name, v)}
                                                        class="scale-75 origin-left"
                                                />
                                                <Label for={`sync-${tracker.name}`} class="text-xs text-muted-foreground cursor-pointer flex items-center gap-1">
                                                    <Settings2 class="h-3.5 w-3.5" />
                                                    {i18n.t('settings.auto_sync')}
                                                </Label>
                                            </div>
                                        {/if}
                                    </div>
                                {:else}
                                    <p class="text-sm text-muted-foreground mt-0.5">{i18n.t('settings.not_connected')}</p>
                                {/if}
                            </div>
                        </div>

                        <div class="shrink-0 flex items-center justify-end">
                            {#if tracker.connected}
                                <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-xl h-11 w-11 transition-colors" onclick={() => confirmRemoveTracker(tracker.name)}>
                                    <Trash2 class="h-5 w-5" />
                                </Button>
                            {:else}
                                <Button variant="outline" class="rounded-xl h-11 font-bold shadow-sm" onclick={() => openAddTrackerDialog(tracker)}>
                                    <Plus class="h-4 w-4 mr-2" />
                                    <span>{i18n.t('settings.connect')}</span>
                                </Button>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</div>

<AlertDialog.Root bind:open={showRemoveTrackerAlert}>
    <AlertDialog.Content class="border-destructive/20 sm:rounded-2xl">
        <AlertDialog.Header>
            <AlertDialog.Title class="text-destructive flex items-center gap-2 text-xl">
                <AlertTriangle class="h-6 w-6" /> {i18n.t('settings.disconnect_tracker')}
            </AlertDialog.Title>
            <AlertDialog.Description class="text-base">
                {i18n.t('settings.disconnect_tracker_desc')}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="flex-col sm:flex-row gap-3 sm:gap-2 mt-6">
            <AlertDialog.Cancel class="w-full sm:w-auto rounded-xl font-bold">{i18n.t('settings.cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action class="w-full sm:w-auto bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm rounded-xl font-bold" onclick={handleRemoveTracker}>
                {#if removingTracker}
                    <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                {/if}
                {i18n.t('settings.disconnect')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>

<Dialog.Root bind:open={showAddTrackerDialog}>
    <Dialog.Content class="sm:max-w-md sm:rounded-2xl">
        <Dialog.Header>
            <Dialog.Title class="capitalize text-xl font-bold">{i18n.t('settings.connect')} {newTrackerDisplayName}</Dialog.Title>
            <Dialog.Description class="text-base">
                {i18n.t('settings.connect_tracker_desc', { name: newTrackerDisplayName })}
            </Dialog.Description>
        </Dialog.Header>
        <form onsubmit={handleAddTracker} class="space-y-6 py-2">
            <div class="space-y-2">
                <div class="flex items-center justify-between">
                    <Label for="token" class="text-base font-bold">{i18n.t('settings.token')}</Label>

                    {#if newTrackerAuth?.oauthFlow === 'implicit'}
                        <a
                                href="{newTrackerAuth.authUrl}?client_id={newTrackerAuth.clientId}&response_type=token"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-sm font-bold text-primary hover:underline flex items-center gap-1.5 transition-all"
                        >
                            {i18n.t('settings.get_token', { name: newTrackerDisplayName })} <ExternalLink class="h-3.5 w-3.5" />
                        </a>
                    {/if}
                </div>
                <Input id="token" type="password" placeholder={i18n.t('settings.paste_token')} bind:value={newTrackerToken} required class="rounded-xl h-11 w-full" />
            </div>
            <Dialog.Footer class="flex-col sm:flex-row gap-3 sm:gap-2 pt-4">
                <div class="w-full sm:w-auto order-last sm:order-first mt-2 sm:mt-0">
                    <Dialog.Close class="w-full">
                        <Button type="button" variant="outline" class="w-full rounded-xl h-11 font-bold">{i18n.t('settings.cancel')}</Button>
                    </Dialog.Close>
                </div>
                <Button type="submit" disabled={addingTracker} class="w-full sm:w-auto shadow-sm rounded-xl h-11 font-bold">
                    {#if addingTracker}
                        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                    {/if}
                    {i18n.t('settings.connect_tracker')}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>