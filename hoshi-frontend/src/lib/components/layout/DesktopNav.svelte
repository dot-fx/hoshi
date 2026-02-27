<script lang="ts">
    import { page } from '$app/state';
    import { quintOut } from 'svelte/easing';
    import { crossfade } from 'svelte/transition';
    import { auth } from '$lib/auth.svelte';

    import { Bell, User, Settings, ShoppingBag, LogOut } from 'lucide-svelte';

    import { Button } from '$lib/components/ui/button';
    import * as Avatar from '$lib/components/ui/avatar';
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";

    const [send, receive] = crossfade({
        duration: 400,
        easing: quintOut
    });

    let { routes }: { routes: Array<{ name: string, path: string, icon: any }> } = $props();

    const isActive = $derived((path: string) =>
        path === '/' ?
            page.url.pathname === '/' : page.url.pathname.startsWith(path)
    );
</script>

<header class="hidden md:flex sticky top-0 z-50 w-full items-center justify-between border-b border-border bg-background/80 px-6 py-3 backdrop-blur-lg">
    <div class="flex items-center gap-2">
        <div class="h-8 w-8 rounded-full bg-primary/20 flex items-center justify-center text-primary font-bold">H</div>
        <span class="text-xl font-bold tracking-tight text-foreground">hoshi</span>
    </div>

    <nav class="relative flex items-center gap-1 bg-muted/50 p-1 rounded-full border border-border">
        {#each routes as route (route.path)}
            <a
                    href={route.path}
                    class="relative z-10 px-4 py-1.5 text-sm font-medium transition-colors duration-200
                {isActive(route.path) ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'}"
            >
                {route.name}
                {#if isActive(route.path)}
                    <div
                            in:receive={{ key: 'active-pill' }}
                            out:send={{ key: 'active-pill' }}
                            class="absolute inset-0 -z-10 rounded-full bg-background shadow-sm border border-border/50"
                    ></div>
                {/if}
            </a>
        {/each}
    </nav>

    <div class="flex items-center gap-4">
        <Button variant="ghost" size="icon" class="text-muted-foreground hover:text-foreground rounded-full">
            <Bell class="h-5 w-5" />
        </Button>

        {#if auth.user}
            <DropdownMenu.Root>
                <DropdownMenu.Trigger class="focus:outline-none">
                    <Avatar.Root class="h-9 w-9 border border-border cursor-pointer hover:border-foreground/50 transition-colors">
                        <Avatar.Image src={auth.user.avatar} alt={auth.user.username} />
                        <Avatar.Fallback class="bg-muted text-muted-foreground">
                            {auth.user.username[0].toUpperCase()}
                        </Avatar.Fallback>
                    </Avatar.Root>
                </DropdownMenu.Trigger>

                <DropdownMenu.Content align="end" class="w-56 bg-popover border-border text-popover-foreground">
                    <DropdownMenu.Label class="font-normal">
                        <div class="flex flex-col space-y-1">
                            <p class="text-sm font-medium leading-none text-foreground">{auth.user.username}</p>
                        </div>
                    </DropdownMenu.Label>
                    <DropdownMenu.Separator class="bg-border" />

                    <DropdownMenu.Item href="/profile" class="cursor-pointer focus:bg-accent focus:text-accent-foreground">
                        <User class="mr-2 h-4 w-4" />
                        <span>Profile</span>
                    </DropdownMenu.Item>

                    <DropdownMenu.Item href="/marketplace" class="cursor-pointer focus:bg-accent focus:text-accent-foreground">
                        <ShoppingBag class="mr-2 h-4 w-4" />
                        <span>Marketplace</span>
                    </DropdownMenu.Item>

                    <DropdownMenu.Item href="/settings" class="cursor-pointer focus:bg-accent focus:text-accent-foreground">
                        <Settings class="mr-2 h-4 w-4" />
                        <span>Settings</span>
                    </DropdownMenu.Item>

                    <DropdownMenu.Separator class="bg-border" />

                    <DropdownMenu.Item onclick={() => auth.logout()} class="text-destructive focus:text-destructive focus:bg-destructive/20 cursor-pointer">
                        <LogOut class="mr-2 h-4 w-4" />
                        <span>Log out</span>
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>
        {:else}
            <Avatar.Root class="h-9 w-9 border border-border">
                <Avatar.Fallback class="bg-muted text-muted-foreground">?</Avatar.Fallback>
            </Avatar.Root>
        {/if}
    </div>
</header>