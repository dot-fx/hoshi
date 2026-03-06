<script lang="ts">
    import { page } from '$app/state';
    import { auth } from '$lib/auth.svelte';
    import * as Avatar from '$lib/components/ui/avatar';
    import { i18n } from '$lib/i18n/index.svelte';

    let { routes }: { routes: Array<{ name: string, path: string, icon: any }> } = $props();

    const isActive = $derived((path: string) =>
        path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path)
    );
</script>

<nav class="md:hidden fixed bottom-0 z-50 w-full border-t border-border bg-background/95 backdrop-blur-xl pb-safe">

    <div class="flex justify-around items-center h-16 px-2">
        {#each routes as route}
            {@const Icon = route.icon}
            <a
                    href={route.path}
                    class="flex flex-col items-center justify-center gap-1 w-16 transition-colors duration-200
                {isActive(route.path) ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}"
            >
                <div class="flex items-center justify-center {isActive(route.path) ? 'bg-primary/15 text-primary' : ''} h-8 w-14 rounded-full transition-all duration-300">

                    {#if route.name === 'profile' && auth.user}
                        <Avatar.Root class="size-6 border transition-colors {isActive(route.path) ? 'border-primary' : 'border-transparent'}">
                            <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                            <Avatar.Fallback class="bg-muted text-[10px] font-medium text-muted-foreground">{auth.user.username[0].toUpperCase()}</Avatar.Fallback>
                        </Avatar.Root>
                    {:else}
                        <Icon
                                class="h-5 w-5 {isActive(route.path) ? 'opacity-100' : 'opacity-80'}"
                                stroke-width={isActive(route.path) ? 2.5 : 2}
                        />
                    {/if}

                </div>
                <span class="text-[10px] font-medium {isActive(route.path) ? 'font-semibold' : ''}">
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