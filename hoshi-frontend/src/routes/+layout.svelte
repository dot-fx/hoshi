<script lang="ts">
    import './layout.css';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { fade, slide } from 'svelte/transition';

    import { auth } from '$lib/auth.svelte';
    import { Toaster } from '$lib/components/ui/sonner';

    import TauriTitleBar from '$lib/components/layout/TauriTitleBar.svelte';
    import DesktopSidebar from '$lib/components/layout/DesktopSidebar.svelte';
    import MobileTopBar from '$lib/components/layout/MobileTopBar.svelte';
    import MobileBottomNav from '$lib/components/layout/MobileBottomNav.svelte';
    import { i18n } from '$lib/i18n/index.svelte';

    import { Search, Home, Calendar, User, Settings, ShoppingBag } from 'lucide-svelte';

    let { children } = $props();

    const mainRoutes = $derived([
        { name: i18n.t('home'), path: '/home', icon: Home },
        { name: i18n.t('search'), path: '/search', icon: Search },
        { name: i18n.t('schedule'), path: '/schedule', icon: Calendar }
    ]);

    const profileRoutes = $derived([
        { name: i18n.t('profile'), path: '/profile', icon: User },
        { name: i18n.t('marketplace'), path: '/marketplace', icon: ShoppingBag },
        { name: i18n.t('settings'), path: '/settings', icon: Settings },
    ]);

    onMount(() => {
        auth.restore();
    });

    const pathname = $derived(page.url.pathname);

    const isViewer = $derived(
        pathname.includes('/watch/') ||
        pathname.includes('/read/') ||
        pathname.includes('/read-novel/')
    );

    const showNav = $derived(
        auth.user !== null && pathname !== '/' && !isViewer
    );

    const pageTitle = $derived(() => {
        if (pathname.startsWith('/home')) return i18n.t('home');
        if (pathname.startsWith('/search')) return i18n.t('search');
        if (pathname.startsWith('/schedule')) return i18n.t('schedule');
        if (pathname.startsWith('/profile')) return i18n.t('profile');
        if (pathname.startsWith('/settings')) return i18n.t('settings');
        if (pathname.startsWith('/marketplace')) return i18n.t('marketplace');
        if (pathname.includes('/content/')) return i18n.t('details');
        if (isViewer) return i18n.t('reader');
        return 'Hoshi';
    });

    $effect(() => {
        if (!auth.initialized) return;
        const isRoot = pathname === '/';

        if (!auth.user && !isRoot) {
            goto('/');
        } else if (auth.user && isRoot) {
            goto('/home');
        }
    });
</script>

<div class="h-screen w-full bg-background text-foreground flex flex-col overflow-hidden">

    <TauriTitleBar title={pageTitle()} />

    <div class="flex flex-1 overflow-hidden relative">

        {#if showNav}
            <div transition:slide={{axis: 'x', duration: 300}} class="h-full z-40">
                <DesktopSidebar {mainRoutes} {profileRoutes} />
            </div>
        {/if}

        <div class="flex-1 flex flex-col relative overflow-hidden bg-background">

            {#if showNav}
                <div transition:slide={{axis: 'y', duration: 300}} class="w-full z-40 md:hidden">
                    <MobileTopBar title={pageTitle()} {profileRoutes} />
                </div>
            {/if}

            <main class="flex-1 overflow-y-auto relative w-full h-full">
                {@render children()}
            </main>

            {#if showNav}
                <div transition:slide={{axis: 'y', duration: 300}} class="w-full z-40 md:hidden relative">
                    <MobileBottomNav routes={[...mainRoutes, profileRoutes[0]]} />
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