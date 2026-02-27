<script lang="ts">
    import { auth } from '$lib/auth.svelte';
    import { LogOut, User as UserIcon, Settings, ShoppingBag } from 'lucide-svelte';

    import * as Avatar from '$lib/components/ui/avatar';
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
</script>

<header class="md:hidden sticky top-0 z-50 flex items-center justify-between border-b border-border bg-background/90 px-4 py-3 backdrop-blur-md transition-all duration-300">

    <div class="flex items-center gap-2">
        <div class="h-6 w-6 rounded bg-primary/20 flex items-center justify-center text-primary text-xs font-bold">H</div>
        <span class="text-lg font-bold tracking-tight text-foreground">Hoshi</span>
    </div>

    {#if auth.user}
        <DropdownMenu.Root>
            <DropdownMenu.Trigger class="focus:outline-none">
                <Avatar.Root class="h-8 w-8 border border-border transition-colors hover:border-foreground/50">
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                    <Avatar.Fallback class="bg-muted text-xs font-medium text-muted-foreground">
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

                <DropdownMenu.Item href="/profile" class="focus:bg-accent focus:text-accent-foreground cursor-pointer">
                    <UserIcon class="mr-2 h-4 w-4" />
                    <span>Profile</span>
                </DropdownMenu.Item>

                <DropdownMenu.Item href="/marketplace" class="focus:bg-accent focus:text-accent-foreground cursor-pointer">
                    <ShoppingBag class="mr-2 h-4 w-4" />
                    <span>Marketplace</span>
                </DropdownMenu.Item>

                <DropdownMenu.Item href="/settings" class="focus:bg-accent focus:text-accent-foreground cursor-pointer">
                    <Settings class="mr-2 h-4 w-4" />
                    <span>Settings</span>
                </DropdownMenu.Item>

                <DropdownMenu.Separator class="bg-border" />

                <DropdownMenu.Item onclick={() => auth.logout()} class="text-destructive focus:text-destructive focus:bg-destructive/20 cursor-pointer">
                    <LogOut class="mr-2 h-4 w-4" />
                    <span>Logout</span>
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    {:else}
        <a href="/" class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors">
            Login
        </a>
    {/if}

</header>