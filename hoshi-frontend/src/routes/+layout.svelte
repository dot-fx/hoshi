<script lang="ts">
    import './layout.css';
    import { onMount } from 'svelte';
    import { goto, afterNavigate } from '$app/navigation';
    import { page } from '$app/state';
    import { slide } from 'svelte/transition';

    import { auth } from '$lib/auth.svelte';
    import { extensions } from '$lib/extensions.svelte';
    import { Toaster } from '$lib/components/ui/sonner';

    import TauriTitleBar from '$lib/components/layout/TauriTitleBar.svelte';
    import DesktopSidebar from '$lib/components/layout/DesktopSidebar.svelte';
    import MobileTopBar from '$lib/components/layout/MobileTopBar.svelte';
    import MobileBottomNav from '$lib/components/layout/MobileBottomNav.svelte';
    import SwitchProfile from '@/components/modals/SwitchProfile.svelte';
    import { i18n } from '$lib/i18n/index.svelte';
    import { Search, Home, Calendar, Settings, List, Tv } from 'lucide-svelte';
    import {discordApi} from "@/api/discord/discord";
    import {openUrl} from "@tauri-apps/plugin-opener";

    let { children } = $props();

    let innerWidth = $state(0);
    let isTouchDevice = $state(false);

    let isMobile = $derived(innerWidth < 1024 || isTouchDevice);

    const mainRoutes = $derived([
        { name: i18n.t('layout.home'), path: '/', icon: Home },
        { name: i18n.t('layout.search'), path: '/search', icon: Search },
        { name: i18n.t('layout.list'), path: '/list', icon: List },
        { name: i18n.t('layout.schedule'), path: '/schedule', icon: Calendar }
    ]);

    const profileRoutes = $derived([
        { name: i18n.t('layout.settings'), path: '/settings', icon: Settings },
        { name: i18n.t('watchparty.title'), path: '#watchparty', icon: Tv },
    ]);

    onMount(() => {
        isTouchDevice = window.matchMedia('(pointer: coarse)').matches;

        auth.restore().then(() => {
            if (auth.isAuthenticated) {
                extensions.load();
            }
        });
    });

    const pathname = $derived(page.url.pathname);

    const isViewer = $derived(
        pathname.startsWith('/watch/') ||
        pathname.startsWith('/read/') ||
        pathname.startsWith('/read-novel/') ||
        pathname.startsWith('/watchparty/') ||
        pathname.startsWith('/setup')
    );

    const showNav = $derived(
        auth.user !== null && !isViewer
    );

    let lastScrollY = $state(0);
    let isNavHidden = $state(false);
    let showSwitchProfileModal = $state(false);

    let mainElement: HTMLElement | null = $state(null);

    afterNavigate(() => {
        if (mainElement) {
            mainElement.scrollTo(0, 0);
        }
    });

    function handleScroll(e: Event) {
        if (!showNav || !isMobile) {
            if (isNavHidden) isNavHidden = false;
            return;
        }

        const target = e.target as HTMLElement;
        const currentScroll = target.scrollTop;

        if (currentScroll < 0) return;

        if (currentScroll > lastScrollY && currentScroll > 50) {
            isNavHidden = true;
        } else if (currentScroll < lastScrollY) {
            isNavHidden = false;
        }

        lastScrollY = currentScroll;
    }

    async function handleGlobalLinks(e: MouseEvent) {
        const target = e.target as HTMLElement;
        const anchor = target.closest('a');

        if (!anchor || !anchor.href) return;

        let url: URL;
        try {
            url = new URL(anchor.href);
        } catch (err) {
            return;
        }

        const isExternal = url.origin !== window.location.origin || url.protocol === 'mailto:';

        if (isExternal) {
            e.preventDefault();
            e.stopPropagation();

            try {
                // openUrl delega correctamente al sistema de Intents de Android y navegadores en Desktop
                await openUrl(anchor.href);
            } catch (err) {
                window.open(anchor.href, '_blank');
            }
        }
    }

    $effect(() => {
        if (!auth.initialized) return;

        const isWatchparty = pathname.startsWith('/watchparty/');
        const isSetup = pathname.startsWith('/setup');

        if (!auth.user && !isSetup && !isWatchparty) {
            goto('/setup');
        }
        else if (auth.user && isSetup) {
            goto('/');
        }

        if (auth.user) {
            extensions.load();
        }
    });

    $effect(() => {
        if (auth.initialized && !auth.user && extensions.initialized) {
            extensions.installed = [];
            extensions.initialized = false;
        }
    });

    $effect(() => {
        if (!auth.initialized || !auth.user) return;

        if (!isViewer) {
            discordApi.setActivity({
                title: "Hoshi",
                details: i18n.t('discord.browsing'),
                isVideo: false,
                isNsfw: false
            }).catch(() => {});
        }
    });

</script>

<svelte:window bind:innerWidth />
<svelte:document onclickcapture={handleGlobalLinks} />

<div class="h-dvh w-full bg-background text-foreground flex flex-col overflow-hidden">

    <TauriTitleBar />

    <div class="flex flex-1 overflow-hidden relative">

        {#if showNav}
            <div transition:slide={{axis: 'x', duration: 300}} class="h-full z-40 hidden lg:block">
                <DesktopSidebar {mainRoutes} {profileRoutes} bind:showSwitchProfileModal />
            </div>
        {/if}

        <div class="flex-1 flex flex-col relative overflow-hidden bg-background">

            {#if showNav}
                <div class="w-full z-40 lg:hidden absolute top-0 left-0 transition-transform duration-300 ease-in-out {isNavHidden ? '-translate-y-full' : 'translate-y-0'}">
                    <MobileTopBar {profileRoutes} bind:showSwitchProfileModal />
                </div>
            {/if}

            <main
                    bind:this={mainElement}
                    class="flex-1 relative w-full h-full {isViewer ? 'overflow-hidden' : 'overflow-y-auto overflow-x-hidden touch-pan-y'} {showNav ? 'pt-24 pb-20 lg:py-0' : ''}"
                    onscroll={handleScroll}
            >
                {@render children()}
            </main>

            {#if showNav}
                <div class="w-full z-40 lg:hidden absolute bottom-0 left-0 transition-transform duration-300 ease-in-out {isNavHidden ? 'translate-y-full' : 'translate-y-0'}">
                    <MobileBottomNav routes={mainRoutes} />
                </div>
            {/if}

        </div>
    </div>

    <Toaster />
    <SwitchProfile bind:open={showSwitchProfileModal} />
</div>

<style>
    :global(html, body) {
        overflow: hidden;
        height: 100%;
        width: 100%;
    }

    @media (hover: none) and (pointer: coarse) {
        :global(html, body) {
            touch-action: pan-x pan-y;
        }
        :global(main::-webkit-scrollbar) {
            display: none;
        }
        :global(main) {
            -ms-overflow-style: none;
            scrollbar-width: none;
        }
    }
</style>