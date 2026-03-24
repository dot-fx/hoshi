<script lang="ts">
    import type { RoomInfo } from '@/api/watchparty/types';
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import { Label } from '@/components/ui/label';
    import { Tv, Lock, User } from 'lucide-svelte';
    import { Spinner } from "$lib/components/ui/spinner";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Select from "$lib/components/ui/select";
    import { i18n } from "@/i18n/index.svelte.js";
    import LanguageSelector from "@/components/LanguageSelector.svelte";

    let { roomId, remoteUrl, onJoined }: { roomId: string, remoteUrl: string | null, onJoined: (token: string) => void } = $props();

    let roomInfo = $state<RoomInfo | null>(null);
    let isLoadingInfo = $state(true);
    let fetchError = $state(false);

    let displayName = $state('');
    let password = $state('');
    let isJoining = $state(false);
    let joinError = $state(false);

    // Definimos la base URL de forma dinámica limpiando barras finales
    const baseUrl = $derived(remoteUrl ? remoteUrl.replace(/\/$/, '') : '');

    $effect(() => {
        // GET directo con fetch usando la ruta que indicaste
        fetch(`${baseUrl}/rooms/${roomId}/join`)
            .then(res => {
                if (!res.ok) throw new Error("Room not found");
                return res.json();
            })
            .then(info => {
                roomInfo = info;
                isLoadingInfo = false;
            })
            .catch(err => {
                console.error("Error obteniendo info de la sala:", err);
                fetchError = true;
                isLoadingInfo = false;
            });
    });

    function getInitials(name: string) {
        if (!name) return '??';
        return name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2);
    }

    async function handleJoin(e: Event) {
        e.preventDefault();
        if (!displayName.trim()) return;
        if (roomInfo?.hasPassword && !password.trim()) return;

        isJoining = true;
        joinError = false;

        try {
            const res = await fetch(`${baseUrl}/rooms/${roomId}/join`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    displayName: displayName.trim(),
                    password: roomInfo?.hasPassword ? password : undefined
                })
            });

            if (!res.ok) throw new Error("Error joining room");

            const data = await res.json();
            onJoined(data.guestToken || data.guest_token);

        } catch (err: any) {
            console.error("Error uniendo a la sala:", err);
            joinError = true;
        } finally {
            isJoining = false;
        }
    }
</script>

<div class="m-auto w-full max-w-md p-8 bg-card border border-border/40 rounded-[2rem] shadow-2xl relative overflow-hidden">
    <div class="absolute top-0 inset-x-0 h-32 bg-gradient-to-b from-primary/10 to-transparent"></div>

    <div class="absolute top-4 right-4 z-50">
        <div class="absolute top-4 right-4 z-50">
            <LanguageSelector
                    compact={true}
                    class="h-9 bg-background/50 backdrop-blur-md border-border/50 rounded-xl px-3 gap-2 focus:ring-0 shadow-sm"
            />
        </div>
    </div>

    {#if isLoadingInfo}
        <div class="relative flex flex-col items-center gap-5 py-10 mt-6">
            <Spinner class="w-12 h-12 text-primary animate-spin" />
            <p class="text-muted-foreground font-bold animate-pulse">{i18n.t('watchparty.join.searching_room')}</p>
        </div>

    {:else if fetchError}
        <div class="relative flex flex-col items-center text-center gap-4 py-6 mt-6">
            <div class="p-4 bg-destructive/10 rounded-2xl">
                <Tv class="w-10 h-10 text-destructive" />
            </div>
            <div>
                <h2 class="text-2xl font-black mb-2">{i18n.t('watchparty.join.room_not_found')}</h2>
                <p class="text-muted-foreground font-medium">{i18n.t('watchparty.join.room_not_found_desc')}</p>
            </div>
            <Button variant="outline" class="mt-4 w-full h-12 rounded-xl font-bold" href="/">
                {i18n.t('watchparty.join.back_to_home')}
            </Button>
        </div>

    {:else if roomInfo}
        <div class="relative flex flex-col items-center gap-3 mb-8 text-center mt-4">
            <Avatar.Root class="w-20 h-20 border-4 border-background shadow-xl ring-2 ring-primary/20">
                <Avatar.Image
                        src={roomInfo.hostAvatarUrl || undefined}
                        alt={roomInfo.hostDisplayName}
                        class="object-cover"
                />
                <Avatar.Fallback class="bg-primary/10 text-primary font-black text-2xl">
                    {getInitials(roomInfo.hostDisplayName)}
                </Avatar.Fallback>
            </Avatar.Root>

            <div>
                <h1 class="text-2xl font-black tracking-tight line-clamp-1 mt-2">{roomInfo.name}</h1>
                <p class="text-muted-foreground font-medium mt-1.5 flex items-center justify-center gap-1.5">
                    <User class="w-4 h-4" /> {i18n.t('watchparty.join.host')}: <span class="text-foreground font-bold">{roomInfo.hostDisplayName}</span>
                </p>
            </div>
        </div>

        <form onsubmit={handleJoin} class="relative space-y-5">
            {#if joinError}
                <div class="p-3 bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-xl font-semibold text-center">
                    {i18n.t('watchparty.join.join_error')}
                </div>
            {/if}

            <div class="space-y-2">
                <Label class="font-bold ml-1">{i18n.t('watchparty.join.your_name')}</Label>
                <Input
                        bind:value={displayName}
                        placeholder={i18n.t('watchparty.join.your_name_placeholder')}
                        required
                        disabled={isJoining}
                        class="h-12 rounded-xl bg-muted/20 border-border/50 font-medium text-base"
                />
            </div>

            {#if roomInfo.hasPassword}
                <div class="space-y-2">
                    <Label class="font-bold ml-1 flex items-center gap-2">
                        <Lock class="w-4 h-4 text-muted-foreground" /> {i18n.t('watchparty.join.room_password')}
                    </Label>
                    <Input
                            type="password"
                            bind:value={password}
                            placeholder="••••••••"
                            required
                            disabled={isJoining}
                            class="h-12 rounded-xl bg-muted/20 border-border/50 font-medium text-base"
                    />
                </div>
            {/if}

            <Button
                    type="submit"
                    size="lg"
                    class="w-full h-12 rounded-xl font-bold text-base shadow-md mt-4"
                    disabled={isJoining || !displayName.trim() || (roomInfo.hasPassword && !password.trim())}
            >
                {#if isJoining}
                    <Spinner class="w-5 h-5 mr-2 animate-spin" /> {i18n.t('watchparty.join.joining')}
                {:else}
                    {i18n.t('watchparty.join.join_room')}
                {/if}
            </Button>
        </form>
    {/if}
</div>