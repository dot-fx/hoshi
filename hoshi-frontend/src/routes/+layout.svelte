<script lang="ts">
    import './layout.css';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';

    import { auth } from '$lib/auth.svelte';

    import DesktopNav from '$lib/components/layout/DesktopNav.svelte';
    import MobileTopBar from '$lib/components/layout/MobileTopBar.svelte';
    import MobileBottomNav from '$lib/components/layout/MobileBottomNav.svelte';
    import { Toaster } from '$lib/components/ui/sonner';

    import { Search, Home, Image, Calendar } from 'lucide-svelte';

    let { children } = $props();

    const routes = [
        { name: 'Home', path: '/home', icon: Home },
        { name: 'Gallery', path: '/booru', icon: Image },
        { name: 'Search', path: '/search', icon: Search },
        { name: 'Schedule', path: '/schedule', icon: Calendar }
    ];

    onMount(() => {
        auth.restore();
    });

    const pathname = $derived(page.url.pathname);

    const isPlayer = $derived(/^\/anime\/[^/]+\/\d+/.test(pathname));

    const showNav = $derived(
        auth.user !== null && pathname !== '/' && !isPlayer
    );

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

<div class="min-h-screen bg-background text-foreground flex flex-col">
    {#if showNav}
        <div class="sticky top-0 z-50 w-full">
            <DesktopNav {routes} />
            <MobileTopBar />
        </div>
    {/if}

    <main class="flex-1 w-full flex flex-col relative z-0">
        {@render children()}
    </main>

    {#if showNav}
        <div class="relative z-50">
            <MobileBottomNav {routes} />
        </div>
    {/if}

    <Toaster />
</div>