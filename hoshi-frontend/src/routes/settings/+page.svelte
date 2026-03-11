<script lang="ts">
    import { usersApi } from "@/api/users/users";
    import type { UserPrivate } from "@/api/users/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import {
        Loader2, User, Link2, Settings, Palette, MonitorPlay, Puzzle, BookOpen, BookOpenText
    } from "lucide-svelte";

    // --- COMPONENTES DE CONFIGURACIÓN ---
    import AccountSettings from "$lib/components/settings/AccountSettings.svelte";
    import GeneralSettings from "@/components/settings/GeneralSettings.svelte";
    import MangaReaderSettings from "@/components/settings/MangaReader.svelte";
    import NovelReaderSettings from "@/components/settings/NovelReader.svelte";
    import TrackerSettings from "@/components/settings/TrackerSettings.svelte";

    // --- UI COMPONENTS (SHADCN) ---
    import * as Select from "$lib/components/ui/select";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";

    // --- STORE GLOBAL ---
    import { appConfig } from "@/config.svelte";

    let loading = $state(true);
    let user = $state<UserPrivate | null>(null);
    let configSaving = $state(false);

    // Cargamos datos del usuario (la config se asume cargada por el layout)
    $effect(() => { loadUserData(); });

    async function loadUserData() {
        loading = true;
        try {
            user = await usersApi.getMe();
        } catch (error) {
            toast.error("Failed to load profile data");
        } finally {
            loading = false;
        }
    }

    // Función unificada para persistir cambios en el backend
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
    <title>Settings — Hoshi</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-10 pt-6 md:pt-8 px-4 md:px-6 lg:px-8 xl:px-10 w-full max-w-[2400px] mx-auto space-y-8">

    <!-- HEADER -->
    <header class="flex flex-col md:flex-row md:items-center justify-between gap-5 border-b border-border/40 pb-6 w-full">
        <div class="space-y-1">
            <h1 class="text-3xl md:text-4xl font-black tracking-tight flex items-center gap-3">
                <Settings class="h-8 w-8 md:h-10 md:w-10 text-primary" />
                Settings
            </h1>
            <p class="text-sm md:text-base text-muted-foreground font-medium opacity-80">
                Manage your account, preferences, and reader settings.
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

                    <!-- SIDEBAR NAVEGACIÓN -->
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
                        <Tabs.Trigger value="manga" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <BookOpen class="h-4 w-4" /> Manga Reader
                        </Tabs.Trigger>
                        <Tabs.Trigger value="novel" class="relative px-4 py-3 rounded-xl text-sm font-bold transition-all data-[state=active]:bg-primary/10 data-[state=active]:text-primary data-[state=inactive]:hover:bg-muted/50 whitespace-nowrap w-full justify-start flex items-center gap-3">
                            <BookOpenText class="h-4 w-4" /> Novel Reader
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

                        <!-- ACCOUNT -->
                        <Tabs.Content value="account" class="focus-visible:outline-none mt-0 w-full">
                            <AccountSettings {user} onUpdate={loadUserData} />
                        </Tabs.Content>

                        {#if appConfig.data}
                            <!-- GENERAL -->
                            <Tabs.Content value="general" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <GeneralSettings
                                        bind:showAdultContent={appConfig.data.general.showAdultContent}
                                        bind:blurAdultContent={appConfig.data.general.blurAdultContent}
                                        onSave={handleSaveConfig}
                                />
                            </Tabs.Content>

                            <!-- INTERFACE -->
                            <Tabs.Content value="interface" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <section>
                                    <div class="mb-6">
                                        <h2 class="text-2xl font-bold tracking-tight">Interface</h2>
                                        <p class="text-sm text-muted-foreground mt-1">Customize the look and feel of the app.</p>
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                        <div class="space-y-1 pr-4">
                                            <Label class="text-base font-bold">Theme</Label>
                                            <p class="text-sm text-muted-foreground">Select your visual environment.</p>
                                        </div>
                                        <Select.Root type="single" bind:value={appConfig.data.general.theme} onValueChange={handleSaveConfig}>
                                            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">{appConfig.data.general.theme}</Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value="system">System Default</Select.Item>
                                                <Select.Item value="light">Light</Select.Item>
                                                <Select.Item value="dark">Dark</Select.Item>
                                                <Select.Item value="sepia">Sepia</Select.Item>
                                                <Select.Item value="oled">OLED</Select.Item>
                                            </Select.Content>
                                        </Select.Root>
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                        <div class="space-y-1 pr-4">
                                            <Label class="text-base font-bold">Accent Color</Label>
                                            <p class="text-sm text-muted-foreground">Primary color used for buttons and active states.</p>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <Input type="color" bind:value={appConfig.data.general.accentColor} onchange={handleSaveConfig} class="w-12 h-10 p-1 rounded-lg border-none" />
                                            <span class="text-xs font-mono font-bold opacity-50 uppercase">{appConfig.data.general.accentColor}</span>
                                        </div>
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                        <div class="space-y-1 pr-4">
                                            <Label class="text-base font-bold" for="sidebarCollapsed">Minimize Sidebar</Label>
                                            <p class="text-sm text-muted-foreground">Keep the main navigation sidebar collapsed by default on desktop.</p>
                                        </div>
                                        <Switch id="sidebarCollapsed" bind:checked={appConfig.data.general.sidebarCollapsed} onCheckedChange={handleSaveConfig} />
                                    </div>
                                </section>
                            </Tabs.Content>

                            <!-- MANGA READER -->
                            <Tabs.Content value="manga" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <MangaReaderSettings
                                        bind:layout={appConfig.data.manga.layout}
                                        bind:direction={appConfig.data.manga.direction}
                                        bind:pagesPerView={appConfig.data.manga.pagesPerView}
                                        bind:fitMode={appConfig.data.manga.fitMode}
                                        bind:gapX={appConfig.data.manga.gapX}
                                        bind:gapY={appConfig.data.manga.gapY}
                                        bind:preloadPages={appConfig.data.manga.preloadPages}
                                        bind:defaultChapterLayout={appConfig.data.manga.defaultChapterLayout}
                                        bind:notifyNewChapters={appConfig.data.manga.notifyNewChapters}
                                        onSave={handleSaveConfig}
                                />
                            </Tabs.Content>

                            <!-- NOVEL READER -->
                            <Tabs.Content value="novel" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <NovelReaderSettings
                                        bind:theme={appConfig.data.novel.theme}
                                        bind:fontFamily={appConfig.data.novel.fontFamily}
                                        bind:fontSize={appConfig.data.novel.fontSize}
                                        bind:lineHeight={appConfig.data.novel.lineHeight}
                                        bind:maxWidth={appConfig.data.novel.maxWidth}
                                        bind:textAlign={appConfig.data.novel.textAlign}
                                        onSave={handleSaveConfig}
                                />
                            </Tabs.Content>

                            <!-- PLAYER -->
                            <Tabs.Content value="player" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <section>
                                    <div class="mb-6">
                                        <h2 class="text-2xl font-bold tracking-tight">Player</h2>
                                        <p class="text-sm text-muted-foreground mt-1">Playback and metadata preferences.</p>
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                        <div class="space-y-1 pr-4">
                                            <Label class="text-base font-bold">Preferred Subtitle Language</Label>
                                            <p class="text-sm text-muted-foreground">Default subtitle track to select if available.</p>
                                        </div>
                                        <Select.Root type="single" bind:value={appConfig.data.anime.preferredSubLang} onValueChange={handleSaveConfig}>
                                            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">{appConfig.data.anime.preferredSubLang}</Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value="en">English</Select.Item>
                                                <Select.Item value="es">Spanish</Select.Item>
                                                <Select.Item value="ja">Japanese</Select.Item>
                                                <Select.Item value="none">None</Select.Item>
                                            </Select.Content>
                                        </Select.Root>
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                        <div class="space-y-1 pr-4">
                                            <Label class="text-base font-bold" for="autoNext">Autoplay</Label>
                                            <p class="text-sm text-muted-foreground">Play the next episode automatically.</p>
                                        </div>
                                        <Switch id="autoNext" bind:checked={appConfig.data.anime.autoplayNextEpisode} onCheckedChange={handleSaveConfig} />
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                                        <div class="space-y-1 pr-4">
                                            <Label class="text-base font-bold">Seek Step</Label>
                                            <p class="text-sm text-muted-foreground">Seconds to skip using keyboard arrows.</p>
                                        </div>
                                        <Select.Root type="single" value={appConfig.data.anime.seekStep.toString()} onValueChange={(v) => { appConfig.data.anime.seekStep = parseInt(v); handleSaveConfig(); }}>
                                            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">{appConfig.data.anime.seekStep} seconds</Select.Trigger>
                                            <Select.Content>
                                                <Select.Item value="5">5 seconds</Select.Item>
                                                <Select.Item value="10">10 seconds</Select.Item>
                                                <Select.Item value="15">15 seconds</Select.Item>
                                                <Select.Item value="30">30 seconds</Select.Item>
                                            </Select.Content>
                                        </Select.Root>
                                    </div>
                                </section>
                            </Tabs.Content>

                            <!-- TRACKING -->
                            <Tabs.Content value="tracking" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <TrackerSettings />
                            </Tabs.Content>

                            <!-- EXTENSIONS -->
                            <Tabs.Content value="extensions" class="space-y-16 focus-visible:outline-none mt-0 w-full">
                                <section>
                                    <div class="mb-6">
                                        <h2 class="text-2xl font-bold tracking-tight">Extensions</h2>
                                        <p class="text-sm text-muted-foreground mt-1">Manage content repositories and sources.</p>
                                    </div>

                                    <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-4 py-6">
                                        <div class="space-y-1 pr-4 flex-1">
                                            <Label class="text-base font-bold">Extension Repository URL</Label>
                                            <p class="text-sm text-muted-foreground">URL where extensions and plugins are fetched from.</p>
                                        </div>
                                        <div class="w-full sm:max-w-md space-y-3">
                                            <Input bind:value={appConfig.data.anime.extensionRepoUrl} placeholder="https://..." class="rounded-xl h-11" />
                                            <div class="flex justify-end">
                                                <Button variant="secondary" size="sm" class="rounded-lg font-bold" onclick={handleSaveConfig}>
                                                    Update Repo
                                                </Button>
                                            </div>
                                        </div>
                                    </div>
                                </section>
                            </Tabs.Content>
                        {:else}
                            <div class="flex flex-col items-center justify-center p-20 gap-4 text-muted-foreground">
                                <Loader2 class="size-8 animate-spin" />
                                <p class="text-sm font-bold uppercase tracking-widest opacity-50">Syncing Preferences...</p>
                            </div>
                        {/if}

                    </div>
                </Tabs.Root>
            </div>
        {/if}
    </section>
</main>