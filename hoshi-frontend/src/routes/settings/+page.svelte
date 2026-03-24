<script lang="ts">
    import { auth } from "$lib/auth.svelte";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import {
        User, Link2, Settings, MonitorPlay, Puzzle, BookOpen, Bell, LayoutTemplate, Database
    } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import * as Avatar from "$lib/components/ui/avatar";
    import Account from "$lib/components/settings/Account.svelte";
    import Tracker from "$lib/components/settings/Tracker.svelte";
    import General from "$lib/components/settings/General.svelte";
    import UI from "$lib/components/settings/UI.svelte";
    import Content from "$lib/components/settings/Content.svelte";
    import Notifications from "$lib/components/settings/Notifications.svelte";
    import Extensions from "$lib/components/settings/Extensions.svelte";
    import Player from "$lib/components/settings/Player.svelte";
    import Readers from "$lib/components/settings/Readers.svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import { appConfig } from "@/config.svelte";
    import { layoutState } from '@/layout.svelte.js';
    import {i18n} from "@/i18n/index.svelte";

    $effect(() => {
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    let configSaving = $state(false);

    async function handleSaveConfig() {
        if (!appConfig.data) return;
        configSaving = true;
        try {
            await appConfig.update(appConfig.data);
            toast.success("Preferences updated");
        } catch (err) {
            console.error(err);
            toast.error("Failed to update preferences");
        } finally {
            configSaving = false;
        }
    }
</script>

<svelte:head>
    <title>{i18n.t('settings.title')}</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-12 pt-8 md:pt-12 px-4 md:px-8 lg:px-12 w-full max-w-[2000px] mx-auto space-y-8">

    <header class="flex flex-col md:flex-row md:items-center justify-between gap-6 border-b border-border/40 pb-8 w-full">
        <div class="flex items-center gap-5">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>

            <div class="space-y-0.5">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight">{auth.user?.username || 'Account'}</h1>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-70 uppercase tracking-wider">
                    {i18n.t('settings.preferences')}
                </p>
            </div>
        </div>
    </header>

    <section class="w-full">
        {#if !auth.user || !appConfig.data}
            <div in:fade class="h-[50vh] flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Spinner class="h-10 w-10 animate-spin text-primary" />
                <p class="text-sm font-bold animate-pulse">{i18n.t('settings.loading')}</p>
            </div>
        {:else}
            <div in:fade class="w-full">
                <Tabs.Root value="account" class="flex flex-col lg:flex-row gap-8 lg:gap-16 w-full items-start">
                    <Tabs.List class="flex flex-row lg:flex-col justify-start bg-transparent h-auto p-0 gap-1 w-full lg:w-64 shrink-0 overflow-x-auto hide-scrollbar lg:pr-4 pb-2 lg:pb-0 border-b lg:border-b-0 border-border/40">

                        <div class="hidden lg:block px-4 pt-2 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_profile', { defaultValue: 'Profile' })}
                        </div>
                        <Tabs.Trigger value="account" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <User class="h-4 w-4" /> {i18n.t('settings.account')}
                        </Tabs.Trigger>

                        <div class="hidden lg:block px-4 pt-6 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_application', { defaultValue: 'Application' })}
                        </div>
                        <Tabs.Trigger value="general" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Settings class="h-4 w-4" /> {i18n.t('settings.general')}
                        </Tabs.Trigger>
                        <Tabs.Trigger value="ui" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <LayoutTemplate class="h-4 w-4" /> {i18n.t('settings.interface')}
                        </Tabs.Trigger>
                        <Tabs.Trigger value="notifications" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Bell class="h-4 w-4" /> {i18n.t('settings.notifications')}
                        </Tabs.Trigger>

                        <div class="hidden lg:block px-4 pt-6 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_experience', { defaultValue: 'Media Experience' })}
                        </div>
                        <Tabs.Trigger value="player" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <MonitorPlay class="h-4 w-4" /> {i18n.t('settings.player')}
                        </Tabs.Trigger>
                        <Tabs.Trigger value="readers" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <BookOpen class="h-4 w-4" /> {i18n.t('settings.readers')}
                        </Tabs.Trigger>
                        <Tabs.Trigger value="content" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Database class="h-4 w-4" /> {i18n.t('settings.content')}
                        </Tabs.Trigger>

                        <div class="hidden lg:block px-4 pt-6 pb-2 text-[10px] font-black uppercase tracking-widest text-muted-foreground/50 text-left w-full">
                            {i18n.t('settings.section_integrations', { defaultValue: 'Integrations' })}
                        </div>
                        <Tabs.Trigger value="extensions" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Puzzle class="h-4 w-4" /> {i18n.t('settings.extensions')}
                        </Tabs.Trigger>
                        <Tabs.Trigger value="tracking" class="relative px-4 py-2.5 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Link2 class="h-4 w-4" /> {i18n.t('settings.tracking')}
                        </Tabs.Trigger>
                    </Tabs.List>

                    <div class="flex-1 min-w-0 w-full max-w-5xl space-y-16 pb-12">
                        <Tabs.Content value="account" class="focus-visible:outline-none mt-0 w-full">
                            <Account user={auth.user} onUpdate={() => auth.restore(true)} />
                        </Tabs.Content>

                        {#if appConfig.data}
                            <Tabs.Content value="general" class="focus-visible:outline-none mt-0 w-full">
                                <General bind:config={appConfig.data.general} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="ui" class="focus-visible:outline-none mt-0 w-full">
                                <UI bind:config={appConfig.data.ui} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="notifications" class="focus-visible:outline-none mt-0 w-full">
                                <Notifications bind:config={appConfig.data.notifications} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="player" class="focus-visible:outline-none mt-0 w-full">
                                <Player bind:config={appConfig.data.player} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="readers" class="focus-visible:outline-none mt-0 w-full">
                                <Readers bind:mangaConfig={appConfig.data.manga} bind:novelConfig={appConfig.data.novel} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="content" class="focus-visible:outline-none mt-0 w-full">
                                <Content bind:config={appConfig.data.content} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="extensions" class="focus-visible:outline-none mt-0 w-full">
                                <Extensions bind:config={appConfig.data.extensions} onSave={handleSaveConfig} />
                            </Tabs.Content>
                            <Tabs.Content value="tracking" class="focus-visible:outline-none mt-0 w-full">
                                <Tracker />
                            </Tabs.Content>
                        {/if}
                    </div>
                </Tabs.Root>
            </div>
        {/if}
    </section>
</main>