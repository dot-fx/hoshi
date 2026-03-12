<script lang="ts">
    import { usersApi } from "@/api/users/users";
    import type { UserPrivate } from "@/api/users/types";
    import { toast } from "svelte-sonner";
    import { i18n } from "$lib/i18n/index.svelte";

    import * as Avatar from "$lib/components/ui/avatar";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";

    import { Loader2, Trash2, AlertTriangle, Camera } from "lucide-svelte";

    let { user, onUpdate }: { user: UserPrivate, onUpdate: () => Promise<void> } = $props();

    // --- ESTADOS DE CUENTA ---
    let username = $state("");

    $effect(() => {
        username = user.username;
    });

    let savingProfile = $state(false);
    let previewAvatarUrl = $state<string>("");

    $effect(() => {
        previewAvatarUrl = user.avatar || "";
    })
    let selectedAvatarFile = $state<File | null>(null);
    let avatarRemoved = $state(false);
    let fileInput: HTMLInputElement | undefined = $state();

    // --- DERIVED: HAY CAMBIOS SIN GUARDAR ---
    let hasChanges = $derived(
        username !== user.username ||
        selectedAvatarFile !== null ||
        avatarRemoved
    );

    let currentPassword = $state("");
    let newPassword = $state("");
    let confirmPassword = $state("");
    let savingPassword = $state(false);
    let canSavePassword = $derived(
        newPassword.length >= 8 &&
        newPassword === confirmPassword &&
        (!user.hasPassword || currentPassword.length > 0)
    );

    let showDeleteAlert = $state(false);
    let deletingAccount = $state(false);
    let deletePassword = $state("");

    // --- FUNCIONES ---
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
        if (fileInput) fileInput.value = "";
    }

    async function handleUpdateProfile(e: Event) {
        e.preventDefault();
        if (!hasChanges) return;

        savingProfile = true;
        try {
            if (username !== user.username) await usersApi.updateMe({ username });
            if (selectedAvatarFile) await usersApi.uploadAvatar(selectedAvatarFile);
            else if (avatarRemoved && user.avatar) await usersApi.deleteAvatar();

            toast.success(i18n.t('profile_updated')); // i18n
            await onUpdate();

            selectedAvatarFile = null;
            avatarRemoved = false;
        } catch (error: any) {
            toast.error(error?.message || i18n.t('failed_update_profile')); // i18n
        } finally {
            savingProfile = false;
        }
    }

    async function handleChangePassword(e: Event) {
        e.preventDefault();
        savingPassword = true;
        try {
            await usersApi.changePassword({ currentPassword, newPassword });
            toast.success(i18n.t('password_updated')); // i18n
            currentPassword = ""; newPassword = ""; confirmPassword = "";
        } catch (error: any) {
            toast.error(error?.message || i18n.t('failed_change_password')); // i18n
        } finally {
            savingPassword = false;
        }
    }

    async function handleDeleteAccount() {
        if (user.hasPassword && !deletePassword) { toast.error(i18n.t('password_required_delete')); return; } // i18n
        deletingAccount = true;
        try {
            await usersApi.deleteMe({ password: deletePassword });
            toast.success(i18n.t('account_deleted')); // i18n
            showDeleteAlert = false;
            window.location.href = "/";
        } catch (error: any) {
            toast.error(error?.message || i18n.t('failed_delete_account')); // i18n
            showDeleteAlert = false;
        } finally {
            deletingAccount = false;
        }
    }
</script>

<div class="space-y-16 w-full">

    <!-- Profile Section -->
    <section>
        <div class="mb-2">
            <h2 class="text-2xl font-bold tracking-tight">{i18n.t('public_profile')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('public_profile_desc')}</p>
        </div>

        <form onsubmit={handleUpdateProfile} class="space-y-0">
            <!-- Fila limpia: Avatar a la izquierda, Username a la derecha -->
            <div class="flex flex-col sm:flex-row items-center sm:items-start gap-8 py-8 border-b border-border/40">

                <!-- Columna Izquierda: Avatar y pequeños controles -->
                <div class="shrink-0 flex flex-col items-center gap-3 sm:w-40">
                    <button
                            type="button"
                            class="relative group rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary transition-transform active:scale-95"
                            onclick={() => fileInput.click()}
                            aria-label={i18n.t('change_avatar')}
                    >
                        <Avatar.Root class="h-24 w-24 sm:h-28 sm:w-28 border border-border/50 shadow-sm transition-opacity group-hover:opacity-90">
                            {#if previewAvatarUrl}
                                <Avatar.Image src={previewAvatarUrl} alt={username} class="object-cover" />
                            {/if}
                            <Avatar.Fallback class="bg-primary/10 text-primary text-4xl font-bold uppercase">
                                {username.charAt(0)}
                            </Avatar.Fallback>
                        </Avatar.Root>

                        <div class="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity text-white backdrop-blur-[2px]">
                            <Camera class="size-8" />
                        </div>
                    </button>
                    <input bind:this={fileInput} type="file" accept="image/png, image/jpeg, image/webp" class="hidden" onchange={handleFileSelect} />

                    <div class="flex items-center gap-3 text-sm">
                        <button type="button" class="font-semibold text-muted-foreground hover:text-foreground transition-colors" onclick={() => fileInput.click()}>
                            {i18n.t('change_avatar').split(' ')[0]} <!-- Mostramos solo "Change" -->
                        </button>
                        {#if previewAvatarUrl}
                            <span class="text-border/50">•</span>
                            <button type="button" class="font-semibold text-destructive hover:underline transition-all" onclick={handleRemoveAvatar}>
                                {i18n.t('remove')}
                            </button>
                        {/if}
                    </div>
                </div>

                <!-- Columna Derecha: Username -->
                <div class="flex-1 w-full space-y-2 sm:pt-2">
                    <Label for="username" class="text-base font-bold">{i18n.t('username')}</Label>
                    <Input id="username" bind:value={username} class="rounded-xl h-11 w-full sm:max-w-md" required />
                    <p class="text-sm text-muted-foreground">{i18n.t('username_help')}</p>
                </div>
            </div>

            <!-- Botón de guardado -->
            <div class="flex justify-end pt-8">
                <Button type="submit" disabled={!hasChanges || savingProfile} class="rounded-xl px-8 h-11 font-bold shadow-sm transition-all">
                    {#if savingProfile}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
                    {hasChanges ? i18n.t('save_changes') : i18n.t('saved')}
                </Button>
            </div>
        </form>
    </section>

    <!-- Security Section -->
    <section>
        <div class="mb-2">
            <h2 class="text-2xl font-bold tracking-tight">{i18n.t('security')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('security_desc')}</p>
        </div>

        <form onsubmit={handleChangePassword} class="space-y-0">
            {#if user.hasPassword}
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label for="currentPassword" class="text-base font-bold">{i18n.t('current_password')}</Label>
                        <p class="text-sm text-muted-foreground">{i18n.t('current_password_help')}</p>
                    </div>
                    <Input id="currentPassword" type="password" bind:value={currentPassword} class="rounded-xl h-11 w-full sm:max-w-md" required />
                </div>
            {/if}

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label for="newPassword" class="text-base font-bold">{i18n.t('new_password')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('new_password_help')}</p>
                </div>
                <Input id="newPassword" type="password" bind:value={newPassword} class="rounded-xl h-11 w-full sm:max-w-md" required />
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label for="confirmPassword" class="text-base font-bold">{i18n.t('confirm_password')}</Label>
                    <p class="text-sm text-muted-foreground">{i18n.t('confirm_password_help')}</p>
                </div>
                <Input id="confirmPassword" type="password" bind:value={confirmPassword} class="rounded-xl h-11 w-full sm:max-w-md" required />
            </div>

            <div class="flex justify-end pt-8">
                <Button type="submit" disabled={!canSavePassword || savingPassword} variant="secondary" class="rounded-xl px-8 h-11 font-bold shadow-sm transition-all">
                    {#if savingPassword}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
                    {i18n.t('update_password')}
                </Button>
            </div>
        </form>
    </section>

    <!-- Danger Zone -->
    <section>
        <div class="mb-2">
            <h2 class="text-xl font-bold tracking-tight text-destructive">{i18n.t('danger_zone')}</h2>
            <p class="text-sm text-muted-foreground mt-1">{i18n.t('danger_zone_desc')}</p>
        </div>

        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-y border-border/40">
            <div class="space-y-1 pr-4">
                <Label class="text-base font-bold text-foreground">{i18n.t('delete_account')}</Label>
                <p class="text-sm text-muted-foreground">{i18n.t('delete_account_help')}</p>
            </div>
            <div class="flex flex-col sm:flex-row gap-3 w-full sm:w-auto shrink-0">
                {#if user.hasPassword}
                    <Input type="password" placeholder={i18n.t('verify_password')} bind:value={deletePassword} class="border-destructive/30 focus-visible:ring-destructive/50 rounded-xl h-11 w-full sm:w-64" />
                {/if}
                <Button type="button" variant="destructive" class="rounded-xl h-11 font-bold w-full sm:w-auto shadow-sm" onclick={() => showDeleteAlert = true}>
                    <Trash2 class="mr-2 h-4 w-4" /> {i18n.t('delete_account')}
                </Button>
            </div>
        </div>
    </section>
</div>

<!-- Modal -->
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
            <AlertDialog.Action class="w-full sm:w-auto bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm rounded-xl font-bold" onclick={handleDeleteAccount}>
                {#if deletingAccount}<Loader2 class="h-4 w-4 mr-2 animate-spin" />{/if}
                {i18n.t('delete_my_account')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>