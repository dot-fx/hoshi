<script lang="ts">
    import { page } from '$app/state';
    import { browser } from '$app/environment';
    import { auth } from '$lib/auth.svelte';
    import { LogOut, PanelLeftClose, PanelLeftOpen, LogIn } from 'lucide-svelte';
    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import { onMount } from 'svelte';
    import { i18n } from '$lib/i18n/index.svelte';

    let { mainRoutes, profileRoutes }: { mainRoutes: any[], profileRoutes: any[] } = $props();

    // 1. Leemos el localStorage SÍNCRONAMENTE si estamos en el cliente, antes de pintar
    let isCollapsed = $state(browser ? localStorage.getItem('sidebarCollapsed') === 'true' : false);

    // 2. Estado para controlar cuándo activar las animaciones
    let isMounted = $state(false);

    onMount(() => {
        // Un micro-retraso asegura que el navegador ya pintó el estado inicial cerrado
        // antes de que le agreguemos las clases de transición.
        setTimeout(() => isMounted = true, 50);
    });

    // 3. Guardar automáticamente (solo si ya montó para no sobreescribir falsos positivos)
    $effect(() => {
        if (browser && isMounted) {
            localStorage.setItem('sidebarCollapsed', String(isCollapsed));
        }
    });

    const isActive = $derived((path: string) =>
        path === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(path)
    );

    // 4. Esta variable contiene las clases de animación solo cuando ya cargó
    const t = $derived(isMounted ? 'transition-all duration-300' : '');
</script>

<aside class="hidden md:flex flex-col h-full shrink-0 bg-transparent {t} ease-in-out relative z-10 {isCollapsed ? 'w-24' : 'w-64'} pt-4 pb-4">

    <div class="h-14 flex items-center shrink-0 mb-4 {t} {isCollapsed ? 'justify-center px-0' : 'justify-between px-6'}">

        <div class="flex items-center overflow-hidden {t} {isCollapsed ? 'w-0 opacity-0 gap-0' : 'w-full opacity-100 gap-3'}">
            <div class="h-9 w-9 shrink-0 rounded-2xl bg-primary/10 flex items-center justify-center text-primary font-bold shadow-sm">H</div>
            <span class="text-2xl font-bold tracking-tight whitespace-nowrap text-foreground">Hoshi</span>
        </div>

        <button
                class="shrink-0 p-2 rounded-xl hover:bg-muted text-muted-foreground hover:text-foreground {t}"
                onclick={() => isCollapsed = !isCollapsed}
                aria-label="Alternar menú"
        >
            {#if isCollapsed}
                <PanelLeftOpen class="size-5" />
            {:else}
                <PanelLeftClose class="size-5" />
            {/if}
        </button>
    </div>

    <nav class="flex-1 overflow-y-auto py-2 px-4 space-y-6 scrollbar-hide">

        <div class="space-y-1">
            <div class="px-4 text-[10px] font-bold text-muted-foreground/60 uppercase tracking-widest whitespace-nowrap overflow-hidden {t} {isCollapsed ? 'h-0 opacity-0 pb-0' : 'h-auto opacity-100 pb-2'}">
                {i18n.t('menu')}
            </div>
            {#each mainRoutes as route}
                {@const Icon = route.icon}
                <a href={route.path} class="block" title={isCollapsed ? route.name : undefined}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-2xl {t} {isCollapsed ? 'justify-center px-0' : 'justify-start px-4'} {isActive(route.path) ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5 {t} {isCollapsed ? 'mr-0' : 'mr-3'} {isActive(route.path) ? 'opacity-100' : 'opacity-70'}" />
                        <span class="whitespace-nowrap overflow-hidden {t} {isCollapsed ? 'max-w-0 opacity-0' : 'max-w-[200px] opacity-100'}">
                            {route.name}
                        </span>
                    </Button>
                </a>
            {/each}
        </div>

        <div class="space-y-1">
            <div class="px-4 text-[10px] font-bold text-muted-foreground/60 uppercase tracking-widest whitespace-nowrap overflow-hidden {t} {isCollapsed ? 'h-0 opacity-0 pb-0' : 'h-auto opacity-100 pb-2'}">
                {i18n.t('account')}
            </div>
            {#each profileRoutes as route}
                {@const Icon = route.icon}
                <a href={route.path} class="block" title={isCollapsed ? route.name : undefined}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-2xl {t} {isCollapsed ? 'justify-center px-0' : 'justify-start px-4'} {isActive(route.path) ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5 {t} {isCollapsed ? 'mr-0' : 'mr-3'} {isActive(route.path) ? 'opacity-100' : 'opacity-70'}" />
                        <span class="whitespace-nowrap overflow-hidden {t} {isCollapsed ? 'max-w-0 opacity-0' : 'max-w-[200px] opacity-100'}">
                            {route.name}
                        </span>
                    </Button>
                </a>
            {/each}
        </div>
    </nav>

    <div class="px-4 shrink-0 mt-2">
        {#if auth.user}
            <div class="flex items-center {isCollapsed ? 'justify-center' : 'justify-between px-2'} py-2 rounded-3xl hover:bg-muted/50 {t} group cursor-default overflow-hidden">
                <div class="flex items-center overflow-hidden {t} {isCollapsed ? 'gap-0' : 'gap-3'}">
                    <Avatar.Root class="size-10 shrink-0 border-none shadow-sm">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} />
                        <Avatar.Fallback class="bg-primary/5 text-primary text-xs font-bold">{auth.user.username[0].toUpperCase()}</Avatar.Fallback>
                    </Avatar.Root>
                    <div class="flex flex-col truncate overflow-hidden {t} {isCollapsed ? 'max-w-0 opacity-0' : 'max-w-[120px] opacity-100'}">
                        <span class="text-sm font-semibold truncate text-foreground">{auth.user.username}</span>
                    </div>
                </div>

                <div class="overflow-hidden {t} {isCollapsed ? 'max-w-0 opacity-0' : 'max-w-[32px] opacity-100'}">
                    <Button variant="ghost" size="icon" class="size-8 rounded-full text-muted-foreground opacity-0 group-hover:opacity-100 hover:text-destructive hover:bg-destructive/10 {t} cursor-pointer" onclick={(e) => { e.stopPropagation(); auth.logout(); }}>
                        <LogOut class="size-4 shrink-0" />
                    </Button>
                </div>
            </div>
        {:else}
            <div class="px-1">
                <Button variant="default" class="w-full rounded-2xl shadow-sm {t} {isCollapsed ? 'px-0 justify-center' : ''}" href="/">
                    {#if isCollapsed}
                        <LogIn class="size-5 shrink-0" />
                    {:else}
                        <LogIn class="mr-2 size-4 shrink-0" /> <span class="whitespace-nowrap">{i18n.t('login')}</span>
                    {/if}
                </Button>
            </div>
        {/if}
    </div>
</aside>