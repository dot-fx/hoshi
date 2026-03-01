<script lang="ts">
    import { auth } from '$lib/auth.svelte';
    import { LogOut, User as UserIcon, Settings, ShoppingBag } from 'lucide-svelte';

    import * as Avatar from '$lib/components/ui/avatar';
    import * as Drawer from "$lib/components/ui/drawer";
    import { Button } from "$lib/components/ui/button";
</script>

<header class="md:hidden sticky top-0 z-50 flex items-center justify-between border-b border-border bg-background/90 px-4 py-3 backdrop-blur-md transition-all duration-300">

    <div class="flex items-center gap-2">
        <div class="h-6 w-6 rounded bg-primary/20 flex items-center justify-center text-primary text-xs font-bold">H</div>
        <span class="text-lg font-bold tracking-tight text-foreground">Hoshi</span>
    </div>

    {#if auth.user}
        <Drawer.Root>
            <Drawer.Trigger>
                <Avatar.Root class="h-8 w-8 border border-border transition-colors hover:border-foreground/50">
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                    <Avatar.Fallback class="bg-muted text-xs font-medium text-muted-foreground">
                        {auth.user.username[0].toUpperCase()}
                    </Avatar.Fallback>
                </Avatar.Root>
            </Drawer.Trigger>

            <Drawer.Content class="px-4 pb-8 pt-2">
                <Drawer.Header class="text-left px-0 pb-6 border-b border-border/40">
                    <Drawer.Title class="text-xl">Account</Drawer.Title>
                    <Drawer.Description>Logged in as {auth.user.username}</Drawer.Description>
                </Drawer.Header>

                <div class="flex flex-col gap-2 mt-6">
                    <Button variant="ghost" class="w-full justify-start h-14 text-lg" href="/profile">
                        <UserIcon class="mr-4 h-6 w-6 text-muted-foreground" /> Profile
                    </Button>
                    <Button variant="ghost" class="w-full justify-start h-14 text-lg" href="/marketplace">
                        <ShoppingBag class="mr-4 h-6 w-6 text-muted-foreground" /> Marketplace
                    </Button>
                    <Button variant="ghost" class="w-full justify-start h-14 text-lg" href="/settings">
                        <Settings class="mr-4 h-6 w-6 text-muted-foreground" /> Settings
                    </Button>

                    <div class="h-px w-full bg-border/40 my-2"></div>

                    <Button variant="ghost" class="w-full justify-start h-14 text-lg text-destructive hover:text-destructive hover:bg-destructive/10" onclick={() => auth.logout()}>
                        <LogOut class="mr-4 h-6 w-6" /> Logout
                    </Button>
                </div>
            </Drawer.Content>
        </Drawer.Root>
    {:else}
        <a href="/" class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">
            Login
        </a>
    {/if}
</header>