<script lang="ts">
    import { auth } from '$lib/auth.svelte';
    import { LogOut } from 'lucide-svelte';
    import { i18n } from '$lib/i18n/index.svelte';

    import * as Avatar from '$lib/components/ui/avatar';
    import * as Drawer from '$lib/components/ui/drawer';
    import { Button } from '$lib/components/ui/button';

    let { title, profileRoutes }: { title: string, profileRoutes: any[] } = $props();
</script>

<header class="md:hidden sticky top-0 z-40 flex items-center justify-between border-b border-border bg-background/90 backdrop-blur-md px-4 pb-3 pt-safe shrink-0">
    <!-- LOGO + TITLE -->
    <div class="flex items-center gap-3">

        <div class="h-7 w-7 rounded bg-primary/20 flex items-center justify-center text-primary text-xs font-bold">
            H
        </div>

        <span class="text-lg font-bold tracking-tight text-foreground capitalize">
            {title}
        </span>

    </div>

    <!-- USER -->
    {#if auth.user}

        <Drawer.Root>

            <Drawer.Trigger>

                <Avatar.Root class="size-8 border border-border hover:border-foreground/40 transition-colors">

                    <Avatar.Image
                            src={auth.user.avatar}
                            alt={auth.user.username}
                            class="object-cover"
                    />

                    <Avatar.Fallback class="bg-muted text-xs font-medium text-muted-foreground">
                        {auth.user.username[0].toUpperCase()}
                    </Avatar.Fallback>

                </Avatar.Root>

            </Drawer.Trigger>

            <Drawer.Content class="px-4 pt-2 pb-8">

                <Drawer.Header class="text-left px-0 pb-6 border-b border-border/40">

                    <Drawer.Title class="text-xl">
                        {i18n.t('account')}
                    </Drawer.Title>

                    <Drawer.Description>
                        {i18n.t('logged_in_as')} {auth.user.username}
                    </Drawer.Description>

                </Drawer.Header>

                <div class="flex flex-col gap-2 mt-6">

                    {#each profileRoutes as route}
                        {@const Icon = route.icon}

                        <Button
                                variant="ghost"
                                class="w-full justify-start h-14 text-lg"
                                href={route.path}
                        >

                            <Icon class="mr-4 size-6 text-muted-foreground" />

                            {route.name}

                        </Button>

                    {/each}

                    <div class="h-px w-full bg-border/40 my-2"></div>

                    <Button
                            variant="ghost"
                            class="w-full justify-start h-14 text-lg text-destructive hover:text-destructive hover:bg-destructive/10"
                            onclick={() => auth.logout()}
                    >

                        <LogOut class="mr-4 size-6" />

                        {i18n.t('logout')}

                    </Button>

                </div>

            </Drawer.Content>

        </Drawer.Root>

    {:else}

        <Button variant="ghost" size="sm" href="/">
            {i18n.t('login')}
        </Button>

    {/if}

</header>

<style>
    .pt-safe {
        /* Adds the notch height PLUS your original 0.75rem (py-3) spacing */
        padding-top: calc(env(safe-area-inset-top) + 0.75rem);
    }
</style>