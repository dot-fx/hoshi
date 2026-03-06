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

    import { Search, Home, Calendar, User, Settings, ShoppingBag } from 'lucide-svelte';

    let { children } = $props();

    const mainRoutes = [
        { name: 'Home', path: '/home', icon: Home },
        { name: 'Search', path: '/search', icon: Search },
        { name: 'Schedule', path: '/schedule', icon: Calendar }
    ];

    const profileRoutes = [
        { name: 'Profile', path: '/profile', icon: User },
        { name: 'Marketplace', path: '/marketplace', icon: ShoppingBag },
        { name: 'Settings', path: '/settings', icon: Settings },
    ];

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
        if (pathname.startsWith('/home')) return 'Home';
        if (pathname.startsWith('/search')) return 'Search';
        if (pathname.startsWith('/schedule')) return 'Schedule';
        if (pathname.startsWith('/profile')) return 'Profile';
        if (pathname.startsWith('/settings')) return 'Settings';
        if (pathname.startsWith('/marketplace')) return 'Marketplace';
        if (pathname.includes('/content/')) return 'Details';
        if (isViewer) return 'Reader';
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