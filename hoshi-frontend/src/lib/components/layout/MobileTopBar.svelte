<script lang="ts">
    import { auth } from '$lib/auth.svelte';
    import {LogOut, ChevronLeft, Users} from 'lucide-svelte';
    import { i18n } from '$lib/i18n/index.svelte';
    import { goto } from '$app/navigation';
    import { layoutState } from '@/layout.svelte.js';
    import * as Avatar from '$lib/components/ui/avatar';
    import * as Drawer from '$lib/components/ui/drawer';
    import { Button } from '$lib/components/ui/button';

    import CreateRoom from '@/components/modals/CreateRoom.svelte';

    let { profileRoutes, showSwitchProfileModal = $bindable(false) } = $props();
    let drawerOpen = $state(false);
    let showWatchpartyModal = $state(false);

    function handleBack() {
        if (layoutState.backUrl) {
            goto(layoutState.backUrl);
        } else {
            window.history.back();
        }
    }
</script>

<header class="md:hidden sticky top-0 z-40 flex items-center justify-between border-b border-border bg-background/90 backdrop-blur-md px-4 pb-3 pt-safe gap-3 shrink-0">
    <div class="flex items-center gap-2 sm:gap-3 shrink-0">
        {#if layoutState.showBack}
            <Button variant="ghost" size="icon" class="h-8 w-8 shrink-0 -ml-2 rounded-full" onclick={handleBack}>
                <ChevronLeft class="size-5" />
            </Button>
        {:else}
            <div class="h-7 w-7 shrink-0 rounded bg-primary/20 flex items-center justify-center text-primary text-xs font-bold">
                H
            </div>
        {/if}

        {#if !layoutState.headerAction && layoutState.title}
            <span class="text-lg font-bold tracking-tight text-foreground capitalize truncate">
                {layoutState.title}
            </span>
        {/if}
    </div>

    <div class="flex-1 flex justify-end items-center min-w-0">
        {#if layoutState.headerAction}
            {@render layoutState.headerAction()}
        {/if}
    </div>

    {#if auth.user}
        <Drawer.Root bind:open={drawerOpen}>
            <Drawer.Trigger>
                <Avatar.Root class="size-8 border border-border hover:border-foreground/40 transition-colors">
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                    <Avatar.Fallback class="bg-muted text-xs font-medium text-muted-foreground">
                        {auth.user.username[0].toUpperCase()}
                    </Avatar.Fallback>
                </Avatar.Root>
            </Drawer.Trigger>
            <Drawer.Content class="px-4 pt-2 pb-8">
                <Drawer.Header class="text-left px-0 pb-6 border-b border-border/40">
                    <Drawer.Title class="text-xl">{i18n.t('layout.account')}</Drawer.Title>
                    <Drawer.Description>{i18n.t('layout.logged_as', {name: auth.user.username})}</Drawer.Description>
                </Drawer.Header>
                <div class="flex flex-col gap-2 mt-6">
                    {#each profileRoutes as route}
                        {@const Icon = route.icon}

                        {#if route.path === '#watchparty'}
                            <Button
                                    variant="ghost"
                                    class="w-full justify-start h-14 text-lg"
                                    onclick={(e) => {
                                    e.preventDefault();
                                    drawerOpen = false;
                                    showWatchpartyModal = true;
                                }}
                            >
                                <Icon class="mr-4 size-6 text-muted-foreground" />
                                {route.name}
                            </Button>
                        {:else}
                            <Button
                                    variant="ghost"
                                    class="w-full justify-start h-14 text-lg"
                                    href={route.path}
                                    onclick={() => drawerOpen = false}
                            >
                                <Icon class="mr-4 size-6 text-muted-foreground" />
                                {route.name}
                            </Button>
                        {/if}
                    {/each}
                    <div class="h-px w-full bg-border/40 my-2"></div>

                    <Button
                            variant="ghost"
                            class="w-full justify-start h-14 text-lg text-foreground hover:bg-muted/60"
                            onclick={() => {
        drawerOpen = false;
        showSwitchProfileModal = true;
    }}
                    >
                        <Users class="mr-4 size-6 text-muted-foreground" />
                        {i18n.t('layout.switch_profile')}
                    </Button>
                </div>
            </Drawer.Content>
        </Drawer.Root>
    {/if}
</header>
<CreateRoom bind:open={showWatchpartyModal} />

<style>
    .pt-safe {
        padding-top: calc(env(safe-area-inset-top) + 0.75rem);
    }
</style>