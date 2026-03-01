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
    import { Plus, Upload, X, User } from "lucide-svelte";
    import { fade } from "svelte/transition";

    const usersPromise = usersApi.getAll();
    type DialogMode = "login" | "create" | null;

    let dialogOpen = $state(false);
    let dialogMode = $state<DialogMode>(null);
    let selectedUser = $state<any>(null);
    let password = $state("");
    let username = $state("");

    let avatarFile = $state<File | null>(null);
    let previewAvatarUrl = $state<string | null>(null);
    let fileInput: HTMLInputElement | undefined = $state();
    let isSubmitting = $state(false);

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

    function handleFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            const file = target.files[0];
            avatarFile = file;
            previewAvatarUrl = URL.createObjectURL(file);
        }
    }

    function clearAvatarSelection() {
        avatarFile = null;
        previewAvatarUrl = null;
        if (fileInput) fileInput.value = "";
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        isSubmitting = true;

        try {
            if (dialogMode === "login" && selectedUser) {
                await auth.login(selectedUser.id, password);
                goto("/home");
            }

            if (dialogMode === "create") {
                if (!username.trim()) {
                    toast.error("Username is required");
                    isSubmitting = false;
                    return;
                }

                await auth.register({
                    username: username.trim(),
                    password: password || undefined,
                }, avatarFile);

                goto("/home");
            }

            dialogOpen = false;
        } catch (err: any) {
            toast.error(err?.message ?? "Something went wrong");
        } finally {
            isSubmitting = false;
        }
    }

    $effect(() => {
        if (!dialogOpen) {
            password = "";
            username = "";
            selectedUser = null;
            dialogMode = null;
            clearAvatarSelection();
        }
    });
</script>

<svelte:head>
    <title>Users</title>
</svelte:head>

<main class="flex flex-col items-center justify-center min-h-[80vh] gap-12 px-4">
    <h1 class="text-3xl md:text-5xl font-semibold text-center text-foreground tracking-tight">
        Who's watching?
    </h1>

    {#await usersPromise}
        <div in:fade class="flex flex-col items-center gap-4 text-muted-foreground">
            <div class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
            <p class="animate-pulse font-medium">Loading profiles...</p>
        </div>
    {:then data}
        <div in:fade class="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12 w-full max-w-4xl place-items-center">
            {#each data.users as user}
                <button
                        onclick={() => handleSelect(user)}
                        class="group flex flex-col items-center gap-4 cursor-pointer outline-none"
                >
                    <Avatar.Root class="w-24 h-24 md:w-32 md:h-32 shadow-sm transition-all duration-300 group-hover:scale-105 ring-offset-background group-hover:ring-4 group-hover:ring-primary/50 group-hover:ring-offset-4 group-focus-visible:ring-4 group-focus-visible:ring-primary/50">
                        {#if user.avatar}
                            <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                        {:else}
                            <Avatar.Fallback class="bg-primary/10 text-primary text-3xl md:text-5xl font-medium uppercase">
                                {user.username.charAt(0)}
                            </Avatar.Fallback>
                        {/if}
                    </Avatar.Root>

                    <span class="text-muted-foreground group-hover:text-foreground text-lg md:text-xl transition-colors font-medium">
                        {user.username}
                    </span>
                </button>
            {/each}

            <button
                    onclick={handleAdd}
                    class="group flex flex-col items-center gap-4 cursor-pointer outline-none"
            >
                <div class="w-24 h-24 md:w-32 md:h-32 rounded-full border-2 border-dashed border-border flex items-center justify-center text-muted-foreground transition-all duration-300 group-hover:scale-105 group-hover:border-foreground group-hover:text-foreground bg-muted/10 group-hover:bg-muted/40 shadow-sm">
                    <Plus class="h-10 w-10 md:h-12 md:w-12" />
                </div>

                <span class="text-muted-foreground group-hover:text-foreground text-lg md:text-xl transition-colors font-medium">
                    Add Profile
                </span>
            </button>
        </div>
    {:catch error}
        <p class="text-destructive font-medium bg-destructive/10 px-4 py-2 rounded-md">
            Error loading users
        </p>
    {/await}
</main>

<Dialog.Root bind:open={dialogOpen}>
    <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
            <Dialog.Title class="text-xl">
                {dialogMode === "login" ? "Enter password" : "Create profile"}
            </Dialog.Title>

            <Dialog.Description>
                {dialogMode === "login"
                    ? `Please enter the password for ${selectedUser?.username}`
                    : "Set up a new user profile."}
            </Dialog.Description>
        </Dialog.Header>

        <form class="grid gap-6 mt-4" onsubmit={handleSubmit}>
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
                    <Input id="username" bind:value={username} required placeholder="Enter a name" />
                </div>

                <div class="grid gap-2">
                    <Label for="password">Password (optional)</Label>
                    <Input id="password" type="password" bind:value={password} placeholder="Leave blank for no password" />
                </div>

                <div class="grid gap-3">
                    <Label>Profile Picture (optional)</Label>
                    <div class="flex items-center gap-4 bg-muted/30 p-3 rounded-lg border border-border/50">
                        <Avatar.Root class="w-16 h-16 border-2 border-background shadow-sm">
                            {#if previewAvatarUrl}
                                <Avatar.Image src={previewAvatarUrl} class="object-cover" />
                            {:else}
                                <Avatar.Fallback class="bg-primary/10 text-primary">
                                    <User class="h-6 w-6" />
                                </Avatar.Fallback>
                            {/if}
                        </Avatar.Root>

                        <div class="flex flex-col gap-2 w-full">
                            <Button type="button" variant="secondary" size="sm" class="w-full shadow-sm" onclick={() => fileInput.click()}>
                                <Upload class="mr-2 h-4 w-4" /> Choose Image
                            </Button>
                            {#if previewAvatarUrl}
                                <Button type="button" variant="ghost" size="sm" class="w-full text-destructive hover:bg-destructive/10 hover:text-destructive h-8" onclick={clearAvatarSelection}>
                                    <X class="mr-2 h-4 w-4" /> Remove
                                </Button>
                            {/if}
                        </div>
                    </div>
                    <input
                            bind:this={fileInput}
                            type="file"
                            accept="image/png, image/jpeg, image/webp, image/gif"
                            class="hidden"
                            onchange={handleFileSelect}
                    />
                </div>
            {/if}

            <Dialog.Footer class="mt-2 flex-col sm:flex-row gap-2">
                <Dialog.Close class="w-full sm:w-auto">
                    <Button type="button" variant="outline" class="w-full">
                        Cancel
                    </Button>
                </Dialog.Close>

                <Button type="submit" disabled={isSubmitting} class="w-full sm:w-auto shadow-sm">
                    {dialogMode === "login" ? "Login" : "Create Profile"}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>