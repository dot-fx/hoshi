<script lang="ts">
    import { usersApi } from "@/api/users/users";
    import type { UserPrivate } from "@/api/users/types";
    import { toast } from "svelte-sonner";
    import AccountSettings from "$lib/components/settings/AccountSettings.svelte"; // <-- IMPORTA TU NUEVO COMPONENTE

    import * as Select from "$lib/components/ui/select";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { fade } from "svelte/transition";

    import {
        Loader2, User, Link2, Settings, Palette, MonitorPlay, Puzzle
    } from "lucide-svelte";
    import GeneralSettings from "@/components/settings/GeneralSettings.svelte";
    import TrackerSettings from "@/components/settings/TrackerSettings.svelte";

    let loading = $state(true);
    let user = $state<UserPrivate | null>(null);

    // --- ESTADOS DE CONFIGURACIÓN ---
    let configSaving = $state(false);

    // General
    let appLanguage = $state("en");
    let showAdultContent = $state(false);
    let blurAdultContent = $state(true);

    // Interface
    let theme = $state("system");
    let accentColor = $state("default");
    let reduceAnimations = $state(false);
    let uiFontSize = $state("medium");
    let sidebarCollapsed = $state(false);

    // Player
    let playerAutoNext = $state(true);
    let prefSubLang = $state("en");
    let prefAudioLang = $state("ja");
    let skipIntroAuto = $state(false);
    let skipOutroAuto = $state(false);
    let seekStep = $state("10");
    let resumeLastPos = $state(true);

    // Tracking
    let preferredTracker = $state("anilist");
    let autoUpdateProgress = $state(true);
    let syncTrackerStartup = $state(false);

    // Extensions
    let autoUpdateExt = $state(true);
    let extensionRepoUrls = $state("https://raw.githubusercontent.com/hoshi/extensions/main/index.json");

    $effect(() => { loadData(); });

    async function loadData() {
        loading = true;
        try {
            const userRes = await usersApi.getMe();
            user = userRes;
        } catch (error) {
            toast.error("Failed to load profile data");
        } finally {
            loading = false;
        }
    }

    async function handleSaveConfig() {
        configSaving = true;
        await new Promise(resolve => setTimeout(resolve, 400));
        toast.success("Preferences updated");
        configSaving = false;
    }
</script>

<svelte:head>
    <title>Settings - Hoshi</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-10 pt-6 md:pt-8 px-4 md:px-6 lg:px-8 xl:px-10 w-full max-w-[2400px] mx-auto space-y-8">

    <header class="flex flex-col md:flex-row md:items-center justify-between gap-5 border-b border-border/40 pb-6 w-full">
        <div class="space-y-1">
            <h1 class="text-3xl md:text-4xl font-black tracking-tight flex items-center gap-3">
                <Settings class="h-8 w-8 md:h-10 md:w-10 text-primary" />
                Settings
            </h1>
            <p class="text-sm md:text-base text-muted-foreground font-medium opacity-80">
                Manage your account, preferences, and playback settings.
            </p>
        </div>
    </header>

    <section class="w-full">
        {#if loading}
            <div in:fade class="h-[50vh] flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Loader2 class="h-10 w-10 animate-spin text-primary" />
                <p class="text-sm font-bold animate-pulse">Loading settings...</p>
            </div>
        {:else if user}
            <div in:fade class="w-full">

                <Tabs.Root value="account" class="flex flex-col lg:flex-row gap-8 lg:gap-16 w-full items-start">

                    <!-- SIDEBAR -->
                    <Tabs.List class="flex flex-row lg:flex-col justify-start bg-transparent h-auto p-0 gap-2 w-full lg:w-64 shrink-0 overflow-x-auto hide-scrollbar lg:pr-4 pb-2 lg:pb-0 border-b lg:border-b-0 border-border/40">
                        <Tabs.Trigger value="account" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <User class="h-4 w-4" /> Account
                        </Tabs.Trigger>
                        <Tabs.Trigger value="general" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Settings class="h-4 w-4" /> General
                        </Tabs.Trigger>
                        <Tabs.Trigger value="interface" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Palette class="h-4 w-4" /> Interface
                        </Tabs.Trigger>
                        <Tabs.Trigger value="player" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <MonitorPlay class="h-4 w-4" /> Player
                        </Tabs.Trigger>
                        <Tabs.Trigger value="tracking" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Link2 class="h-4 w-4" /> Tracking
                        </Tabs.Trigger>
                        <Tabs.Trigger value="extensions" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <Puzzle class="h-4 w-4" /> Extensions
                        </Tabs.Trigger>
                    </Tabs.List>

                    <!-- CONTENIDO DE LAS PESTAÑAS -->
                    <div class="flex-1 min-w-0 w-full max-w-5xl space-y-16 pb-12">

                        <!-- 1. ACCOUNT TAB -->
                        <Tabs.Content value="account" class="focus-visible:outline-none mt-0 w-full">
                            <AccountSettings {user} onUpdate={loadData} />
                        </Tabs.Content>

                        <Tabs.Content value="general" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                            <GeneralSettings
                                    bind:appLanguage
                                    bind:showAdultContent
                                    bind:blurAdultContent
                                    onSave={handleSaveConfig}
                            />
                        </Tabs.Content>

                        <!-- 3. INTERFACE TAB -->
                        <Tabs.Content value="interface" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                            <section>
                                <div class="mb-2">
                                    <h2 class="text-2xl font-bold tracking-tight">Interface</h2>
                                    <p class="text-sm text-muted-foreground mt-1">Customize the look and feel of the app.</p>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold">Theme</Label>
                                        <p class="text-sm text-muted-foreground">Choose your preferred visual theme.</p>
                                    </div>
                                    <Select.Root type="single" bind:value={theme} onValueChange={handleSaveConfig}>
                                        <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">{theme}</Select.Trigger>
                                        <Select.Content>
                                            <Select.Item value="system">System Default</Select.Item>
                                            <Select.Item value="light">Light</Select.Item>
                                            <Select.Item value="dark">Dark</Select.Item>
                                        </Select.Content>
                                    </Select.Root>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold">Accent Color</Label>
                                        <p class="text-sm text-muted-foreground">Change the primary color used across the app.</p>
                                    </div>
                                    <Select.Root type="single" bind:value={accentColor} onValueChange={handleSaveConfig}>
                                        <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">{accentColor}</Select.Trigger>
                                        <Select.Content>
                                            <Select.Item value="default">Default</Select.Item>
                                            <Select.Item value="blue">Blue</Select.Item>
                                            <Select.Item value="purple">Purple</Select.Item>
                                            <Select.Item value="green">Green</Select.Item>
                                        </Select.Content>
                                    </Select.Root>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="reduceAnimations">Reduce Animations</Label>
                                        <p class="text-sm text-muted-foreground">Minimize motion effects to improve performance or reduce motion sickness.</p>
                                    </div>
                                    <Switch id="reduceAnimations" bind:checked={reduceAnimations} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold">Font Size</Label>
                                        <p class="text-sm text-muted-foreground">Adjust the global text size of the interface.</p>
                                    </div>
                                    <Select.Root type="single" bind:value={uiFontSize} onValueChange={handleSaveConfig}>
                                        <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">{uiFontSize}</Select.Trigger>
                                        <Select.Content>
                                            <Select.Item value="small">Small</Select.Item>
                                            <Select.Item value="medium">Medium</Select.Item>
                                            <Select.Item value="large">Large</Select.Item>
                                        </Select.Content>
                                    </Select.Root>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="sidebarCollapsed">Sidebar Collapsed by Default (Desktop)</Label>
                                        <p class="text-sm text-muted-foreground">Keep the main navigation sidebar minimized on desktop.</p>
                                    </div>
                                    <Switch id="sidebarCollapsed" bind:checked={sidebarCollapsed} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>
                            </section>
                        </Tabs.Content>

                        <!-- 4. PLAYER TAB -->
                        <Tabs.Content value="player" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                            <section>
                                <div class="mb-2">
                                    <h2 class="text-2xl font-bold tracking-tight">Player</h2>
                                    <p class="text-sm text-muted-foreground mt-1">Configure your viewing experience.</p>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="playerAutoNext">Autoplay Next Episode</Label>
                                        <p class="text-sm text-muted-foreground">Automatically play the next episode when the current one finishes.</p>
                                    </div>
                                    <Switch id="playerAutoNext" bind:checked={playerAutoNext} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold">Preferred Subtitle Language</Label>
                                        <p class="text-sm text-muted-foreground">Default subtitle track to select if available.</p>
                                    </div>
                                    <Select.Root type="single" bind:value={prefSubLang} onValueChange={handleSaveConfig}>
                                        <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">
                                            {prefSubLang === 'en' ? 'English' : prefSubLang === 'es' ? 'Spanish' : 'None'}
                                        </Select.Trigger>
                                        <Select.Content>
                                            <Select.Item value="en">English</Select.Item>
                                            <Select.Item value="es">Spanish</Select.Item>
                                            <Select.Item value="none">None</Select.Item>
                                        </Select.Content>
                                    </Select.Root>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold">Preferred Audio Language</Label>
                                        <p class="text-sm text-muted-foreground">Default audio track to select if available (Dub/Sub).</p>
                                    </div>
                                    <Select.Root type="single" bind:value={prefAudioLang} onValueChange={handleSaveConfig}>
                                        <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">
                                            {prefAudioLang === 'ja' ? 'Japanese' : prefAudioLang === 'en' ? 'English' : 'Spanish'}
                                        </Select.Trigger>
                                        <Select.Content>
                                            <Select.Item value="ja">Japanese</Select.Item>
                                            <Select.Item value="en">English</Select.Item>
                                            <Select.Item value="es">Spanish</Select.Item>
                                        </Select.Content>
                                    </Select.Root>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="skipIntroAuto">Auto-Skip Intro</Label>
                                        <p class="text-sm text-muted-foreground">Automatically skip opening sequences when available.</p>
                                    </div>
                                    <Switch id="skipIntroAuto" bind:checked={skipIntroAuto} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="skipOutroAuto">Auto-Skip Outro</Label>
                                        <p class="text-sm text-muted-foreground">Automatically skip ending sequences when available.</p>
                                    </div>
                                    <Switch id="skipOutroAuto" bind:checked={skipOutroAuto} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold">Seek Step</Label>
                                        <p class="text-sm text-muted-foreground">Time to skip backward/forward using arrow keys.</p>
                                    </div>
                                    <Select.Root type="single" bind:value={seekStep} onValueChange={handleSaveConfig}>
                                        <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">{seekStep} seconds</Select.Trigger>
                                        <Select.Content>
                                            <Select.Item value="5">5 seconds</Select.Item>
                                            <Select.Item value="10">10 seconds</Select.Item>
                                            <Select.Item value="15">15 seconds</Select.Item>
                                            <Select.Item value="30">30 seconds</Select.Item>
                                        </Select.Content>
                                    </Select.Root>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="resumeLastPos">Resume from Last Position</Label>
                                        <p class="text-sm text-muted-foreground">Remember where you left off and resume playback automatically.</p>
                                    </div>
                                    <Switch id="resumeLastPos" bind:checked={resumeLastPos} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>
                            </section>
                        </Tabs.Content>

                        <!-- 5. TRACKING TAB -->
                        <Tabs.Content value="tracking" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                            <TrackerSettings/>
                        </Tabs.Content>

                        <!-- 6. EXTENSIONS TAB -->
                        <Tabs.Content value="extensions" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                            <section>
                                <div class="mb-2">
                                    <h2 class="text-2xl font-bold tracking-tight">Extensions</h2>
                                    <p class="text-sm text-muted-foreground mt-1">Manage your content sources and plugins.</p>
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4">
                                        <Label class="text-base font-bold" for="autoUpdateExt">Auto-Update Extensions</Label>
                                        <p class="text-sm text-muted-foreground">Automatically download and install updates for your active extensions in the background.</p>
                                    </div>
                                    <Switch id="autoUpdateExt" bind:checked={autoUpdateExt} onCheckedChange={handleSaveConfig} class="shrink-0" />
                                </div>

                                <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-4 py-6 border-b border-border/40">
                                    <div class="space-y-1 pr-4 flex-1">
                                        <Label class="text-base font-bold">Extension Repository URLs</Label>
                                        <p class="text-sm text-muted-foreground">Links to the repositories where extensions are hosted.</p>
                                    </div>
                                    <div class="w-full sm:max-w-md space-y-2">
                                        <Input bind:value={extensionRepoUrls} placeholder="https://..." class="rounded-xl h-11 w-full" />
                                        <div class="flex justify-end">
                                            <Button variant="secondary" size="sm" class="rounded-lg font-bold" onclick={handleSaveConfig}>
                                                Update URLs
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            </section>
                        </Tabs.Content>

                    </div>
                </Tabs.Root>
            </div>
        {/if}
    </section>
</main>