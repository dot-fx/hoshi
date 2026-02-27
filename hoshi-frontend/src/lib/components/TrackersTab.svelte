<script lang="ts">
    import { integrationsApi } from "@/api/tracker/tracker";
    import type { TrackerInfo } from "@/api/tracker/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";

    import { Loader2, RefreshCw, Trash2, Plus } from "lucide-svelte";
    import * as Card from "$lib/components/ui/card";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Badge } from "$lib/components/ui/badge";

    let trackers = $state<TrackerInfo[]>([]);
    let loading = $state(true);
    let syncing = $state(false);

    let showRemoveTrackerAlert = $state(false);
    let trackerToRemove = $state<string | null>(null);
    let removingTracker = $state(false);

    let showAddTrackerDialog = $state(false);
    let newTrackerName = $state("");
    let newTrackerDisplayName = $state("");
    let newTrackerToken = $state("");
    let addingTracker = $state(false);

    $effect(() => {
        loadTrackers();
    });

    async function loadTrackers() {
        loading = true;
        try {
            trackers = await integrationsApi.getAll() || [];
        } catch (error) {
            toast.error("Failed to load trackers");
        } finally {
            loading = false;
        }
    }

    async function handleSyncTrackers() {
        syncing = true;
        try {
            const res = await integrationsApi.sync();
            toast.success(`Sync complete: ${res.synced} items updated`);
        } catch (error) {
            toast.error("Failed to sync trackers");
        } finally {
            syncing = false;
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
            toast.success(`Tracker disconnected`);
            await loadTrackers();
        } catch (error) {
            toast.error(`Failed to disconnect tracker`);
        } finally {
            removingTracker = false;
            showRemoveTrackerAlert = false;
            trackerToRemove = null;
        }
    }

    function openAddTrackerDialog(name: string, displayName: string) {
        newTrackerName = name;
        newTrackerDisplayName = displayName;
        newTrackerToken = "";
        showAddTrackerDialog = true;
    }

    async function handleAddTracker(e: Event) {
        e.preventDefault();
        if (!newTrackerToken) {
            toast.error("Access token is required");
            return;
        }

        addingTracker = true;
        try {
            await integrationsApi.add({
                trackerName: newTrackerName,
                accessToken: newTrackerToken,
            } as any);
            toast.success(`${newTrackerDisplayName} connected successfully`);
            showAddTrackerDialog = false;
            await loadTrackers();
        } catch (error: any) {
            toast.error(error?.message || `Failed to connect ${newTrackerDisplayName}`);
        } finally {
            addingTracker = false;
        }
    }
</script>

<Card.Root class="shadow-sm border-border/50">
    <Card.Header class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4">
        <div class="space-y-1">
            <Card.Title>Connected Trackers</Card.Title>
            <Card.Description>Manage your connections to external tracking services.</Card.Description>
        </div>
        <Button
                variant="outline"
                size="sm"
                onclick={handleSyncTrackers}
                disabled={syncing || trackers.filter(t => t.connected).length === 0 || loading}
                class="w-full sm:w-auto flex-shrink-0 shadow-sm"
        >
            <RefreshCw class="mr-2 h-4 w-4 {syncing ? 'animate-spin' : ''}" />
            Sync Now
        </Button>
    </Card.Header>
    <Card.Content>
        {#if loading}
            <div in:fade class="flex justify-center py-12 text-muted-foreground">
                <Loader2 class="h-8 w-8 animate-spin text-primary" />
            </div>
        {:else}
            <div in:fade class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {#each trackers as tracker}
                    <div class="flex items-center justify-between p-4 rounded-xl border border-border/60 shadow-sm transition-colors {tracker.connected ? 'bg-muted/10' : 'bg-background'}">

                        <div class="flex items-center gap-4 min-w-0 flex-1 mr-4">
                            {#if tracker.iconUrl}
                                <img src={tracker.iconUrl} alt={tracker.displayName} class="h-10 w-10 rounded-full object-cover flex-shrink-0 border border-border/50" />
                            {:else}
                                <div class="h-10 w-10 rounded-full bg-primary/10 flex items-center justify-center text-primary font-bold uppercase flex-shrink-0">
                                    {tracker.displayName.slice(0, 2)}
                                </div>
                            {/if}

                            <div class="min-w-0">
                                <p class="font-medium capitalize text-foreground truncate" title={tracker.displayName}>
                                    {tracker.displayName}
                                </p>
                                {#if tracker.connected}
                                    <Badge variant="default" class="text-[10px] h-4 mt-1">Connected</Badge>
                                {:else}
                                    <span class="text-xs text-muted-foreground block mt-1">Not connected</span>
                                {/if}
                            </div>
                        </div>

                        <div class="flex-shrink-0">
                            {#if tracker.connected}
                                <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-destructive hover:bg-destructive/10" onclick={() => confirmRemoveTracker(tracker.name)}>
                                    <Trash2 class="h-4 w-4" />
                                </Button>
                            {:else}
                                <Button variant="outline" size="sm" class="shadow-sm" onclick={() => openAddTrackerDialog(tracker.name, tracker.displayName)}>
                                    <Plus class="h-4 w-4 mr-1 md:hidden lg:inline-block" />
                                    <span>Connect</span>
                                </Button>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </Card.Content>
</Card.Root>

<AlertDialog.Root bind:open={showRemoveTrackerAlert}>
    <AlertDialog.Content class="w-[95vw] sm:w-full">
        <AlertDialog.Header>
            <AlertDialog.Title>Disconnect tracker?</AlertDialog.Title>
            <AlertDialog.Description>
                You will no longer be able to sync your list with this tracker. You can reconnect it at any time.
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="flex-col sm:flex-row gap-2 sm:gap-0 mt-4">
            <AlertDialog.Cancel class="w-full sm:w-auto mt-2 sm:mt-0">Cancel</AlertDialog.Cancel>
            <AlertDialog.Action class="w-full sm:w-auto bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm" onclick={handleRemoveTracker}>
                {#if removingTracker}
                    <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                {/if}
                Disconnect
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>

<Dialog.Root bind:open={showAddTrackerDialog}>
    <Dialog.Content class="w-[95vw] sm:max-w-md">
        <Dialog.Header>
            <Dialog.Title class="capitalize">Connect {newTrackerDisplayName}</Dialog.Title>
            <Dialog.Description>
                Enter your Personal Access Token to connect your {newTrackerDisplayName} account.
            </Dialog.Description>
        </Dialog.Header>
        <form onsubmit={handleAddTracker} class="space-y-4 py-2">
            <div class="space-y-2">
                <Label for="token">Access Token</Label>
                <Input id="token" type="password" placeholder="Paste your token here..." bind:value={newTrackerToken} required class="w-full" />
            </div>
            <Dialog.Footer class="flex-col sm:flex-row gap-2 sm:gap-2 pt-4">
                <div class="w-full sm:w-auto order-last sm:order-first mt-2 sm:mt-0">
                    <Dialog.Close class="w-full">
                        <Button type="button" variant="outline" class="w-full">Cancel</Button>
                    </Dialog.Close>
                </div>
                <Button type="submit" disabled={addingTracker} class="w-full sm:w-auto shadow-sm">
                    {#if addingTracker}
                        <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                    {/if}
                    Connect Tracker
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>