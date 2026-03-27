<script lang="ts">
    import { page } from '$app/state';
    import { goto } from '$app/navigation';
    import { slide } from 'svelte/transition';

    import { WatchPartySocket } from '@/api/watchparty/ws';
    import type { RoomSnapshot, ServerEvent, PlaylistItem } from '@/api/watchparty/types';
    import { auth } from '$lib/auth.svelte';
    import {
        getProxiedVideoUrl,
        getBaseWsUrl,
        resolveAndEmitSource
    } from '@/api/watchparty/helpers';
    import JoinRoom from '@/components/watchparty/JoinRoom.svelte';
    import Player from '@/components/Player.svelte';
    import { Button } from '@/components/ui/button';
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import AddContentModal from "@/components/watchparty/AddContentModal.svelte";
    import Chat from "@/components/watchparty/Chat.svelte";
    import Queue from "@/components/watchparty/Queue.svelte";
    import HostControls from "@/components/watchparty/HostControls.svelte";
    import TerminateWatchparty from '@/components/watchparty/TerminateWatchParty.svelte';
    import { AlertCircle, Users, Search, PlaySquare, PanelRightClose, PanelRightOpen, Link, Check } from 'lucide-svelte';
    import { Spinner } from "$lib/components/ui/spinner";
    import { i18n } from "@/i18n/index.svelte.js";
    import {layoutState} from "@/layout.svelte.js";
    import {proxyApi} from "@/api/proxy/proxy";
    import type { CoreError } from "@/api/client";

    const roomId = $derived(page.params.roomId as string);
    const remoteUrl = $derived(page.url.searchParams.get('remoteUrl'));
    const isRemote = $derived(!!remoteUrl);
    const hasValidLocalSession = $derived(!isRemote && !!auth.user);
    let socket: WatchPartySocket | null = null;
    let roomState = $state<RoomSnapshot | null>(null);
    let disconnectReason = $state<string | null>(null);
    let isHostClosing = $state(false);
    const tokenKey = $derived(isRemote ? `wp_guest_${roomId}` : `wp_token_${roomId}`);

    let token = $state<string | null>(
        typeof sessionStorage !== 'undefined' ? sessionStorage.getItem(tokenKey) : null
    );
    let lastHeartbeat = $state(0);
    let addContentOpen = $state(false);
    let activeTab = $state<'chat' | 'queue' | 'settings'>('chat');
    let isSidebarOpen = $state(true);
    let windowWidth = $state(0);

    let hostSettings = $state({
        extension: null as string | null,
        server: null as string | null,
        isDub: false
    });

    $effect(() => {
        layoutState.title = "Watchparty";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    let currentItem = $state<PlaylistItem | null>(null);

    let proxiedVideoUrl = $derived(getProxiedVideoUrl(roomState?.currentItem?.source));
    let proxiedSubtitles = $state<any[]>([]);

    let lastProcessedSourceUrl = $state<string | null>(null);
    let subtitleBlobUrls: string[] = [];

    $effect(() => {
        const source = roomState?.currentItem?.source;
        const sourceUrl = source?.url || null;

        if (source && sourceUrl !== lastProcessedSourceUrl) {
            lastProcessedSourceUrl = sourceUrl;

            subtitleBlobUrls.forEach(URL.revokeObjectURL);
            subtitleBlobUrls = [];

            Promise.all(
                (source.subtitles ?? []).map(async (s: any) => {
                    const proxyParams = { url: s.url, ...(source.headers || {}) };

                    try {
                        const blob = await proxyApi.fetch(proxyParams);
                        const isAss = s.url.toLowerCase().endsWith('.ass') || s.url.toLowerCase().endsWith('.ssa');

                        let finalBlob = blob;
                        if (isAss) {
                            const textData = await blob.text();
                            const vttText = convertAssToVtt(textData);
                            finalBlob = new Blob([vttText], { type: 'text/vtt' });
                        }

                        const blobUrl = URL.createObjectURL(finalBlob);
                        subtitleBlobUrls.push(blobUrl);

                        return {
                            ...s,
                            url: blobUrl,
                            type: 'vtt',
                            language: s.language || s.label || "Unknown"
                        };
                    } catch (err) {
                        console.error(`[Watchparty] Error cargando subtítulo ${s.language}:`, err);
                        return null;
                    }
                })
            ).then(subs => {
                proxiedSubtitles = subs.filter(s => s !== null);
            });
        } else if (!source && lastProcessedSourceUrl !== null) {
            proxiedSubtitles = [];
            lastProcessedSourceUrl = null;
            subtitleBlobUrls.forEach(URL.revokeObjectURL);
            subtitleBlobUrls = [];
        }
    });

    let copiedUrl = $state(false);

    function handleCopyPublicUrl() {
        if (roomState?.publicUrl) {
            const baseUrl = roomState.publicUrl.replace(/\/$/, '');
            const fullUrl = `${baseUrl}/watchparty/${roomId}`;

            navigator.clipboard.writeText(fullUrl);
            copiedUrl = true;

            setTimeout(() => {
                copiedUrl = false;
            }, 2000);
        }
    }

    function handleReconnect() {
        if (!auth.user) {
            sessionStorage.removeItem(`wp_token_${roomId}`);
            token = null;
        }
        disconnectReason = null;
    }

    function connectToRoom() {
        if (socket) return;
        const wsBaseUrl = remoteUrl ? remoteUrl.replace(/^http/, 'ws') : getBaseWsUrl();
        socket = new WatchPartySocket({
            baseUrl: wsBaseUrl,
            roomId,
            token: token ?? null,
            onEvent: handleServerEvent,
        });
        socket.connect();
    }

    function formatAssTime(assTime: string) {
        let [hms, msPart = "00"] = assTime.trim().split('.');
        let [h, m, s] = hms.split(':');
        return `${h.padStart(2, '0')}:${m.padStart(2, '0')}:${s.padStart(2, '0')}.${msPart.padEnd(3, '0').substring(0, 3)}`;
    }

    function convertAssToVtt(assData: string) {
        const lines = assData.split(/\r?\n/);
        let vtt = "WEBVTT\n\n";
        let isEvents = false;
        let format: string[] = [];
        for (let line of lines) {
            line = line.trim();
            if (line === "[Events]") { isEvents = true; continue; }
            if (!isEvents) continue;
            if (line.startsWith("Format:")) {
                format = line.substring(7).split(",").map(s => s.trim());
                continue;
            }

            if (line.startsWith("Dialogue:")) {
                const parts = line.substring(9).split(",");
                const startIdx = format.indexOf("Start");
                const endIdx = format.indexOf("End");
                const textIdx = format.indexOf("Text");
                if (startIdx === -1 || endIdx === -1 || textIdx === -1) continue;

                const start = formatAssTime(parts[startIdx]);
                const end = formatAssTime(parts[endIdx]);
                let text = parts.slice(textIdx).join(",");
                text = text.replace(/\{[^}]+\}/g, "").replace(/\\N/gi, "\n");

                vtt += `${start} --> ${end}\n${text}\n\n`;
            }
        }
        return vtt;
    }

    $effect(() => {
        if (!auth.initialized || disconnectReason || socket) return;
        if (token || hasValidLocalSession) {
            connectToRoom();
        }
        return () => {
            socket?.close();
            socket = null;
        };
    });

    function handleResolveSource() {
        if (auth.user && currentItem && socket) {
            resolveAndEmitSource(currentItem, hostSettings, socket);
        }
    }

    function handleServerEvent(event: ServerEvent) {
        switch (event.event) {
            case 'room_state':
                roomState = event.data;
                currentItem = event.data.currentItem ?? null;
                if (currentItem) {
                    if (event.data.currentSource) roomState.currentItem.source = event.data.currentSource;
                    else if (!roomState.currentItem.source) handleResolveSource();
                }
                break;
            case 'track_changed':
                currentItem = event.data.item;
                if (roomState) {
                    roomState.currentItem = event.data.item;
                    roomState.videoState = event.data.videoState;
                }
                handleResolveSource();
                break;
            case 'source_resolved':
                if (roomState?.currentItem) {
                    roomState.currentItem.source = event.data.source;
                    roomState.videoState = event.data.videoState;
                }
                break;
            case 'video_sync':
                if (roomState) roomState.videoState = event.data;
                break;
            case 'queue_updated':
                if (roomState) roomState.queue = event.data;
                break;
            case 'chat_message':
                if (roomState) roomState.chatHistory = [...roomState.chatHistory, event.data];
                break;
            case 'member_joined':
                if (roomState) roomState.members = [...roomState.members, event.data];
                break;
            case 'member_left':
                if (roomState) roomState.members = roomState.members.filter(m => m.userId !== event.data.userId);
                break;
            case 'room_closed':
                disconnectReason = event.data.reason;
                socket = null;
                break;
        }
    }

    async function handleCloseRoom() {
        if (!auth.user) return;
        isHostClosing = true;
        socket?.close();
        await goto('/');
    }
</script>

<svelte:head>
    <title>{i18n.t('watchparty.title')} - {roomState?.name || 'Anime'}</title>
</svelte:head>

<svelte:window bind:innerWidth={windowWidth} />

<div class="absolute inset-0 bg-background flex flex-col overflow-hidden text-foreground">

    <AlertDialog.Root open={!!disconnectReason && !isHostClosing}>
        <AlertDialog.Content class="border-border/40 bg-card/95 backdrop-blur-xl outline-none max-w-sm">
            <AlertDialog.Header class="flex flex-col items-center text-center gap-1">
                <div class="p-4 bg-destructive/10 rounded-full mb-3">
                    <AlertCircle class="w-12 h-12 text-destructive" />
                </div>
                <AlertDialog.Title class="text-2xl font-black text-foreground">{i18n.t('watchparty.connection_lost')}</AlertDialog.Title>
                <AlertDialog.Description class="text-muted-foreground font-medium text-base">
                    {disconnectReason}
                </AlertDialog.Description>
            </AlertDialog.Header>
            <AlertDialog.Footer class="flex-col sm:flex-col gap-2 mt-4 space-x-0 sm:space-x-0">
                <Button class="w-full h-11 rounded-xl font-bold" onclick={handleReconnect}>
                    {i18n.t('watchparty.try_reconnect')}
                </Button>
                <Button variant="ghost" class="w-full h-11 rounded-xl font-bold text-muted-foreground hover:text-foreground" onclick={() => goto('/home')}>
                    {i18n.t('watchparty.back_to_home')}
                </Button>
            </AlertDialog.Footer>
        </AlertDialog.Content>
    </AlertDialog.Root>

    {#if !auth.initialized}
        <div class="m-auto flex flex-col items-center gap-5">
            <Spinner class="w-16 h-16 text-primary animate-spin" />
            <p class="text-muted-foreground font-bold text-lg animate-pulse">{i18n.t('watchparty.verifying_session')}</p>
        </div>

    {:else if !token && !hasValidLocalSession}
        <JoinRoom
                roomId={roomId}
                remoteUrl={remoteUrl}
                onJoined={(newToken) => {
                sessionStorage.setItem(`wp_token_${roomId}`, newToken);
                token = newToken;
            }}
        />

    {:else if !roomState}
        <div class="m-auto flex flex-col items-center gap-5">
            <div class="relative flex items-center justify-center">
                <Spinner class="w-16 h-16 text-primary animate-spin" />
                <div class="absolute w-3 h-3 bg-primary rounded-full animate-ping"></div>
            </div>
            <p class="text-muted-foreground font-bold text-lg animate-pulse">{i18n.t('watchparty.connecting_to_room')}</p>
        </div>

    {:else}
        <div class="flex-1 flex flex-col lg:flex-row w-full h-full overflow-hidden bg-background">

            <div class="flex-1 flex flex-col relative overflow-hidden bg-black/95">
                <div class="absolute top-0 inset-x-0 z-50 p-4 lg:p-6 bg-gradient-to-b from-black/80 via-black/40 to-transparent flex items-start justify-between pointer-events-none transition-opacity">
                    <div class="text-white drop-shadow-lg pointer-events-auto">
                        <h2 class="text-xl lg:text-2xl font-black tracking-tight line-clamp-1 flex items-center gap-2">
                            {#if roomState.currentItem}
                                <span>{roomState.currentItem.metadata?.seriesTitle || roomState.currentItem.title}</span>
                                {#if roomState.currentItem.metadata?.unitNumber}
                                    <span class="text-lg lg:text-xl text-white/70 font-bold shrink-0">
                                        - Ep {roomState.currentItem.metadata.unitNumber}
                                    </span>
                                {/if}
                            {:else}
                                {roomState.name}
                            {/if}
                        </h2>
                        <div class="flex items-center gap-2 mt-1.5 opacity-80 text-sm font-semibold">
                            <Users class="w-4 h-4" />
                            <span>{roomState.members.length} {i18n.t('watchparty.connected_count')}</span>
                        </div>
                    </div>

                    <div class="pointer-events-auto flex items-center gap-2">
                        {#if auth.user}
                            <TerminateWatchparty onTerminate={handleCloseRoom} />

                            <Button
                                    onclick={() => addContentOpen = true}
                                    variant="secondary"
                                    size="sm"
                                    class="bg-white/10 hover:bg-white/20 text-white border-none backdrop-blur-md font-bold rounded-lg shadow-lg"
                            >
                                <Search class="w-4 h-4 mr-2" /> {i18n.t('watchparty.add_content')}
                            </Button>
                        {/if}

                        {#if roomState.publicUrl}
                            <Button
                                    onclick={handleCopyPublicUrl}
                                    variant="secondary"
                                    size="sm"
                                    class="bg-white/10 hover:bg-white/20 text-white border-none backdrop-blur-md font-bold rounded-lg shadow-lg transition-all"
                            >
                                {#if copiedUrl}
                                    <Check class="w-4 h-4 mr-2 text-green-400" /> {i18n.t('watchparty.copied')}
                                {:else}
                                    <Link class="w-4 h-4 mr-2" /> {i18n.t('watchparty.invite')}
                                {/if}
                            </Button>
                        {/if}

                        <Button
                                variant="secondary"
                                size="icon"
                                class="hidden lg:inline-flex bg-white/10 hover:bg-white/20 text-white border-none backdrop-blur-md rounded-lg shadow-lg w-9 h-9"
                                onclick={() => isSidebarOpen = !isSidebarOpen}
                                title={isSidebarOpen ? i18n.t('watchparty.hide_sidebar') : i18n.t('watchparty.show_sidebar')}
                        >
                            {#if isSidebarOpen}
                                <PanelRightClose class="w-5 h-5" />
                            {:else}
                                <PanelRightOpen class="w-5 h-5" />
                            {/if}
                        </Button>
                    </div>

                    <AddContentModal
                            bind:open={addContentOpen}
                            onAdd={(item) => socket?.addToQueue(item)}
                    />
                </div>

                <div class="flex-1 flex items-center justify-center relative w-full h-full">
                    {#if roomState.currentItem?.source}
                        <Player
                                src={proxiedVideoUrl}
                                subtitles={proxiedSubtitles}
                                chapters={roomState.currentItem.source.chapters || []}
                                animeTitle={(roomState.currentItem?.metadata?.seriesTitle || roomState.currentItem?.title) ?? ''}
                                episodeTitle={roomState.currentItem?.metadata?.unitNumber ? `Episodio ${roomState.currentItem.metadata.unitNumber}` : 'Watchparty'}
                                cid={roomState.currentItem?.metadata?.contentId || 'wp-dummy'}
                                episode={roomState.currentItem?.metadata?.unitNumber || 1}
                                totalEpisodes={1}
                                isHost={!!auth.user}
                                syncState={roomState.videoState}
                                onPlay={() => socket?.play()}
                                onPause={() => socket?.pause()}
                                onSeek={(pos) => socket?.seek(pos)}
                                onTimeUpdate={(data) => {
                                if (auth.user && socket && roomState) {
                                    const now = Date.now();
                                    if (now - lastHeartbeat > 2000) {
                                        lastHeartbeat = now;
                                        socket.heartbeat(data.currentTime, data.paused ? 'paused' : 'playing');
                                    }
                                }
                            }}
                        />
                    {:else if roomState.currentItem}
                        <div class="text-white/40 text-center flex flex-col items-center gap-5 p-8 max-w-md">
                            <Spinner class="w-12 h-12 animate-spin text-primary" />
                            <p class="text-sm font-medium">{i18n.t('watchparty.loading_video_source')}</p>
                        </div>
                    {:else}
                        <div class="text-white/40 text-center flex flex-col items-center gap-5 p-8 max-w-md">
                            <div class="p-6 rounded-full bg-white/5 border border-white/10">
                                <PlaySquare class="w-16 h-16" />
                            </div>
                            <div>
                                <h3 class="text-2xl font-black text-white/80 mb-2">{i18n.t('watchparty.blank_screen')}</h3>
                                <p class="text-sm font-medium leading-relaxed">
                                    {#if auth.user}
                                        {i18n.t('watchparty.host_blank_screen_desc')}
                                    {:else}
                                        {i18n.t('watchparty.guest_blank_screen_desc')}
                                    {/if}
                                </p>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>

            {#if isSidebarOpen || windowWidth < 1024}
                <div transition:slide={{ duration: 300, axis: 'x' }} class="w-full lg:w-[400px] xl:w-[450px] h-[45vh] lg:h-full flex flex-col shrink-0 bg-card border-t lg:border-t-0 lg:border-l border-border/40 z-10 shadow-2xl relative">

                    <div class="flex p-2 gap-2 border-b border-border/40 bg-muted/5 z-10 shadow-sm">
                        <Button variant={activeTab === 'chat' ? 'default' : 'ghost'} class="flex-1 h-10 font-bold rounded-xl" onclick={() => activeTab = 'chat'}>
                            {i18n.t('watchparty.tab_chat')}
                        </Button>
                        <Button variant={activeTab === 'queue' ? 'default' : 'ghost'} class="flex-1 h-10 font-bold rounded-xl" onclick={() => activeTab = 'queue'}>
                            {i18n.t('watchparty.tab_queue')}
                        </Button>
                        {#if auth.user}
                            <Button variant={activeTab === 'settings' ? 'default' : 'ghost'} class="flex-1 h-10 font-bold rounded-xl" onclick={() => activeTab = 'settings'}>
                                {i18n.t('watchparty.tab_settings')}
                            </Button>
                        {/if}
                    </div>

                    <div class="flex-1 overflow-hidden relative">
                        <div class="absolute inset-0" class:hidden={activeTab !== 'chat'}>
                            <Chat chatHistory={roomState.chatHistory} onSendMessage={(text) => socket?.sendChat(text)} />
                        </div>

                        <div class="absolute inset-0" class:hidden={activeTab !== 'queue'}>
                            <Queue
                                    queue={roomState.queue}
                                    currentItem={roomState.currentItem}
                                    isHost={!!auth.user}
                                    onPlayItem={(id) => socket?.skipToItem(id)}
                                    onRemoveItem={(id) => socket?.removeFromQueue(id)}
                                    onReorder={(ids) => socket?.reorderQueue(ids)}
                            />
                        </div>

                        {#if auth.user}
                            <div class="absolute inset-0 overflow-y-auto" class:hidden={activeTab !== 'settings'}>
                                <HostControls
                                        onSettingsChange={(settings) => {
                                        hostSettings = settings;
                                        if (currentItem) handleResolveSource();
                                    }}
                                />
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>
    {/if}
</div>