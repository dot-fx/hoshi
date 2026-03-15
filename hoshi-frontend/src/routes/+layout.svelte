<script lang="ts">
    import './layout.css';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { slide } from 'svelte/transition';

    import { auth } from '$lib/auth.svelte';
    import { extensions } from '$lib/extensions.svelte';
    import { Toaster } from '$lib/components/ui/sonner';

    import TauriTitleBar from '$lib/components/layout/TauriTitleBar.svelte';
    import DesktopSidebar from '$lib/components/layout/DesktopSidebar.svelte';
    import MobileTopBar from '$lib/components/layout/MobileTopBar.svelte';
    import MobileBottomNav from '$lib/components/layout/MobileBottomNav.svelte';
    import { i18n } from '$lib/i18n/index.svelte';

    import { Search, Home, Calendar, Settings, ShoppingBag, List, Tv } from 'lucide-svelte';

    let { children } = $props();

    const mainRoutes = $derived([
        { name: i18n.t('layout.home'), path: '/home', icon: Home },
        { name: i18n.t('layout.search'), path: '/search', icon: Search },
        { name: i18n.t('layout.list'), path: '/list', icon: List },
        { name: i18n.t('layout.schedule'), path: '/schedule', icon: Calendar }
    ]);

    const profileRoutes = $derived([
        { name: i18n.t('layout.settings'), path: '/settings', icon: Settings },
        { name: i18n.t('watchparty.title'), path: '#watchparty', icon: Tv },
        { name: i18n.t('layout.marketplace'), path: '/marketplace', icon: ShoppingBag },
    ]);

    onMount(() => {
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
        pathname.startsWith('/watchparty/')
    );

    const showNav = $derived(
        auth.user !== null && pathname !== '/' && !isViewer
    );

    $effect(() => {
        if (!auth.initialized) return;
        const isRoot = pathname === '/';
        const isWatchparty = pathname.startsWith('/watchparty/');

        if (!auth.user && !isRoot && !isWatchparty) {
            goto('/');
        } else if (auth.user && isRoot) {
            goto('/home');
            extensions.load();
        }
    });

    $effect(() => {
        if (auth.initialized && !auth.user && extensions.initialized) {
            extensions.installed = [];
            extensions.initialized = false;
        }
    });
</script>

<div class="h-screen w-full bg-background text-foreground flex flex-col overflow-hidden">

    <TauriTitleBar />

    <div class="flex flex-1 overflow-hidden relative">

        {#if showNav}
            <div transition:slide={{axis: 'x', duration: 300}} class="h-full z-40">
                <DesktopSidebar {mainRoutes} {profileRoutes} />
            </div>
        {/if}

        <div class="flex-1 flex flex-col relative overflow-hidden bg-background">

            {#if showNav}
                <div transition:slide={{axis: 'y', duration: 300}} class="w-full z-40 md:hidden">
                    <MobileTopBar {profileRoutes} />
                </div>
            {/if}

            <main class="flex-1 overflow-y-auto relative w-full">
                {@render children()}
            </main>

            {#if showNav}
                <div transition:slide={{axis: 'y', duration: 300}} class="w-full z-40 md:hidden relative">
                    <MobileBottomNav routes={mainRoutes} />
                </div>
            {/if}

        </div>
    </div>

    <Toaster />
</div>

<style>
    :global(html, body) {
        overflow: hidden;
        height: 100%;
        width: 100%;
    }
</style>