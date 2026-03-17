<script lang="ts">
    import { i18n } from "$lib/i18n/index.svelte";
    import { themeManager } from "$lib/theme.svelte";
    import { configApi } from "@/api/config/config";
    import { usersApi } from "@/api/users/users";
    import Tracker from "@/components/settings/Tracker.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import LanguageSelector from "@/components/LanguageSelector.svelte";

    import { Check, ChevronRight, ChevronLeft, Loader2 } from "lucide-svelte";
    import { slide } from "svelte/transition";
    import { toast } from "svelte-sonner";
    import { goto } from "$app/navigation";
    import {layoutState} from "@/layoutState.svelte";

    // --- PROPS PARA EL MODO ---
    let {
        mode = 'user'
    }: {
        mode: 'app' | 'user'
    } = $props();

    // --- ESTADO DINÁMICO DEL WIZARD ---
    let availableSteps = $derived(
        mode === 'app'
            ? ['appearance', 'profile', 'content', 'notifications', 'trackers']
            : ['appearance', 'content', 'notifications', 'trackers']
    );
    let currentIndex = $state(0);
    let currentStepId = $derived(availableSteps[currentIndex]);
    let isSaving = $state(false);

    // --- ESTADOS DE LOS FORMULARIOS ---
    let language = $state(i18n.locale);
    let username = $state("");
    let password = $state("");
    let avatarFile = $state<File | null>(null);
    let showAdultContent = $state(false);
    let blurAdultContent = $state(true);
    let preferredMetadataProvider = $state<'anilist' | 'myanimelist' | 'simkl'>('anilist');
    let defaultHomeSection = $state<'anime' | 'manga' | 'novel'>('anime');
    let notificationsEnabled = $state(true);
    let notifyNewEpisodes = $state(true);

    const themes = [
        { id: 'light', label: 'Light', classes: 'bg-zinc-50 text-zinc-950 border-zinc-200' },
        { id: 'dark', label: 'Dark', classes: 'bg-zinc-900 text-zinc-50 border-zinc-800' },
        { id: 'oled', label: 'OLED', classes: 'bg-black text-white border-zinc-900' }
    ];

    // --- COLORES DE ACENTO ---
    const colorPresets = [
        { name: 'Purple', value: '#a855f7' },
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Cyan', value: '#06b6d4' },
        { name: 'Green', value: '#22c55e' },
        { name: 'Amber', value: '#f59e0b' },
        { name: 'Orange', value: '#f97316' },
        { name: 'Rose', value: '#f43f5e' },
        { name: 'Pink', value: '#ec4899' },
    ];

    $effect(() => {
        layoutState.title = "Setup";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    function setPresetColor(color: string) {
        themeManager.setAccentColor(color);
    }

    function handleCustomColor(event: Event) {
        const input = event.target as HTMLInputElement;
        themeManager.setAccentColor(input.value);
    }

    // --- NAVEGACIÓN ---
    function nextStep() {
        if (currentIndex < availableSteps.length - 1) currentIndex++;
    }

    function prevStep() {
        if (currentIndex > 0) currentIndex--;
    }

    function skipStep() {
        if (currentIndex < availableSteps.length - 1) currentIndex++;
        else finishSetup();
    }

    async function finishSetup() {
        isSaving = true;
        try {
            if (mode === 'app' && username.trim() !== "") {
                await usersApi.updateMe({ username, password: password ? password : undefined });
                if (avatarFile) await usersApi.uploadAvatar(avatarFile);
            }

            await configApi.patchConfig({
                general: {
                    showAdultContent,
                    blurAdultContent,
                    needSetup: false
                },
                ui: { defaultHomeSection },
                content: { preferredMetadataProvider },
                notifications: { enabled: notificationsEnabled, notifyNewEpisodes }
            });

            toast.success(mode === 'app' ? i18n.t('setup.server_setup_complete') : i18n.t('setup.preferences_saved'));
            goto("/home");
        } catch (error) {
            toast.error(i18n.t('errors.network'));
        } finally {
            isSaving = false;
        }
    }

    function handleAvatarChange(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files[0]) avatarFile = input.files[0];
    }
</script>
<svelte:head><title>{i18n.t("setup.title")}</title></svelte:head>

<div class="min-h-screen bg-background flex flex-col items-center justify-center p-4">
    <div class="w-full max-w-2xl bg-card border border-border/50 rounded-3xl shadow-xl overflow-hidden flex flex-col min-h-[650px]">

        <div class="p-8 pb-4 border-b border-border/40 bg-muted/10">
            <h1 class="text-3xl font-bold tracking-tight text-center mb-6">
                {mode === 'app' ? i18n.t('setup.welcome_app') : i18n.t('setup.welcome_user')}
            </h1>
            <div class="flex items-center justify-center gap-2">
                {#each availableSteps as _, i}
                    <div class="h-2 rounded-full transition-all duration-300 {currentIndex >= i ? 'w-12 bg-primary' : 'w-4 bg-border'}"></div>
                {/each}
            </div>
        </div>

        <div class="flex-1 p-8 relative overflow-y-auto overflow-x-hidden">

            {#if currentStepId === 'appearance'}
                <div in:slide={{ axis: 'x', duration: 300 }} class="space-y-8 pb-4">
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.appearance.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.appearance.description')}</p>
                    </div>

                    <div class="space-y-4">
                        <Label class="text-base font-bold">{i18n.t('setup.appearance.language')}</Label>
                        <LanguageSelector
                                class="w-full h-11 rounded-xl bg-muted/20"
                                onLanguageChange={(code) => { language = code; i18n.setLocale(code); }}
                        />
                    </div>

                    <div class="space-y-4">
                        <Label class="text-base font-bold">{i18n.t('setup.appearance.theme')}</Label>
                        <div class="grid grid-cols-3 gap-3">
                            {#each themes as theme}
                                <button onclick={() => themeManager.setTheme(theme.id)} class="relative flex items-center justify-center h-14 rounded-xl border-2 font-bold {theme.classes} {themeManager.theme === theme.id ? 'ring-2 ring-primary border-transparent' : 'opacity-80 border-transparent'}">
                                    {theme.label}
                                    {#if themeManager.theme === theme.id}
                                        <div class="absolute top-1 right-1 bg-primary rounded-full p-0.5">
                                            <Check class="size-3 text-primary-foreground" />
                                        </div>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    </div>

                    <div class="space-y-4">
                        <Label class="text-base font-bold">{i18n.t('setup.appearance.accent_color')}</Label>
                        <div class="flex flex-wrap items-center gap-3">
                            <div class="relative flex items-center gap-3 bg-muted/20 p-2 rounded-2xl border border-border/50">
                                <Input
                                        type="color"
                                        value={themeManager.accentColor || '#ffffff'}
                                        onchange={handleCustomColor}
                                        class="w-10 h-10 p-0 rounded-lg border-none cursor-pointer bg-transparent shrink-0"
                                />
                                <span class="text-xs font-mono font-bold pr-2 uppercase opacity-70">{themeManager.accentColor}</span>
                            </div>

                            <div class="h-8 w-px bg-border/40 mx-2 hidden sm:block"></div>

                            <div class="flex flex-wrap gap-2">
                                {#each colorPresets as preset}
                                    <button
                                            type="button"
                                            onclick={() => setPresetColor(preset.value)}
                                            class="size-10 rounded-full border-2 border-background shadow-sm transition-transform active:scale-90 flex items-center justify-center"
                                            style="background-color: {preset.value}"
                                            title={preset.name}
                                    >
                                        {#if themeManager.accentColor?.toLowerCase() === preset.value.toLowerCase()}
                                            <Check class="size-5 text-white drop-shadow-md" />
                                        {/if}
                                    </button>
                                {/each}
                            </div>
                        </div>
                    </div>
                </div>
            {/if}

            {#if currentStepId === 'profile'}
                <div in:slide={{ axis: 'x', duration: 300 }} class="space-y-8">
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.profile.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.profile.description')}</p>
                    </div>
                    <div class="space-y-4 max-w-sm mx-auto">
                        <div class="space-y-2">
                            <Label for="username">{i18n.t('setup.profile.username')}</Label>
                            <Input id="username" bind:value={username} placeholder={i18n.t('setup.profile.username_placeholder')} class="h-11 rounded-xl" />
                        </div>
                        <div class="space-y-2">
                            <Label for="password">{i18n.t('setup.profile.password')}</Label>
                            <Input id="password" type="password" bind:value={password} placeholder="••••••••" class="h-11 rounded-xl" />
                        </div>
                        <div class="space-y-2">
                            <Label for="avatar">{i18n.t('setup.profile.avatar')}</Label>
                            <Input id="avatar" type="file" accept="image/*" onchange={handleAvatarChange} class="h-11 rounded-xl cursor-pointer" />
                        </div>
                    </div>
                </div>
            {/if}

            {#if currentStepId === 'content'}
                <div in:slide={{ axis: 'x', duration: 300 }} class="space-y-8">
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.content.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.content.description')}</p>
                    </div>

                    <div class="space-y-6">
                        <div class="space-y-2">
                            <Label class="text-base font-bold">{i18n.t('setup.content.metadata_provider')}</Label>
                            <select bind:value={preferredMetadataProvider} class="flex h-11 w-full rounded-xl border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                                <option value="anilist">AniList</option>
                                <option value="myanimelist">MyAnimeList</option>
                                <option value="simkl">Simkl</option>
                            </select>
                        </div>

                        <div class="space-y-2">
                            <Label class="text-base font-bold">{i18n.t('setup.content.default_home_section')}</Label>
                            <select bind:value={defaultHomeSection} class="flex h-11 w-full rounded-xl border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                                <option value="anime">{i18n.t('setup.content.anime')}</option>
                                <option value="manga">{i18n.t('setup.content.manga')}</option>
                                <option value="novel">{i18n.t('setup.content.novel')}</option>
                            </select>
                        </div>

                        <div class="flex items-center justify-between py-2 border-b border-border/40">
                            <div class="space-y-1 pr-4">
                                <Label class="text-base font-bold cursor-pointer" for="showAdultContent">{i18n.t('setup.content.show_nsfw')}</Label>
                            </div>
                            <Switch id="showAdultContent" bind:checked={showAdultContent} class="shrink-0" />
                        </div>

                        <div class="flex items-center justify-between py-2 transition-opacity { !showAdultContent ? 'opacity-50' : '' }">
                            <div class="space-y-1 pr-4">
                                <Label class="text-base font-bold { showAdultContent ? 'cursor-pointer' : 'cursor-not-allowed' }" for="blurAdultContent">{i18n.t('setup.content.blur_nsfw')}</Label>
                            </div>
                            <Switch id="blurAdultContent" bind:checked={blurAdultContent} disabled={!showAdultContent} class="shrink-0" />
                        </div>
                    </div>
                </div>
            {/if}

            {#if currentStepId === 'notifications'}
                <div in:slide={{ axis: 'x', duration: 300 }} class="space-y-8">
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.notifications.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.notifications.description')}</p>
                    </div>

                    <div class="space-y-6 max-w-sm mx-auto">
                        <div class="flex items-center justify-between p-4 rounded-2xl border border-border bg-muted/10">
                            <div class="space-y-1">
                                <Label class="text-base font-bold">{i18n.t('setup.notifications.enable')}</Label>
                                <p class="text-xs text-muted-foreground">{i18n.t('setup.notifications.enable_desc')}</p>
                            </div>
                            <Switch bind:checked={notificationsEnabled} />
                        </div>

                        <div class="flex items-center justify-between p-4 rounded-2xl border border-border bg-muted/10 transition-opacity {!notificationsEnabled ? 'opacity-50' : 'opacity-100'}">
                            <div class="space-y-1">
                                <Label class="text-base font-bold">{i18n.t('setup.notifications.new_episodes')}</Label>
                                <p class="text-xs text-muted-foreground">{i18n.t('setup.notifications.new_episodes_desc')}</p>
                            </div>
                            <Switch bind:checked={notifyNewEpisodes} disabled={!notificationsEnabled} />
                        </div>
                    </div>
                </div>
            {/if}

            {#if currentStepId === 'trackers'}
                <div in:slide={{ axis: 'x', duration: 300 }} class="space-y-4">
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.trackers.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.trackers.description')}</p>
                    </div>
                    <div class="border border-border/50 rounded-2xl p-4 bg-muted/5 max-h-[350px] overflow-y-auto">
                        <Tracker />
                    </div>
                </div>
            {/if}

        </div>

        <div class="p-6 border-t border-border/40 bg-muted/10 flex items-center justify-between">
            <div>
                {#if currentIndex > 0}
                    <Button variant="ghost" onclick={prevStep} class="rounded-xl font-bold h-11">
                        <ChevronLeft class="mr-2 h-4 w-4" /> {i18n.t('setup.navigation.back')}
                    </Button>
                {/if}
            </div>

            <div class="flex items-center gap-3">
                <Button variant="ghost" onclick={skipStep} class="rounded-xl font-bold h-11 text-muted-foreground">{i18n.t('setup.navigation.skip')}</Button>

                {#if currentIndex < availableSteps.length - 1}
                    <Button onclick={nextStep} class="rounded-xl font-bold h-11 px-8 shadow-sm">
                        {i18n.t('setup.navigation.next')} <ChevronRight class="ml-2 h-4 w-4" />
                    </Button>
                {:else}
                    <Button onclick={finishSetup} disabled={isSaving} class="rounded-xl font-bold h-11 px-8 shadow-sm bg-primary text-primary-foreground">
                        {#if isSaving}<Loader2 class="mr-2 h-4 w-4 animate-spin" /> {i18n.t('setup.navigation.saving')}{:else}{i18n.t('setup.navigation.finish')} <Check class="ml-2 h-4 w-4" />{/if}
                    </Button>
                {/if}
            </div>
        </div>
    </div>
</div>