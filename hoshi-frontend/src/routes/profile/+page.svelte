<script lang="ts">
    import { usersApi } from "@/api/users/users";
    import type { UserPrivate } from "@/api/users/types";
    import { toast } from "svelte-sonner";
    import TrackersTab from "$lib/components/TrackersTab.svelte";
    import { i18n } from '$lib/i18n/index.svelte';
    import * as Select from "$lib/components/ui/select";
    import { Loader2, User, Shield, Link2, Trash2, Save, AlertTriangle, Upload, X, Camera, Languages, Settings } from "lucide-svelte";
    import { fade } from "svelte/transition";

    import * as Tabs from "$lib/components/ui/tabs";
    import * as Card from "$lib/components/ui/card";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";

    let loading = $state(true);
    let user = $state<UserPrivate | null>(null);

    // General Settings State
    let username = $state("");
    let savingProfile = $state(false);

    // Avatar State
    let previewAvatarUrl = $state<string | null>(null);
    let selectedAvatarFile = $state<File | null>(null);
    let avatarRemoved = $state(false);
    let fileInput: HTMLInputElement | undefined = $state();

    // Security State
    let currentPassword = $state("");
    let newPassword = $state("");
    let confirmPassword = $state("");
    let savingPassword = $state(false);

    // Delete Account State
    let showDeleteAlert = $state(false);
    let deletingAccount = $state(false);
    let deletePassword = $state("");

    $effect(() => {
        loadData();
    });

    async function loadData() {
        loading = true;
        try {
            const userRes = await usersApi.getMe();
            user = userRes;
            username = user.username;
            previewAvatarUrl = user.avatar || null;
            // Reset avatar states on load
            selectedAvatarFile = null;
            avatarRemoved = false;
        } catch (error) {
            toast.error(i18n.t('failed_load_profile'));
        } finally {
            loading = false;
        }
    }

    // --- AVATAR HANDLING ---
    function handleFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            const file = target.files[0];
            selectedAvatarFile = file;
            previewAvatarUrl = URL.createObjectURL(file);
            avatarRemoved = false;
        }
    }

    function handleRemoveAvatar() {
        selectedAvatarFile = null;
        previewAvatarUrl = null;
        avatarRemoved = true;
        if (fileInput) fileInput.value = ""; // Reset input
    }

    // --- PROFILE SAVING ---
    async function handleUpdateProfile(e: Event) {
        e.preventDefault();
        savingProfile = true;
        try {
            if (username !== user?.username) {
                await usersApi.updateMe({ username });
            }

            if (selectedAvatarFile) {
                await usersApi.uploadAvatar(selectedAvatarFile);
            } else if (avatarRemoved && user?.avatar) {
                await usersApi.deleteAvatar();
            }

            toast.success(i18n.t('profile_updated'));
            await loadData();
        } catch (error: any) {
            toast.error(error?.message || i18n.t('failed_update_profile'));
        } finally {
            savingProfile = false;
        }
    }

    async function handleChangePassword(e: Event) {
        e.preventDefault();
        if (newPassword !== confirmPassword) { toast.error(i18n.t('passwords_not_match')); return; }
        savingPassword = true;
        try {
            await usersApi.changePassword({ currentPassword, newPassword });
            toast.success(i18n.t('password_updated'));
            currentPassword = "";
            newPassword = "";
            confirmPassword = "";
        } catch (error: any) {
            toast.error(error?.message || i18n.t('failed_change_password'));
        } finally {
            savingPassword = false;
        }
    }

    async function handleDeleteAccount() {
        if (user?.hasPassword && !deletePassword) { toast.error(i18n.t('password_required_delete')); return; }

        deletingAccount = true;
        try {
            await usersApi.deleteMe({ password: deletePassword });
            toast.success(i18n.t('account_deleted'));
            showDeleteAlert = false;
            window.location.href = "/";
        } catch (error: any) {
            toast.error(error?.message || i18n.t('failed_delete_account'));
            showDeleteAlert = false;
        } finally {
            deletingAccount = false;
        }
    }
</script>

<svelte:head>
    <title>{i18n.t('profile')}</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-10 pt-6 md:pt-8 px-4 md:px-6 lg:px-8 xl:px-10 w-full max-w-[2400px] mx-auto space-y-6 md:space-y-8">

    <header class="flex flex-col md:flex-row md:items-center justify-between gap-5 md:gap-6 border-b border-border/40 pb-6 w-full">
        <div class="space-y-1">
            <h1 class="text-3xl md:text-4xl font-black tracking-tight flex items-center gap-3">
                <Settings class="h-8 w-8 md:h-10 md:w-10 text-primary" />
                {i18n.t('profile_settings')}
            </h1>
            <p class="text-sm md:text-base text-muted-foreground font-medium opacity-80">
                {i18n.t('profile_settings_desc')}
            </p>
        </div>
    </header>

    <section class="space-y-6 w-full">
        {#if loading}
            <div in:fade class="h-[50vh] flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Loader2 class="h-10 w-10 animate-spin text-primary" />
                <p class="text-sm font-bold animate-pulse">{i18n.t('loading_profile')}</p>
            </div>
        {:else if user}
            <div in:fade class="w-full">

                <Tabs.Root value="general" class="flex flex-col md:flex-row gap-8 lg:gap-12 w-full items-start">

                    <Tabs.List class="flex flex-row md:flex-col justify-start bg-transparent h-auto p-0 gap-2 w-full md:w-56 shrink-0 overflow-x-auto hide-scrollbar md:pr-4">
                        <Tabs.Trigger value="general" class="relative px-4 py-3 md:py-3.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-transparent data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/20 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <User class="h-4 w-4 md:h-5 md:w-5" /> {i18n.t('general')}
                        </Tabs.Trigger>

                        <Tabs.Trigger value="security" class="relative px-4 py-3 md:py-3.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-transparent data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/20 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Shield class="h-4 w-4 md:h-5 md:w-5" /> {i18n.t('security')}
                        </Tabs.Trigger>

                        <Tabs.Trigger value="integrations" class="relative px-4 py-3 md:py-3.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-transparent data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/20 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Link2 class="h-4 w-4 md:h-5 md:w-5" /> {i18n.t('trackers')}
                        </Tabs.Trigger>
                    </Tabs.List>

                    <!-- Limitamos el ancho de las tarjetas, pero dejamos que fluyan a la izquierda naturalmente -->
                    <div class="flex-1 min-w-0 w-full max-w-5xl">

                        <Tabs.Content value="general" class="space-y-6 focus-visible:outline-none focus-visible:ring-0 mt-0 w-full">
                            <Card.Root class="shadow-sm border-border/50 rounded-xl w-full">
                                <Card.Header class="pb-4">
                                    <Card.Title class="flex items-center gap-2 text-lg md:text-xl">
                                        <Languages class="h-5 w-5 text-primary" />
                                        {i18n.t('language')}
                                    </Card.Title>
                                    <Card.Description>{i18n.t('select_language')}</Card.Description>
                                </Card.Header>
                                <Card.Content>
                                    <div class="max-w-xs">
                                        <Select.Root
                                                type="single"
                                                value={i18n.locale}
                                                onValueChange={(v) => i18n.setLocale(v)}
                                        >
                                            <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                                                {i18n.locale === 'en' ? i18n.t('english') : i18n.t('spanish')}
                                            </Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value="en">{i18n.t('english')}</Select.Item>
                                                <Select.Item value="es">{i18n.t('spanish')}</Select.Item>
                                            </Select.Content>
                                        </Select.Root>
                                    </div>
                                </Card.Content>
                            </Card.Root>

                            <Card.Root class="shadow-sm border-border/50 rounded-xl w-full">
                                <Card.Header class="pb-4">
                                    <Card.Title class="text-lg md:text-xl">{i18n.t('public_profile')}</Card.Title>
                                    <Card.Description>{i18n.t('public_profile_desc')}</Card.Description>
                                </Card.Header>
                                <Card.Content>
                                    <form onsubmit={handleUpdateProfile} class="space-y-8 w-full">
                                        <div class="flex flex-col sm:flex-row gap-8 items-center sm:items-start bg-muted/10 p-5 rounded-xl border border-border/40 shadow-sm w-full">
                                            <div class="relative group flex flex-col items-center gap-3 shrink-0">
                                                <Avatar.Root class="h-28 w-28 border-4 border-background shadow-md">
                                                    {#if previewAvatarUrl}
                                                        <Avatar.Image src={previewAvatarUrl} alt={username} class="object-cover" />
                                                    {/if}
                                                    <Avatar.Fallback class="bg-primary/10 text-primary text-3xl font-bold uppercase">
                                                        {username.slice(0, 2)}
                                                    </Avatar.Fallback>
                                                </Avatar.Root>

                                                <button
                                                        type="button"
                                                        onclick={() => fileInput.click()}
                                                        class="absolute top-0 left-0 h-28 w-28 rounded-full bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all cursor-pointer text-white backdrop-blur-sm border-4 border-transparent"
                                                >
                                                    <Camera class="h-8 w-8" />
                                                </button>

                                                <div class="flex flex-col gap-2 w-full mt-2">
                                                    <Button type="button" variant="outline" size="sm" class="w-full text-xs shadow-sm rounded-lg" onclick={() => fileInput.click()}>
                                                        <Upload class="mr-2 h-3 w-3" /> {i18n.t('upload')}
                                                    </Button>
                                                    {#if previewAvatarUrl}
                                                        <Button type="button" variant="ghost" size="sm" class="w-full text-xs text-destructive hover:text-destructive hover:bg-destructive/10 rounded-lg" onclick={handleRemoveAvatar}>
                                                            <X class="mr-2 h-3 w-3" /> {i18n.t('remove')}
                                                        </Button>
                                                    {/if}
                                                </div>

                                                <input
                                                        bind:this={fileInput}
                                                        type="file"
                                                        accept="image/png, image/jpeg, image/webp, image/gif"
                                                        class="hidden"
                                                        onchange={handleFileSelect}
                                                />
                                            </div>

                                            <div class="space-y-4 flex-1 w-full mt-2">
                                                <div class="space-y-2 max-w-lg">
                                                    <Label for="username" class="font-bold text-foreground/90">{i18n.t('username')}</Label>
                                                    <Input id="username" bind:value={username} class="bg-background rounded-xl h-11 focus-visible:ring-1 focus-visible:ring-primary/50 border-border/60" required />
                                                    <p class="text-xs font-medium text-muted-foreground mt-2">
                                                        {i18n.t('public_display_name')}
                                                        {#if selectedAvatarFile || avatarRemoved}
                                                            <br/><span class="text-primary mt-1 inline-block">{i18n.t('unsaved_avatar_changes')}</span>
                                                        {/if}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="flex justify-end pt-4 border-t border-border/40">
                                            <Button type="submit" disabled={savingProfile} class="w-full sm:w-auto shadow-sm rounded-xl px-6 h-11 font-bold">
                                                {#if savingProfile}
                                                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                                                    {i18n.t('saving_changes')}
                                                {:else}
                                                    <Save class="mr-2 h-4 w-4" />
                                                    {i18n.t('save_changes')}
                                                {/if}
                                            </Button>
                                        </div>
                                    </form>
                                </Card.Content>
                            </Card.Root>
                        </Tabs.Content>

                        <Tabs.Content value="security" class="space-y-6 focus-visible:outline-none focus-visible:ring-0 mt-0 w-full">
                            <Card.Root class="shadow-sm border-border/50 rounded-xl w-full">
                                <Card.Header class="pb-4">
                                    <Card.Title class="text-lg md:text-xl">{i18n.t('change_password')}</Card.Title>
                                    <Card.Description>{i18n.t('change_password_desc')}</Card.Description>
                                </Card.Header>
                                <Card.Content>
                                    <form onsubmit={handleChangePassword} class="space-y-6 w-full">
                                        {#if user.hasPassword}
                                            <div class="space-y-2 max-w-lg">
                                                <Label for="currentPassword" class="font-bold text-foreground/90">{i18n.t('current_password')}</Label>
                                                <Input id="currentPassword" type="password" bind:value={currentPassword} class="rounded-xl h-11 bg-background border-border/60" required />
                                            </div>
                                            <div class="h-[1px] w-full max-w-lg bg-border/50 my-4"></div>
                                        {/if}

                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 max-w-2xl w-full">
                                            <div class="space-y-2 w-full">
                                                <Label for="newPassword" class="font-bold text-foreground/90">{i18n.t('new_password')}</Label>
                                                <Input id="newPassword" type="password" bind:value={newPassword} class="rounded-xl h-11 bg-background border-border/60 w-full" required />
                                            </div>
                                            <div class="space-y-2 w-full">
                                                <Label for="confirmPassword" class="font-bold text-foreground/90">{i18n.t('confirm_password')}</Label>
                                                <Input id="confirmPassword" type="password" bind:value={confirmPassword} class="rounded-xl h-11 bg-background border-border/60 w-full" required />
                                            </div>
                                        </div>
                                        <div class="flex justify-end pt-6 border-t border-border/40 w-full">
                                            <Button type="submit" disabled={savingPassword} class="w-full sm:w-auto shadow-sm rounded-xl px-6 h-11 font-bold">
                                                {#if savingPassword}
                                                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                                                    {i18n.t('updating')}
                                                {:else}
                                                    <Shield class="mr-2 h-4 w-4" />
                                                    {i18n.t('update_password')}
                                                {/if}
                                            </Button>
                                        </div>
                                    </form>
                                </Card.Content>
                            </Card.Root>

                            <Card.Root class="border-destructive/30 bg-destructive/5 shadow-sm mt-8 rounded-xl w-full">
                                <Card.Header class="pb-4">
                                    <Card.Title class="text-destructive flex items-center gap-2 text-lg md:text-xl">
                                        <AlertTriangle class="h-5 w-5" /> {i18n.t('danger_zone')}
                                    </Card.Title>
                                    <Card.Description class="text-foreground/80 font-medium">
                                        {i18n.t('danger_zone_desc')}
                                    </Card.Description>
                                </Card.Header>
                                <Card.Content>
                                    <div class="flex flex-col sm:flex-row items-start sm:items-end gap-4 bg-background/50 p-4 rounded-xl border border-destructive/20 w-full max-w-3xl">
                                        {#if user.hasPassword}
                                            <div class="space-y-2 w-full sm:max-w-sm">
                                                <Label for="deletePassword" class="text-destructive font-bold">{i18n.t('enter_password_confirm')}</Label>
                                                <Input
                                                        id="deletePassword"
                                                        type="password"
                                                        bind:value={deletePassword}
                                                        class="border-destructive/30 focus-visible:ring-destructive/50 bg-background rounded-xl h-11 w-full"
                                                        placeholder="••••••••"
                                                />
                                            </div>
                                        {/if}
                                        <Button
                                                type="button"
                                                variant="destructive"
                                                class="w-full sm:w-auto mt-4 sm:mt-0 shadow-sm rounded-xl h-11 font-bold shrink-0"
                                                onclick={() => showDeleteAlert = true}
                                        >
                                            <Trash2 class="mr-2 h-4 w-4" />
                                            {i18n.t('delete_account')}
                                        </Button>
                                    </div>
                                </Card.Content>
                            </Card.Root>
                        </Tabs.Content>

                        <Tabs.Content value="integrations" class="focus-visible:outline-none focus-visible:ring-0 mt-0 w-full">
                            <svelte:boundary>
                                <div class="w-full">
                                    <TrackersTab />
                                </div>
                                {#snippet failed(error, reset)}
                                    <Card.Root class="border-destructive/30 bg-destructive/5 shadow-sm rounded-xl w-full">
                                        <Card.Header>
                                            <Card.Title class="text-destructive flex items-center gap-2 text-lg md:text-xl">
                                                <AlertTriangle class="h-5 w-5" /> {i18n.t('error_loading_trackers_title')}
                                            </Card.Title>
                                            <Card.Description>{i18n.t('error_loading_trackers_desc')}</Card.Description>
                                        </Card.Header>
                                        <Card.Content>
                                            <Button variant="outline" class="border-destructive/30 text-destructive hover:bg-destructive/10 rounded-xl font-bold" onclick={reset}>{i18n.t('try_again')}</Button>
                                        </Card.Content>
                                    </Card.Root>
                                {/snippet}
                            </svelte:boundary>
                        </Tabs.Content>
                    </div>
                </Tabs.Root>
            </div>
        {/if}
    </section>
</main>

<AlertDialog.Root bind:open={showDeleteAlert}>
    <AlertDialog.Content class="border-destructive/20 sm:rounded-2xl">
        <AlertDialog.Header>
            <AlertDialog.Title class="text-destructive flex items-center gap-2 text-xl">
                <AlertTriangle class="h-6 w-6" /> {i18n.t('are_you_sure')}
            </AlertDialog.Title>
            <AlertDialog.Description class="text-base">
                {i18n.t('delete_account_warning')}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer class="flex-col sm:flex-row gap-3 sm:gap-2 mt-6">
            <AlertDialog.Cancel class="w-full sm:w-auto rounded-xl font-bold">{i18n.t('cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action
                    class="w-full sm:w-auto bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm rounded-xl font-bold"
                    onclick={handleDeleteAccount}
            >
                {#if deletingAccount}
                    <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                {/if}
                {i18n.t('delete_my_account')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>