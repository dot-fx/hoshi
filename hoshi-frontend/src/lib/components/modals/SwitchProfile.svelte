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
    import { Plus, ArrowLeft, Lock } from 'lucide-svelte';
    import { Spinner } from "$lib/components/ui/spinner";


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

    async function goToSetup() {
        open = false;
        await auth.logout();
        goto('/setup');
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-sm bg-card/95 backdrop-blur-xl border-border/50 p-6">
        {#if !selectedUser}
            <Dialog.Header class="mb-4">
                <Dialog.Title class="text-xl font-black text-left">{i18n.t('layout.switch_profile')}</Dialog.Title>
                <Dialog.Description class="text-left">{i18n.t('layout.who_is_watching')}</Dialog.Description>
            </Dialog.Header>

            <div class="flex flex-col gap-2">
                {#if loading}
                    <div class="flex justify-center py-8"><Spinner class="size-8 animate-spin text-primary" /></div>
                {:else}
                    {#each users as user}
                        <button
                                class="flex items-center gap-4 p-2.5 rounded-xl hover:bg-muted/60 transition-colors w-full outline-none focus-visible:ring-2 ring-primary ring-offset-2 ring-offset-background group"
                                onclick={() => handleUserClick(user)}
                        >
                            <Avatar.Root class="size-10 sm:size-12 border border-border group-hover:border-primary/50 transition-colors shrink-0">
                                <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                                <Avatar.Fallback class="bg-primary/10 text-primary font-bold text-sm">
                                    {user.username[0].toUpperCase()}
                                </Avatar.Fallback>
                            </Avatar.Root>

                            <span class="flex-1 text-left text-sm font-semibold text-foreground truncate">
                                {user.username}
                            </span>

                            {#if user.hasPassword}
                                <div class="shrink-0 bg-background/50 rounded-full p-1.5 border border-border/50 group-hover:border-border transition-colors">
                                    <Lock class="size-3.5 text-muted-foreground" />
                                </div>
                            {/if}
                        </button>
                    {/each}

                    <div class="h-px w-full bg-border/40 my-1"></div>

                    <button
                            class="flex items-center gap-4 p-2.5 rounded-xl hover:bg-muted/60 transition-colors w-full outline-none focus-visible:ring-2 ring-primary ring-offset-2 ring-offset-background group"
                            onclick={goToSetup}
                    >
                        <div class="size-10 sm:size-12 rounded-full border border-dashed border-muted-foreground/50 flex items-center justify-center group-hover:border-primary/50 group-hover:bg-primary/5 transition-all shrink-0">
                            <Plus class="size-5 text-muted-foreground group-hover:text-primary transition-colors" />
                        </div>
                        <span class="flex-1 text-left text-sm font-semibold text-muted-foreground group-hover:text-foreground transition-colors">
                            {i18n.t('layout.add_profile')}
                        </span>
                    </button>
                {/if}
            </div>
        {:else}
            <Dialog.Header class="flex flex-row items-center gap-2 space-y-0 pb-2">
                <Button variant="ghost" size="icon" class="shrink-0 -ml-2 rounded-full h-8 w-8" onclick={() => { selectedUser = null; error = null; }}>
                    <ArrowLeft class="size-4" />
                </Button>
                <div class="flex items-center gap-3">
                    <Avatar.Root class="size-8 border border-border">
                        <Avatar.Image src={selectedUser.avatar} />
                        <Avatar.Fallback class="bg-primary/10 text-primary font-bold text-xs">{selectedUser.username[0].toUpperCase()}</Avatar.Fallback>
                    </Avatar.Root>
                    <Dialog.Title class="text-lg font-black">{selectedUser.username}</Dialog.Title>
                </div>
            </Dialog.Header>

            <form class="py-2 space-y-4" onsubmit={(e) => { e.preventDefault(); if(selectedUser) attemptLogin(selectedUser.id, passwordInput); }}>
                <div class="space-y-2">
                    <Input
                            type="password"
                            placeholder={i18n.t('layout.enter_password')}
                            bind:value={passwordInput}
                            class="h-11"
                            autofocus
                    />
                    {#if error}
                        <p class="text-sm font-semibold text-destructive">{error}</p>
                    {/if}
                </div>
                <Button type="submit" class="w-full h-11 font-bold" disabled={loginLoading || !passwordInput}>
                    {#if loginLoading}
                        <Spinner class="size-4 animate-spin mr-2" />
                    {/if}
                    {i18n.t('layout.login')}
                </Button>
            </form>
        {/if}
    </Dialog.Content>
</Dialog.Root>