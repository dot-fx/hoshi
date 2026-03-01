<script lang="ts">
    import { page } from '$app/state';

    let { routes }: { routes: Array<{ name: string, path: string, icon: any }> } = $props();

    const isActive = $derived((path: string) =>
        path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path)
    );
</script>

<nav class="md:hidden fixed bottom-0 z-50 w-full border-t border-border bg-background/95 backdrop-blur-xl pb-safe">

    <div class="grid grid-cols-4 h-16">
        {#each routes as route}
            {@const Icon = route.icon}
            <a
                    href={route.path}
                    class="flex flex-col items-center justify-center gap-1 transition-colors duration-200
        {isActive(route.path) ? 'text-foreground bg-foreground/10' : 'text-muted-foreground hover:text-foreground'}"
            >
                <Icon
                        class="h-5 w-5 {isActive(route.path) ? 'opacity-80' : ''}"
                        stroke-width={isActive(route.path) ? 2.5 : 2}
                />

                <span class="text-[10px] font-medium">
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