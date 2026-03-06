<script lang="ts">
    import { page } from '$app/state';
    import { browser } from '$app/environment';
    import { auth } from '$lib/auth.svelte';
    import { LogOut, PanelLeftClose, PanelLeftOpen, LogIn } from 'lucide-svelte';
    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import { i18n } from '$lib/i18n/index.svelte';

    let { mainRoutes, profileRoutes }: { mainRoutes: any[], profileRoutes: any[] } = $props();

    let isCollapsed = $state(browser ? localStorage.getItem('sidebarCollapsed') === 'true' : false);

    $effect(() => {
        if (browser) {
            localStorage.setItem('sidebarCollapsed', String(isCollapsed));
        }
    });

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<aside class="hidden md:flex flex-col h-full shrink-0 bg-transparent transition-[width] duration-300 ease-in-out {isCollapsed ? 'w-24' : 'w-64'} pt-4 pb-4">

    <!-- HEADER -->
    <div class="h-14 flex items-center px-4 mb-4 {isCollapsed ? 'justify-center' : 'justify-between'}">

        <!-- LOGO / TOGGLE -->
        <button
                onclick={() => isCollapsed = !isCollapsed}
                class="flex items-center gap-3 group"
                aria-label="Toggle menu"
        >
            <div class="h-9 w-9 shrink-0 rounded-2xl bg-primary/10 flex items-center justify-center text-primary font-bold shadow-sm group-hover:bg-primary/20 transition-colors">
                H
            </div>

            {#if !isCollapsed}
            <span class="text-2xl font-bold tracking-tight whitespace-nowrap text-foreground">
                Hoshi
            </span>
            {/if}
        </button>

        <!-- TOGGLE ICON SOLO CUANDO EXPANDIDO -->
        {#if !isCollapsed}
            <button
                    onclick={() => isCollapsed = !isCollapsed}
                    class="p-2 rounded-xl hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                    aria-label="Toggle menu"
            >
                <PanelLeftClose class="size-5" />
            </button>
        {/if}

    </div>

    <!-- NAV -->
    <nav class="flex-1 overflow-y-auto py-2 px-4 space-y-6 scrollbar-hide">

        <div class="space-y-1">

            {#if !isCollapsed}
                <div class="px-4 text-[10px] font-bold text-muted-foreground/60 uppercase tracking-widest pb-2">
                    {i18n.t('menu')}
                </div>
            {/if}

            {#each mainRoutes as route}
                {@const Icon = route.icon}

                <a href={route.path} class="block" title={isCollapsed ? route.name : undefined}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-2xl transition-colors {isCollapsed ? 'justify-center px-0' : 'justify-start px-4'} {isActive(route.path) ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5 {isCollapsed ? '' : 'mr-3'} {isActive(route.path) ? 'opacity-100' : 'opacity-70'}" />

                        {#if !isCollapsed}
                            <span class="whitespace-nowrap">
                                {route.name}
                            </span>
                        {/if}
                    </Button>
                </a>
            {/each}

        </div>

        <div class="space-y-1">

            {#if !isCollapsed}
                <div class="px-4 text-[10px] font-bold text-muted-foreground/60 uppercase tracking-widest pb-2">
                    {i18n.t('account')}
                </div>
            {/if}

            {#each profileRoutes as route}
                {@const Icon = route.icon}

                <a href={route.path} class="block" title={isCollapsed ? route.name : undefined}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-2xl transition-colors {isCollapsed ? 'justify-center px-0' : 'justify-start px-4'} {isActive(route.path) ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5 {isCollapsed ? '' : 'mr-3'} {isActive(route.path) ? 'opacity-100' : 'opacity-70'}" />

                        {#if !isCollapsed}
                            <span class="whitespace-nowrap">
                                {route.name}
                            </span>
                        {/if}
                    </Button>
                </a>
            {/each}

        </div>

    </nav>

    <!-- USER -->
    <div class="px-4 shrink-0 mt-2">

        {#if auth.user}

            <div class="flex items-center {isCollapsed ? 'justify-center' : 'justify-between px-2'} py-2 rounded-3xl hover:bg-muted/50 transition-colors group cursor-default">

                <div class="flex items-center {isCollapsed ? '' : 'gap-3'}">

                    <Avatar.Root class="size-10 shrink-0 border-none shadow-sm">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} />
                        <Avatar.Fallback class="bg-primary/5 text-primary text-xs font-bold">
                            {auth.user.username[0].toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>

                    {#if !isCollapsed}
                        <span class="text-sm font-semibold truncate text-foreground max-w-30">
                            {auth.user.username}
                        </span>
                    {/if}

                </div>

                {#if !isCollapsed}
                    <Button
                            variant="ghost"
                            size="icon"
                            class="size-8 rounded-full text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-destructive hover:bg-destructive/10 transition-colors"
                            onclick={(e) => {
                            e.stopPropagation();
                            auth.logout();
                        }}
                    >
                        <LogOut class="size-4 shrink-0" />
                    </Button>
                {/if}

            </div>

        {:else}

            <Button
                    variant="default"
                    class="w-full rounded-2xl shadow-sm {isCollapsed ? 'px-0 justify-center' : ''}"
                    href="/"
            >
                {#if isCollapsed}
                    <LogIn class="size-5 shrink-0" />
                {:else}
                    <LogIn class="mr-2 size-4 shrink-0" />
                    <span>{i18n.t('login')}</span>
                {/if}
            </Button>

        {/if}

    </div>

</aside>