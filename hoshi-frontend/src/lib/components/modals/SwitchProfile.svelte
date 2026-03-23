<script lang="ts">
    import { goto } from '$app/navigation';
    import { usersApi } from '$lib/api/users/users';
    import { auth } from '$lib/auth.svelte';
    import { i18n } from '$lib/i18n/index.svelte';
    import type { UserResponse } from '$lib/api/users/types';
    import * as Dialog from '$lib/components/ui/dialog';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import * as Avatar from '$lib/components/ui/avatar';
    import { Loader2, Plus, ArrowLeft, Lock } from 'lucide-svelte';

    let { open = $bindable(false) } = $props();

    let users = $state<UserResponse[]>([]);
    let loading = $state(false);
    let error = $state<string | null>(null);

    let selectedUser = $state<UserResponse | null>(null);
    let passwordInput = $state("");
    let loginLoading = $state(false);

    $effect(() => {
        if (open) {
            loadUsers();
            selectedUser = null;
            passwordInput = "";
            error = null;
        }
    });

    async function loadUsers() {
        loading = true;
        try {
            const res = await usersApi.getAll();
            users = res.users;
        } catch (e: any) {
            console.error("Failed to load users", e);
        } finally {
            loading = false;
        }
    }

    async function handleUserClick(user: UserResponse) {
        error = null;
        if (user.hasPassword) {
            selectedUser = user;
            passwordInput = "";
        } else {
            await attemptLogin(user.id);
        }
    }

    async function attemptLogin(userId: number, password?: string) {
        loginLoading = true;
        error = null;
        try {
            await auth.login(userId, password);
            open = false;
            location.reload();
        } catch (e: any) {
            error = e?.message || "Login failed";
        } finally {
            loginLoading = false;
        }
    }

    // Corregido: Ahora hace logout antes de ir a setup para evitar redirecciones automáticas
    async function goToSetup() {
        open = false;
        await auth.logout();
        goto('/setup');
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-md bg-card/95 backdrop-blur-xl border-border/50">
        {#if !selectedUser}
            <Dialog.Header>
                <Dialog.Title class="text-2xl font-black text-center">{i18n.t('layout.switch_profile')}</Dialog.Title>
                <Dialog.Description class="text-center">{i18n.t('layout.who_is_watching')}</Dialog.Description>
            </Dialog.Header>

            <div class="py-6">
                {#if loading}
                    <div class="flex justify-center py-8"><Loader2 class="size-8 animate-spin text-primary" /></div>
                {:else}
                    <div class="grid grid-cols-3 sm:grid-cols-4 gap-4 justify-items-center">
                        {#each users as user}
                            <button
                                    class="flex flex-col items-center gap-2 group outline-none"
                                    onclick={() => handleUserClick(user)}
                            >
                                <div class="relative">
                                    <Avatar.Root class="size-16 sm:size-20 border-2 border-transparent group-hover:border-primary transition-all group-focus-visible:ring-2 ring-primary ring-offset-2 ring-offset-background">
                                        <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                                        <Avatar.Fallback class="bg-primary/10 text-primary font-bold text-xl">
                                            {user.username[0].toUpperCase()}
                                        </Avatar.Fallback>
                                    </Avatar.Root>
                                    {#if user.hasPassword}
                                        <div class="absolute -bottom-1 -right-1 bg-background rounded-full p-1 border border-border">
                                            <Lock class="size-3 text-muted-foreground" />
                                        </div>
                                    {/if}
                                </div>
                                <span class="text-sm font-semibold text-muted-foreground group-hover:text-foreground transition-colors truncate max-w-full">
                                    {user.username}
                                </span>
                            </button>
                        {/each}

                        <button
                                class="flex flex-col items-center gap-2 group outline-none"
                                onclick={goToSetup}
                        >
                            <div class="size-16 sm:size-20 rounded-full border-2 border-dashed border-border flex items-center justify-center group-hover:border-primary group-hover:bg-primary/5 transition-all">
                                <Plus class="size-8 text-muted-foreground group-hover:text-primary transition-colors" />
                            </div>
                            <span class="text-sm font-semibold text-muted-foreground group-hover:text-foreground transition-colors">
                                {i18n.t('layout.add_profile')}
                            </span>
                        </button>
                    </div>
                {/if}
            </div>
        {:else}
            <Dialog.Header class="flex flex-row items-center gap-2 space-y-0">
                <Button variant="ghost" size="icon" class="shrink-0 -ml-2 rounded-full" onclick={() => { selectedUser = null; error = null; }}>
                    <ArrowLeft class="size-5" />
                </Button>
                <div class="flex items-center gap-3">
                    <Avatar.Root class="size-8">
                        <Avatar.Image src={selectedUser.avatar} />
                        <Avatar.Fallback class="bg-primary/10 text-primary font-bold text-xs">{selectedUser.username[0].toUpperCase()}</Avatar.Fallback>
                    </Avatar.Root>
                    <Dialog.Title class="text-xl font-black">{selectedUser.username}</Dialog.Title>
                </div>
            </Dialog.Header>

            <form class="py-6 space-y-4" onsubmit={(e) => { e.preventDefault(); if(selectedUser) attemptLogin(selectedUser.id, passwordInput); }}>
                <div class="space-y-2">
                    <Input
                            type="password"
                            placeholder={i18n.t('layout.enter_password')}
                            bind:value={passwordInput}
                            class="h-12"
                            autofocus
                    />
                    {#if error}
                        <p class="text-sm font-semibold text-destructive">{error}</p>
                    {/if}
                </div>
                <Button type="submit" class="w-full h-12 font-bold" disabled={loginLoading || !passwordInput}>
                    {#if loginLoading}
                        <Loader2 class="size-5 animate-spin mr-2" />
                    {/if}
                    {i18n.t('layout.login')}
                </Button>
            </form>
        {/if}
    </Dialog.Content>
</Dialog.Root>