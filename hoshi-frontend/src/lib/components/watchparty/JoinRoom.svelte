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

    const baseUrl = $derived(remoteUrl ? remoteUrl.replace(/\/$/, '') : '');

    $effect(() => {
        fetch(`${baseUrl}/api/rooms/${roomId}`)
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
            const res = await fetch(`${baseUrl}/api/rooms/${roomId}/join`, {
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

<div class="m-auto w-full max-w-md p-6 sm:p-10 bg-card/95 backdrop-blur-xl border border-border/50 rounded-2xl shadow-2xl relative flex flex-col items-center">
    <div class="absolute top-0 inset-x-0 h-32 bg-gradient-to-b from-primary/5 to-transparent pointer-events-none"></div>

    {#if isLoadingInfo}
        <div class="py-16 flex flex-col items-center gap-4">
            <Spinner class="w-10 h-10 text-primary animate-spin" />
            <p class="text-muted-foreground font-bold text-sm tracking-wide uppercase">{i18n.t('watchparty.join.searching_room')}</p>
        </div>

    {:else if fetchError}
        <div class="py-10 flex flex-col items-center text-center gap-5 w-full">
            <div class="p-5 bg-destructive/10 rounded-2xl">
                <Tv class="w-10 h-10 text-destructive" />
            </div>
            <div class="space-y-2">
                <h2 class="text-2xl font-black tracking-tight">{i18n.t('watchparty.join.room_not_found')}</h2>
                <p class="text-muted-foreground text-sm leading-relaxed max-w-[260px] mx-auto">
                    {i18n.t('watchparty.join.room_not_found_desc')}
                </p>
            </div>
            <Button variant="secondary" class="mt-2 w-full h-11 rounded-xl font-bold" href="/">
                {i18n.t('watchparty.join.back_to_home')}
            </Button>
        </div>

    {:else if roomInfo}
        <div class="relative flex flex-col items-center gap-4 mb-10 text-center w-full">
            <Avatar.Root class="w-20 h-20 border-4 border-background shadow-xl ring-1 ring-border/50">
                <Avatar.Image
                        src={roomInfo.hostAvatarUrl || undefined}
                        alt={roomInfo.hostDisplayName}
                        class="object-cover"
                />
                <Avatar.Fallback class="bg-primary/10 text-primary font-black text-2xl">
                    {getInitials(roomInfo.hostDisplayName)}
                </Avatar.Fallback>
            </Avatar.Root>

            <div class="space-y-1.5">
                <h1 class="text-2xl font-black tracking-tight line-clamp-1">{roomInfo.name}</h1>
                <div class="flex items-center justify-center gap-2 text-xs font-bold text-muted-foreground bg-muted/50 px-3 py-1.5 rounded-full border border-border/40">
                    <User class="w-3.5 h-3.5" />
                    <span>{i18n.t('watchparty.join.host')}: <span class="text-foreground">{roomInfo.hostDisplayName}</span></span>
                </div>
            </div>
        </div>

        <form onsubmit={handleJoin} class="w-full space-y-6">
            {#if joinError}
                <div class="p-3 bg-destructive/10 border border-destructive/20 text-destructive text-xs rounded-xl font-bold text-center">
                    {i18n.t('watchparty.join.join_error')}
                </div>
            {/if}

            <div class="space-y-2.5">
                <Label class="text-sm font-bold pl-1">{i18n.t('watchparty.join.your_name')}</Label>
                <Input
                        bind:value={displayName}
                        placeholder={i18n.t('watchparty.join.your_name_placeholder')}
                        required
                        disabled={isJoining}
                        class="h-12 rounded-xl bg-background/50 border-border/50 focus-visible:ring-primary/40 text-base"
                        autofocus
                />
            </div>

            {#if roomInfo.hasPassword}
                <div class="space-y-2.5">
                    <Label class="text-sm font-bold pl-1 flex items-center gap-2">
                        <Lock class="w-3.5 h-3.5" /> {i18n.t('watchparty.join.room_password')}
                    </Label>
                    <Input
                            type="password"
                            bind:value={password}
                            placeholder="••••••••"
                            required
                            disabled={isJoining}
                            class="h-12 rounded-xl bg-background/50 border-border/50 focus-visible:ring-primary/40 text-base"
                    />
                </div>
            {/if}

            <Button
                    type="submit"
                    size="lg"
                    class="w-full h-12 rounded-xl font-bold text-base shadow-lg shadow-primary/10 mt-2 transition-transform active:scale-[0.98]"
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

    <div class="mt-8 pt-6 border-t border-border/30 w-full flex justify-center">
        <LanguageSelector
                compact={false}
                class="w-44 h-10 bg-transparent hover:bg-muted/50 border-none shadow-none text-muted-foreground hover:text-foreground font-semibold rounded-xl transition-colors"
        />
    </div>
</div>