<script lang="ts">
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { usersApi } from "$lib/api/users/users";
    import { auth } from "$lib/auth.svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";

    const usersPromise = usersApi.getAll();
    type DialogMode = "login" | "create" | null;

    let dialogOpen = $state(false);
    let dialogMode = $state<DialogMode>(null);
    let selectedUser = $state<any>(null);
    let password = $state("");
    let username = $state("");
    let profilePictureUrl = $state("");

    async function handleSelect(user: any) {
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

    async function handleSubmit(e: Event) {
        e.preventDefault();

        try {
            if (dialogMode === "login" && selectedUser) {
                await auth.login(selectedUser.id, password);
                goto("/home");
            }

            if (dialogMode === "create") {
                if (!username.trim()) {
                    toast.error("Username is required");
                    return;
                }

                await auth.register({
                    username: username.trim(),
                    password: password || undefined,
                    profilePictureUrl: profilePictureUrl || undefined
                });

                goto("/home");
            }

            dialogOpen = false;
        } catch (err: any) {
            toast.error(err?.message ?? "Something went wrong");
        }
    }

    $effect(() => {
        if (!dialogOpen) {
            password = "";
            username = "";
            selectedUser = null;
            dialogMode = null;
            profilePictureUrl = "";
        }
    });
</script>

<main class="flex flex-col items-center justify-center min-h-[80vh] gap-12">
    <h1 class="text-3xl md:text-5xl font-semibold text-center text-foreground">
        ¿Who's there?
    </h1>

    {#await usersPromise}
        <p class="text-muted-foreground">Loading profiles...</p>

    {:then data}
        <div class="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12 w-full max-w-4xl">
            {#each data.users as user}
                <button
                        onclick={() => handleSelect(user)}
                        class="group flex flex-col items-center gap-4 cursor-pointer outline-none"
                >
                    <Avatar.Root class="w-24 h-24 md:w-32 md:h-32 transition-all duration-200 group-hover:scale-105 ring-offset-background group-hover:ring-2 group-hover:ring-ring group-hover:ring-offset-2">
                        {#if user.profilePictureUrl}
                            <Avatar.Image src={user.profilePictureUrl} alt={user.username} class="object-cover" />
                        {:else}
                            <Avatar.Fallback class="bg-muted text-muted-foreground text-3xl font-medium">
                                {user.username.charAt(0).toUpperCase()}
                            </Avatar.Fallback>
                        {/if}
                    </Avatar.Root>

                    <span class="text-muted-foreground group-hover:text-foreground transition-colors font-medium">
                        {user.username}
                    </span>
                </button>
            {/each}

            <button
                    onclick={handleAdd}
                    class="group flex flex-col items-center gap-4 cursor-pointer outline-none"
            >
                <div class="w-24 h-24 md:w-32 md:h-32 rounded-full border-2 border-dashed border-border flex items-center justify-center text-4xl text-muted-foreground transition-all duration-200 group-hover:scale-105 group-hover:border-foreground group-hover:text-foreground bg-muted/10 group-hover:bg-muted/40">
                    +
                </div>

                <span class="text-muted-foreground group-hover:text-foreground transition-colors font-medium">
                    Agregar
                </span>
            </button>
        </div>

    {:catch error}
        <p class="text-destructive">Error loading users</p>
    {/await}
</main>

<Dialog.Root bind:open={dialogOpen}>
    <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
            <Dialog.Title>
                {dialogMode === "login" ? "Enter password" : "Create profile"}
            </Dialog.Title>

            <Dialog.Description>
                {dialogMode === "login"
                    ? `Password for ${selectedUser?.username}`
                    : "Create a new profile"}
            </Dialog.Description>
        </Dialog.Header>

        <form
                class="grid gap-4 mt-4"
                onsubmit={handleSubmit}
        >
            {#if dialogMode === "login"}
                <div class="grid gap-2">
                    <Label for="password">Password</Label>
                    <Input
                            id="password"
                            type="password"
                            bind:value={password}
                            autofocus
                    />
                </div>

            {:else if dialogMode === "create"}
                <div class="grid gap-2">
                    <Label for="username">Username</Label>
                    <Input id="username" bind:value={username} required />
                </div>

                <div class="grid gap-2">
                    <Label for="password">Password (optional)</Label>
                    <Input id="password" type="password" bind:value={password} />
                </div>

                <div class="grid gap-2">
                    <Label for="avatar">Profile picture URL (optional)</Label>
                    <Input id="avatar" bind:value={profilePictureUrl} />
                </div>
            {/if}

            <Dialog.Footer class="mt-4">
                <Dialog.Close>
                    <Button type="button" variant="outline">
                        Cancel
                    </Button>
                </Dialog.Close>

                <Button type="submit">
                    {dialogMode === "login" ? "Login" : "Create"}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>