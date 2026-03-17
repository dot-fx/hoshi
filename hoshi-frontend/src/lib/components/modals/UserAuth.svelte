<script lang="ts">
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { auth } from "$lib/auth.svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { Upload, X, User } from "lucide-svelte";
    import { i18n } from '$lib/i18n/index.svelte';
    import type { UserResponse } from "$lib/api/users/types";

    let {
        open = $bindable(false),
        mode = $bindable(null),
        selectedUser = $bindable(null)
    }: {
        open: boolean,
        mode: "login" | "create" | null,
        selectedUser: UserResponse | null
    } = $props();

    let password = $state("");
    let username = $state("");
    let avatarFile = $state<File | null>(null);
    let previewAvatarUrl = $state<string | null>(null);
    let fileInput = $state<HTMLInputElement>();
    let isSubmitting = $state(false);

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
            if (mode === "login" && selectedUser) {
                await auth.login(selectedUser.id, password);
                goto("/home");
            } else if (mode === "create") {
                if (!username.trim()) {
                    toast.error("Username is required");
                    isSubmitting = false;
                    return;
                }
                await auth.register({
                    username: username.trim(),
                    password: password || undefined,
                }, avatarFile);
                goto("/setup?mode=user");
            }
            open = false;
        } catch (err: any) {
            toast.error(err?.message ?? "Something went wrong");
        } finally {
            isSubmitting = false;
        }
    }

    $effect(() => {
        if (!open) {
            password = "";
            username = "";
            clearAvatarSelection();
        }
    });
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-md sm:rounded-2xl border-border/50 shadow-lg">
        <Dialog.Header class="space-y-2">
            <Dialog.Title class="text-2xl font-black tracking-tight">
                {mode === "login" ? i18n.t('users.enter_password') : i18n.t('users.create_profile')}
            </Dialog.Title>
        </Dialog.Header>

        <form class="grid gap-6 mt-2" onsubmit={handleSubmit}>
            {#if mode === "login"}
                <div class="grid gap-2">
                    <Label for="password" class="font-bold text-foreground/90">{i18n.t('users.password')}</Label>
                    <Input id="password" type="password" bind:value={password} autofocus class="rounded-xl h-11" />
                </div>
            {:else if mode === "create"}
                <div class="grid gap-2">
                    <Label for="username" class="font-bold text-foreground/90">{i18n.t('users.username')}</Label>
                    <Input id="username" bind:value={username} required placeholder={i18n.t('users.enter_username')} class="rounded-xl h-11" />
                </div>
                <div class="grid gap-2">
                    <Label for="password" class="font-bold text-foreground/90">{i18n.t('users.optional_password')}</Label>
                    <Input id="password" type="password" bind:value={password} placeholder={i18n.t('users.password_placeholder')} class="rounded-xl h-11" />
                </div>

                <div class="grid gap-3">
                    <Label class="font-bold text-foreground/90">{i18n.t('users.profile_picture')}</Label>
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
                            <Button type="button" variant="secondary" size="sm" class="w-full font-bold" onclick={() => fileInput?.click()}>
                                <Upload class="mr-2 h-4 w-4" /> {i18n.t('users.upload_image')}
                            </Button>
                            {#if previewAvatarUrl}
                                <Button type="button" variant="ghost" size="sm" class="w-full text-destructive" onclick={clearAvatarSelection}>
                                    <X class="mr-2 h-4 w-4" /> {i18n.t('users.remove_image')}
                                </Button>
                            {/if}
                        </div>
                    </div>
                    <input bind:this={fileInput} type="file" accept="image/*" class="hidden" onchange={handleFileSelect} />
                </div>
            {/if}

            <Dialog.Footer class="mt-4 flex-col sm:flex-row gap-3 sm:gap-2">
                <Dialog.Close class="w-full sm:w-auto">
                    <Button type="button" variant="outline" class="w-full h-11 rounded-xl font-bold">
                        {i18n.t('users.cancel')}
                    </Button>
                </Dialog.Close>
                <Button type="submit" disabled={isSubmitting} class="w-full sm:w-auto h-11 rounded-xl px-6 font-bold">
                    {mode === "login" ? i18n.t('users.login') : i18n.t('users.create_profile')}
                </Button>
            </Dialog.Footer>
        </form>
    </Dialog.Content>
</Dialog.Root>