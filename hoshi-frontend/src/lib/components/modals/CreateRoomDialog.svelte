<script lang="ts">
    import * as Dialog from "@/components/ui/dialog";
    import { Input } from "@/components/ui/input";
    import { Label } from "@/components/ui/label";
    import { Button } from "@/components/ui/button";
    import { Switch } from "@/components/ui/switch";
    import { Loader2, Tv } from "lucide-svelte";

    import { watchpartyApi } from "@/api/watchparty/watchparty";
    import { isTauri } from "@/api/client";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { i18n } from "@/i18n/index.svelte.js";

    let { open = $bindable(false) } = $props();

    let name = $state("");
    let password = $state("");
    let isPublic = $state(false);
    let loading = $state(false);

    $effect(() => {
        if (open) {
            name = "";
            password = "";
            isPublic = false;
            loading = false;
        }
    });

    async function handleSubmit(e: Event) {
        e.preventDefault();
        if (!name.trim()) return;

        loading = true;
        try {
            if (isTauri()) {
                await watchpartyApi.startServer();
            }

            const res = await watchpartyApi.createRoom({
                name: name.trim(),
                password: password.trim() || undefined,
                public: isPublic
            });

            const roomUrl = res.roomUrl ?? (res as any).room_url;
            const hostToken = res.hostToken ?? (res as any).host_token;
            const roomId = res.roomId ?? (res as any).room_id;

            if (hostToken && roomId) {
                sessionStorage.setItem(`wp_token_${roomId}`, hostToken);
            }

            open = false;
            toast.success(i18n.t('watchparty.room_created'));

            if (roomUrl) {
                await goto(roomUrl);
            }

        } catch (err: any) {
            console.error("Error creating room:", err);
            toast.error(i18n.t('errors.network'));
        } finally {
            loading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[425px] bg-background border-border p-0 overflow-hidden sm:rounded-2xl shadow-lg z-[100]">
        <div class="p-6 pb-4 border-b border-border/40 bg-muted/10">
            <Dialog.Title class="text-xl font-black flex items-center gap-2">
                <Tv class="w-5 h-5 text-primary" />
                {i18n.t('watchparty.create_room')}
            </Dialog.Title>
            <Dialog.Description class="text-sm font-medium mt-1">
                {i18n.t('watchparty.create_room_desc')}
            </Dialog.Description>
        </div>

        <form onsubmit={handleSubmit} class="p-6 space-y-5">
            <div class="space-y-2">
                <Label for="name" class="font-bold">{i18n.t('watchparty.room_name')}</Label>
                <Input
                        id="name"
                        bind:value={name}
                        placeholder={i18n.t('watchparty.room_name_placeholder')}
                        class="h-11 rounded-xl bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-medium"
                        required
                        disabled={loading}
                />
            </div>

            <div class="space-y-2">
                <Label for="password" class="font-bold">{i18n.t('watchparty.password')} <span class="text-muted-foreground font-normal text-xs ml-1">({i18n.t('watchparty.optional')})</span></Label>
                <Input
                        id="password"
                        type="password"
                        bind:value={password}
                        placeholder="••••••••"
                        class="h-11 rounded-xl bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-medium"
                        disabled={loading}
                />
            </div>

            <div class="flex flex-row items-center justify-between rounded-xl border border-border/50 bg-muted/10 p-4 shadow-sm">
                <div class="space-y-0.5">
                    <Label class="text-base font-bold cursor-pointer" for="isPublic">{i18n.t('watchparty.public_room')}</Label>
                    <p class="text-xs text-muted-foreground font-medium">
                        {i18n.t('watchparty.public_room_desc')}
                    </p>
                </div>
                <Switch id="isPublic" bind:checked={isPublic} disabled={loading} class="shrink-0" />
            </div>

            <div class="pt-4 flex gap-3 w-full">
                <Button
                        type="button"
                        variant="outline"
                        class="flex-1 h-11 rounded-xl font-bold border-border/50"
                        onclick={() => open = false}
                        disabled={loading}
                >
                    {i18n.t('watchparty.cancel')}
                </Button>
                <Button
                        type="submit"
                        class="flex-1 h-11 rounded-xl font-bold shadow-sm"
                        disabled={loading || !name.trim()}
                >
                    {#if loading}
                        <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                        {i18n.t('watchparty.creating')}
                    {:else}
                        {i18n.t('watchparty.create')}
                    {/if}
                </Button>
            </div>
        </form>
    </Dialog.Content>
</Dialog.Root>