<script lang="ts">
    import * as Avatar from "$lib/components/ui/avatar";
    import { usersApi } from "$lib/api/users/users";
    import { auth } from "$lib/auth.svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { Plus } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { i18n } from '$lib/i18n/index.svelte';
    import UserDialog from "$lib/components/modals/UserAuth.svelte";
    import type { UsersResponse, UserResponse } from "$lib/api/users/types";

    const usersPromise: Promise<UsersResponse> = usersApi.getAll();

    let dialogOpen = $state(false);
    let dialogMode = $state<"login" | "create" | null>(null);
    let selectedUser = $state<UserResponse | null>(null);

    async function handleSelect(user: UserResponse) {
        if (user.hasPassword) {
            selectedUser = user;
            dialogMode = "login";
            dialogOpen = true;
            return;
        }

        try {
            await auth.login(user.id);
            goto("/home");
        } catch (err: any) {
            toast.error(err?.message ?? "Login failed");
        }
    }

    function handleAdd() {
        dialogMode = "create";
        dialogOpen = true;
    }
</script>

<svelte:head>
    <title>{i18n.t('users.title')}</title>
</svelte:head>

<main class="flex flex-col items-center justify-center min-h-[80vh] gap-12 px-4 md:px-8">
    <h1 class="text-4xl md:text-5xl font-black text-center text-foreground tracking-tight">
        {i18n.t('users.who_is_watching')}
    </h1>

    {#await usersPromise}
        <div in:fade class="flex flex-col items-center gap-4 text-muted-foreground">
            <div class="h-10 w-10 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
            <p class="animate-pulse font-bold">{i18n.t('users.loading')}</p>
        </div>
    {:then data}
        <div in:fade class="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12 w-full max-w-4xl place-items-center">
            {#each data.users as user (user.id)}
                <button
                        onclick={() => handleSelect(user)}
                        class="group flex flex-col items-center gap-5 cursor-pointer outline-none"
                >
                    <Avatar.Root class="w-24 h-24 md:w-32 md:h-32 shadow-lg transition-all duration-300 group-hover:scale-105 ring-offset-background group-hover:ring-4 group-hover:ring-primary/50 group-hover:ring-offset-4">
                        {#if user.avatar}
                            <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                        {:else}
                            <Avatar.Fallback class="bg-primary/10 text-primary text-4xl md:text-5xl font-black uppercase">
                                {user.username.charAt(0)}
                            </Avatar.Fallback>
                        {/if}
                    </Avatar.Root>
                    <span class="text-muted-foreground group-hover:text-foreground text-lg md:text-xl font-bold tracking-tight">
                        {user.username}
                    </span>
                </button>
            {/each}

            <button onclick={handleAdd} class="group flex flex-col items-center gap-5 cursor-pointer outline-none">
                <div class="w-24 h-24 md:w-32 md:h-32 rounded-full border-2 border-dashed border-border/60 flex items-center justify-center text-muted-foreground transition-all duration-300 group-hover:scale-105 group-hover:border-foreground group-hover:text-foreground bg-muted/10">
                    <Plus class="h-10 w-10 md:h-12 md:w-12" />
                </div>
                <span class="text-muted-foreground group-hover:text-foreground text-lg md:text-xl font-bold tracking-tight">
                    {i18n.t('users.add_profile')}
                </span>
            </button>
        </div>
    {:catch error}
        <div class="text-destructive font-bold bg-destructive/10 px-6 py-4 rounded-xl border border-destructive/20 text-center">
            {i18n.t('errors.network')}
        </div>
    {/await}
</main>

<UserDialog bind:open={dialogOpen} bind:mode={dialogMode} bind:selectedUser />