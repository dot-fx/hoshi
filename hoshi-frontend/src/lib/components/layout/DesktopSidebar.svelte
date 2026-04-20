<script lang="ts">
    import { auth } from '@/stores/auth.svelte.js';
    import { Users } from 'lucide-svelte';
    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import { i18n } from '$lib/i18n/index.svelte';
    import {page} from "$app/state";

    let { mainRoutes, profileRoutes, showSwitchProfileModal = $bindable(false) } = $props();

    function isActive(path: string) {
        return path === '/'
            ? page.url.pathname === '/'
            : page.url.pathname.startsWith(path);
    }
</script>

<aside
        class="hidden md:flex flex-col h-full shrink-0 bg-transparent w-20 pt-10 pb-4 z-50"
>
    <div class="h-14 flex items-center px-4 mb-4 justify-center">
        <div class="h-9 w-9 shrink-0 rounded-2xl bg-primary/10 flex items-center justify-center text-primary font-bold shadow-sm">
            H
        </div>
    </div>

    <nav class="flex-1 overflow-y-auto py-2 px-4 space-y-6 scrollbar-hide">
        <div class="space-y-1">
            {#each mainRoutes as route}
                {@const Icon = route.icon}
                <a href={route.path} class="block" title={route.name}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-2xl transition-all duration-300 justify-center px-0
                        {isActive(route.path) ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5 {isActive(route.path) ? 'opacity-100' : 'opacity-70'}" />
                    </Button>
                </a>
            {/each}
        </div>

        <div class="space-y-1">
            {#each profileRoutes as route}
                {@const Icon = route.icon}
                <a href={route.path} class="block" title={route.name}>
                    <Button
                            variant="ghost"
                            class="w-full h-11 rounded-2xl transition-all duration-300 justify-center px-0
                        {isActive(route.path) ? 'bg-primary/10 text-primary font-semibold' : 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
                    >
                        <Icon class="shrink-0 size-5 {isActive(route.path) ? 'opacity-100' : 'opacity-70'}" />
                    </Button>
                </a>
            {/each}
        </div>
    </nav>

    <div class="px-4 shrink-0 mt-2">
        {#if auth.user}
            <div class="flex flex-col gap-3 justify-center px-2 py-2">
                <div class="flex items-center justify-center">
                    <Avatar.Root class="size-10 shrink-0 border-none shadow-sm">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} />
                        <Avatar.Fallback class="bg-primary/5 text-primary text-xs font-bold">
                            {auth.user.username[0].toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                </div>

                <Button
                        variant="ghost"
                        size="icon"
                        class="size-8 rounded-full text-muted-foreground hover:bg-muted/60 hover:text-foreground mx-auto"
                        onclick={(e) => {
                        e.stopPropagation();
                        showSwitchProfileModal = true;
                    }}
                        title={i18n.t('layout.switch_profile')}
                >
                    <Users class="size-4 shrink-0" />
                </Button>
            </div>
        {/if}
    </div>
</aside>