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
    import { i18n } from '$lib/i18n/index.svelte';

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
    <title>{i18n.t('page_title_users')}</title>
</svelte:head>

<main class="flex flex-col items-center justify-center min-h-[80vh] gap-12 px-4 md:px-8">

    <h1 class="text-4xl md:text-5xl font-black text-center text-foreground tracking-tight">
        {i18n.t('whos_watching')}
    </h1>

    {#await usersPromise}
        <div in:fade class="flex flex-col items-center gap-4 text-muted-foreground">
            <div class="h-10 w-10 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
            <p class="animate-pulse font-bold">{i18n.t('loading_profiles')}</p>
        </div>
    {:then data}
        <div in:fade class="grid grid-cols-2 md:grid-cols-4 gap-8 md:gap-12 w-full max-w-4xl place-items-center">
            {#each data.users as user}
                <button
                        onclick={() => handleSelect(user)}
                        class="group flex flex-col items-center gap-5 cursor-pointer outline-none"
                >
                    <Avatar.Root class="w-24 h-24 md:w-32 md:h-32 shadow-lg transition-all duration-300 group-hover:scale-105 ring-offset-background group-hover:ring-4 group-hover:ring-primary/50 group-hover:ring-offset-4 group-focus-visible:ring-4 group-focus-visible:ring-primary/50 group-focus-visible:ring-offset-4 group-focus-visible:scale-105">
                        {#if user.avatar}
                            <Avatar.Image src={user.avatar} alt={user.username} class="object-cover" />
                        {:else}
                            <Avatar.Fallback class="bg-primary/10 text-primary text-4xl md:text-5xl font-black uppercase">
                                {user.username.charAt(0)}
                            </Avatar.Fallback>
                        {/if}
                    </Avatar.Root>

                    <span class="text-muted-foreground group-hover:text-foreground group-focus-visible:text-foreground text-lg md:text-xl transition-colors font-bold tracking-tight">
                        {user.username}
                    </span>
                </button>
            {/each}

            <!-- Add Profile Button -->
            <button
                    onclick={handleAdd}
                    class="group flex flex-col items-center gap-5 cursor-pointer outline-none"
            >
                <div class="w-24 h-24 md:w-32 md:h-32 rounded-full border-2 border-dashed border-border/60 flex items-center justify-center text-muted-foreground transition-all duration-300 group-hover:scale-105 group-hover:border-foreground group-hover:text-foreground bg-muted/10 group-hover:bg-muted/40 shadow-sm group-focus-visible:ring-4 group-focus-visible:ring-primary/50 group-focus-visible:ring-offset-4 group-focus-visible:scale-105 ring-offset-background">
                    <Plus class="h-10 w-10 md:h-12 md:w-12" />
                </div>

                <span class="text-muted-foreground group-hover:text-foreground group-focus-visible:text-foreground text-lg md:text-xl transition-colors font-bold tracking-tight">
                    {i18n.t('add_profile')}
                </span>
            </button>
        </div>
    {:catch error}
        <div class="text-destructive font-bold bg-destructive/10 px-6 py-4 rounded-xl border border-destructive/20 text-center">
            {i18n.t('error_loading_users')}
        </div>
    {/await}
</main>

<Dialog.Root bind:open={dialogOpen}>
    <Dialog.Content class="sm:max-w-md sm:rounded-2xl border-border/50 shadow-lg">
        <Dialog.Header class="space-y-2">
            <Dialog.Title class="text-2xl font-black tracking-tight">
                {dialogMode === "login" ? i18n.t('enter_password') : i18n.t('create_profile')}
            </Dialog.Title>

            <Dialog.Description class="text-base font-medium">
                {dialogMode === "login"
                    ? `${i18n.t('enter_password_for')} ${selectedUser?.username}`
                    : i18n.t('setup_new_profile')}
            </Dialog.Description>
        </Dialog.Header>

        <form class="grid gap-6 mt-2" onsubmit={handleSubmit}>
            {#if dialogMode === "login"}
                <div class="grid gap-2">
                    <Label for="password" class="font-bold text-foreground/90">{i18n.t('password')}</Label>
                    <Input
                            id="password"
                            type="password"
                            bind:value={password}
                            class="rounded-xl h-11 bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50"
                            autofocus
                    />
                </div>
            {:else if dialogMode === "create"}
                <div class="grid gap-2">
                    <Label for="username" class="font-bold text-foreground/90">{i18n.t('username')}</Label>
                    <Input
                            id="username"
                            bind:value={username}
                            required
                            placeholder={i18n.t('enter_name')}
                            class="rounded-xl h-11 bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50"
                    />
                </div>

                <div class="grid gap-2">
                    <Label for="password" class="font-bold text-foreground/90">{i18n.t('password_optional')}</Label>
                    <Input
                            id="password"
                            type="password"
                            bind:value={password}
                            placeholder={i18n.t('leave_blank_no_password')}
                            class="rounded-xl h-11 bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50"
                    />
                </div>

                <div class="grid gap-3">
                    <Label class="font-bold text-foreground/90">{i18n.t('profile_picture_optional')}</Label>
                    <div class="flex items-center gap-4 bg-muted/20 p-4 rounded-xl border border-border/50">
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
                            <Button type="button" variant="secondary" size="sm" class="w-full shadow-sm rounded-lg font-bold" onclick={() => fileInput.click()}>
                                <Upload class="mr-2 h-4 w-4" /> {i18n.t('choose_image')}
                            </Button>
                            {#if previewAvatarUrl}
                                <Button type="button" variant="ghost" size="sm" class="w-full text-destructive hover:bg-destructive/10 hover:text-destructive h-8 rounded-lg font-bold" onclick={clearAvatarSelection}>
                                    <X class="mr-2 h-4 w-4" /> {i18n.t('remove')}
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

            <Dialog.Footer class="mt-4 flex-col sm:flex-row gap-3 sm:gap-2">
                <Dialog.Close class="w-full sm:w-auto">
                    <Button type="button" variant="outline" class="w-full h-11 rounded-xl font-bold">
                        {i18n.t('cancel')}
                    </Button>
                </Dialog.Close>

                <Button type="submit" disabled={isSubmitting} class="w-full sm:w-auto shadow-sm h-11 rounded-xl px-6 font-bold">
                    {dialogMode === "login" ? i18n.t('login') : i18n.t('create_profile')}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>