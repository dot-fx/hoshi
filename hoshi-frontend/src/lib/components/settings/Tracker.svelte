<script lang="ts">
    import { onMount } from "svelte";
    import { integrationsApi } from "@/api/tracker/tracker";
    import type { TrackerInfo } from "@/api/tracker/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from '$lib/i18n/index.svelte';
    import { listen } from "@tauri-apps/api/event";
    import { open } from "@tauri-apps/plugin-shell";
    import { Trash2, Plus, AlertTriangle, ExternalLink, User, Settings2 } from "lucide-svelte";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Badge } from "$lib/components/ui/badge";
    import { Switch } from "$lib/components/ui/switch";
    import { Spinner } from "$lib/components/ui/spinner";

    let trackers = $state<TrackerInfo[]>([]);
    let loading = $state(true);

    let showRemoveTrackerAlert = $state(false);
    let trackerToRemove = $state<string | null>(null);
    let removingTracker = $state(false);

    let showAddTrackerDialog = $state(false);
    let newTrackerName = $state("");
    let newTrackerDisplayName = $state("");
    let newTrackerToken = $state("");

    let newTrackerUsername = $state("");
    let newTrackerPassword = $state("");
    let newTrackerAuth = $state<any>(null);
    let addingTracker = $state(false);

    function generateVerifier() {
        const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
        let res = '';
        const randomValues = new Uint8Array(64);
        window.crypto.getRandomValues(randomValues);
        for (let i = 0; i < 64; i++) {
            res += charset[randomValues[i] % charset.length];
        }
        return res;
    }

    onMount(() => {
        loadTrackers();

        let unlistenAuth: (() => void) | undefined;

        const setupAuthListener = async () => {
            unlistenAuth = await listen("auth-callback", async (event) => {
                const url = event.payload as string;
                if (url.startsWith("hoshi://auth")) {
                    const code = new URL(url).searchParams.get("code");
                    if (code) await finalizeAuth(code);
                }
            });
        };

        setupAuthListener();

        return () => {
            if (unlistenAuth) unlistenAuth();
        };
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
            if (index !== -1) trackers[index].syncEnabled = enabled;
            toast.success(i18n.t('settings.changes_updated'));
        } catch (error) {
            toast.error(i18n.t('errors.network'));
            await loadTrackers();
        }
    }

    function openAddTrackerDialog(tracker: TrackerInfo) {
        newTrackerName = tracker.name;
        newTrackerDisplayName = tracker.displayName;
        newTrackerAuth = tracker.auth;
        newTrackerToken = "";
        newTrackerUsername = "";
        newTrackerPassword = "";
        showAddTrackerDialog = true;
    }

    async function handleAuthStart() {
        if (newTrackerAuth?.oauthFlow === 'pkce') {
            const verifier = generateVerifier();
            localStorage.setItem("mal_verifier", verifier);

            const params = new URLSearchParams({
                client_id: newTrackerAuth.clientId,
                response_type: 'code',
                code_challenge: verifier,
                code_challenge_method: 'plain',
                redirect_uri: 'hoshi://auth'
            });

            const url = `${newTrackerAuth.authUrl}?${params.toString()}`;
            await open(url);
        } else {
            const url = `${newTrackerAuth.authUrl}?client_id=${newTrackerAuth.clientId}&response_type=token`;
            await open(url);
        }
    }

    async function finalizeAuth(code: string) {
        addingTracker = true;
        try {
            await integrationsApi.add({
                trackerName: newTrackerName,
                accessToken: code,
                codeVerifier: localStorage.getItem("mal_verifier") || undefined
            });
            toast.success(i18n.t('settings.trackers_section.connected_successfully', {name: newTrackerDisplayName}));
            showAddTrackerDialog = false;
            await loadTrackers();
        } catch (error: any) {
            toast.error(typeof error === 'string' ? error : i18n.t('errors.auth_error', { defaultValue: "Authentication error" }));
        } finally {
            addingTracker = false;
        }
    }

    async function handleAddTracker(e: Event) {
        e.preventDefault();
        let payload: any = { trackerName: newTrackerName };

        if (newTrackerAuth?.oauthFlow === 'password') {
            if (!newTrackerUsername || !newTrackerPassword) return;
            payload.username = newTrackerUsername;
            payload.password = newTrackerPassword;
        } else {
            if (!newTrackerToken) return;
            payload.accessToken = newTrackerToken;
        }

        addingTracker = true;
        try {
            await integrationsApi.add(payload);
            toast.success(i18n.t('settings.trackers_section.connected_successfully', {name: newTrackerDisplayName}));
            showAddTrackerDialog = false;
            await loadTrackers();
        } catch (error: any) {
            toast.error(typeof error === 'string' ? error : i18n.t('errors.connect_error', { defaultValue: "Failed to connect" }));
        } finally {
            addingTracker = false;
        }
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
</script>

<div class="space-y-16 w-full">
    <section>
        <div class="mb-2">
            <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.trackers_section.trackers_title')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.trackers_section.trackers_desc')}</p>
        </div>

        {#if loading}
            <div in:fade class="flex justify-center py-12 text-muted-foreground border-y border-border/40 mt-6">
                <Spinner class="h-8 w-8 text-primary" />
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
                                        <Badge variant="default" class="text-[10px] h-4">{i18n.t('settings.trackers_section.connected')}</Badge>
                                    {/if}
                                </div>
                                {#if tracker.connected}
                                    <div class="flex flex-wrap items-center gap-x-3 gap-y-1 pt-0.5 text-xs text-muted-foreground">
                                        {#if tracker.trackerUserId}
                                            <span class="flex items-center gap-1">
                                                <User class="h-3.5 w-3.5" /> {tracker.trackerUserId}
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
                                                    <Settings2 class="h-3.5 w-3.5" /> {i18n.t('settings.trackers_section.auto_sync')}
                                                </Label>
                                            </div>
                                        {/if}
                                    </div>
                                {:else}
                                    <p class="text-sm text-muted-foreground mt-0.5">{i18n.t('settings.trackers_section.not_connected')}</p>
                                {/if}
                            </div>
                        </div>
                        <div class="shrink-0 flex items-center justify-end">
                            {#if tracker.connected}
                                <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-destructive rounded-xl h-11 w-11" onclick={() => {trackerToRemove = tracker.name; showRemoveTrackerAlert = true;}}>
                                    <Trash2 class="h-5 w-5" />
                                </Button>
                            {:else}
                                <Button variant="outline" class="rounded-xl h-11 font-bold shadow-sm" onclick={() => openAddTrackerDialog(tracker)}>
                                    <Plus class="h-4 w-4 mr-2" />
                                    <span>{i18n.t('settings.trackers_section.connect')}</span>
                                </Button>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</div>

<Dialog.Root bind:open={showAddTrackerDialog}>
    <Dialog.Content class="sm:max-w-md sm:rounded-2xl">
        <Dialog.Header>
            <Dialog.Title class="capitalize text-xl font-bold">{i18n.t('settings.trackers_section.connect')} {newTrackerDisplayName}</Dialog.Title>
            <Dialog.Description class="text-base">
                {i18n.t('settings.trackers_section.connect_tracker_desc', { name: newTrackerDisplayName })}
            </Dialog.Description>
        </Dialog.Header>

        <div class="py-4">
            {#if newTrackerAuth?.oauthFlow === 'pkce'}
                <div class="flex flex-col items-center space-y-4">
                    <p class="text-sm text-center text-muted-foreground">
                        {i18n.t('settings.trackers_section.pkce_redirect_notice', { defaultValue: "You will be redirected to MyAnimeList to authorize the application. The application will connect automatically upon completion." })}
                    </p>
                    <Button onclick={handleAuthStart} disabled={addingTracker} class="w-full rounded-xl h-11 font-bold">
                        {#if addingTracker}<Spinner class="mr-2 h-4 w-4" />{/if}
                        {i18n.t('settings.trackers_section.login_to_service', { name: newTrackerDisplayName, defaultValue: `Log in to ${newTrackerDisplayName}` })}
                    </Button>
                </div>
            {:else if newTrackerAuth?.oauthFlow === 'password'}
                <form onsubmit={handleAddTracker} class="space-y-4">
                    <div class="space-y-2">
                        <Label for="username" class="text-base font-bold">{i18n.t('settings.trackers_section.email_or_username')}</Label>
                        <Input id="username" type="text" placeholder="ejemplo@correo.com" bind:value={newTrackerUsername} required class="rounded-xl h-11 w-full" />
                    </div>
                    <div class="space-y-2">
                        <Label for="password" class="text-base font-bold">{i18n.t('settings.account_section.new_password')}</Label>
                        <Input id="password" type="password" placeholder="••••••••" bind:value={newTrackerPassword} required class="rounded-xl h-11 w-full" />
                    </div>
                    <Button type="submit" disabled={addingTracker} class="w-full rounded-xl h-11 font-bold mt-4">
                        {#if addingTracker}<Spinner class="mr-2 h-4 w-4" />{/if}
                        {i18n.t('settings.trackers_section.connect_tracker')}
                    </Button>
                </form>
            {:else}
                <form onsubmit={handleAddTracker} class="space-y-4">
                    <div class="space-y-2">
                        <div class="flex items-center justify-between">
                            <Label for="token" class="text-base font-bold">{i18n.t('settings.trackers_section.token')}</Label>
                            <Button variant="link" size="sm" onclick={handleAuthStart} class="text-sm font-bold text-primary p-0 h-auto">
                                {i18n.t('settings.trackers_section.get_token', { name: newTrackerDisplayName })} <ExternalLink class="h-3.5 w-3.5 ml-1" />
                            </Button>
                        </div>
                        <Input id="token" type="password" placeholder={i18n.t('settings.trackers_section.paste_token')} bind:value={newTrackerToken} required class="rounded-xl h-11 w-full" />
                    </div>
                    <Button type="submit" disabled={addingTracker} class="w-full rounded-xl h-11 font-bold mt-4">
                        {#if addingTracker}<Spinner class="mr-2 h-4 w-4" />{/if}
                        {i18n.t('settings.trackers_section.connect_tracker')}
                    </Button>
                </form>
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>

<AlertDialog.Root bind:open={showRemoveTrackerAlert}>
    <AlertDialog.Content class="border-destructive/20 sm:rounded-2xl">
        <AlertDialog.Header>
            <AlertDialog.Title class="text-destructive flex items-center gap-2 text-xl">
                <AlertTriangle class="h-6 w-6" /> {i18n.t('settings.trackers_section.disconnect_tracker')}
            </AlertDialog.Title>
            <AlertDialog.Description class="text-base">
                {i18n.t('settings.trackers_section.disconnect_tracker_desc')}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="mt-6">
            <AlertDialog.Cancel class="rounded-xl font-bold">{i18n.t('settings.general_section.cancel', { defaultValue: 'Cancel' })}</AlertDialog.Cancel>
            <AlertDialog.Action class="bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded-xl font-bold" onclick={handleRemoveTracker}>
                {#if removingTracker}<Spinner class="h-4 w-4 mr-2" />{/if} {i18n.t('settings.trackers_section.disconnect')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>