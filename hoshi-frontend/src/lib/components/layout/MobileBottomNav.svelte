<script lang="ts">
    import { page } from '$app/state';
    import { auth } from '$lib/auth.svelte';
    import * as Avatar from '$lib/components/ui/avatar';

    let { routes }: { routes: Array<{ name: string, path: string, icon: any, key?: string }> } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<nav class="md:hidden fixed bottom-0 z-50 w-full border-t border-border bg-background/95 backdrop-blur-xl pb-safe">

    <div class="flex items-center h-16">

        {#each routes as route}
            {@const Icon = route.icon}
            {@const active = isActive(route.path)}

            <a
                    href={route.path}
                    class="flex flex-col items-center justify-center gap-1 flex-1 transition-colors duration-200
                {active ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}"
            >

                <div class="flex items-center justify-center h-9 w-14 rounded-full transition-all duration-200
                    {active ? 'bg-primary/15 text-primary' : ''}"
                >

                    <Icon
                            class="size-5 {active ? 'opacity-100' : 'opacity-80'}"
                            stroke-width={active ? 2.5 : 2}
                    />

                </div>

                <span class="text-[10px] font-medium {active ? 'font-semibold' : ''}">
                    {route.name}
                </span>

            </a>
        {/each}
    </div>
</nav>

<style>
    .pb-safe {
        padding-bottom: env(safe-area-inset-bottom);
    }
</style>