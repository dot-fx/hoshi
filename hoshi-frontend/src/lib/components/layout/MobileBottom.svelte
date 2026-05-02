<script lang="ts">
    import { page } from '$app/state';
    import { fly } from 'svelte/transition';

    let { routes }: { routes: Array<{ name: string, path: string, icon: any, key?: string }> } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<nav class="lg:hidden fixed bottom-0 z-50 w-full pb-safe">
    <div class="mx-4 mb-3 rounded-2xl border border-border/30 bg-background/80 backdrop-blur-2xl shadow-[0_-2px_24px_rgba(0,0,0,0.15)] overflow-hidden">
        <div class="flex items-center h-14">
            {#each routes as route}
                {@const Icon = route.icon}
                {@const active = isActive(route.path)}


                <a href={route.path}
                class="relative flex flex-col items-center justify-center flex-1 h-full gap-0.5 transition-colors duration-200
                {active ? 'text-primary' : 'text-muted-foreground/60 hover:text-muted-foreground'}"
                >
                {#if active}
                    <div
                            class="absolute inset-x-2 inset-y-1.5 rounded-xl bg-primary/10"
                            in:fly={{ y: 4, duration: 200 }}
                    ></div>
                {/if}

                <div class="relative flex items-center justify-center">
                    <Icon
                            class="size-5 transition-all duration-200 {active ? 'scale-110' : 'scale-100'}"
                            stroke-width={active ? 2.5 : 1.75}
                    />
                </div>

                <!-- Label only shows when active -->
                {#if active}
                        <span class="relative text-[9px] font-semibold tracking-wide leading-none" in:fly={{ y: 3, duration: 150 }}>
                            {route.name}
                        </span>
                {/if}
                </a>
            {/each}
        </div>
    </div>
</nav>

<style>
    .pb-safe {
        padding-bottom: env(safe-area-inset-bottom, 0px);
    }
</style>