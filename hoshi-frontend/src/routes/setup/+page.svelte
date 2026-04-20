<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { themeManager } from "@/stores/theme.svelte.js";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import LanguageSelector from "@/components/LanguageSelector.svelte";
    import { auth } from "@/stores/auth.svelte.js";
    import { appConfig } from "@/stores/config.svelte.js";
    import { Check, ChevronRight, ChevronLeft, UserCircle2, Camera, User } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import { fly, fade } from "svelte/transition";
    import { toast } from "svelte-sonner";
    import { goto } from "$app/navigation";
    import { layoutState } from "@/stores/layout.svelte.js";
    import type { CoreError } from "@/api/client";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import Marketplace from "@/components/settings/extensions/Marketplace.svelte";

    const availableSteps = ['appearance', 'content', 'marketplace'];

    let currentIndex = $state(0);
    let currentStepId = $derived(availableSteps[currentIndex]);
    let isSaving = $state(false);

    let language = $state(i18n.locale);

    let username = $state("");
    let avatarFile = $state<File | null>(null);
    let avatarPreview = $state<string | null>(null);

    let showAdultContent = $state(false);
    let blurAdultContent = $state(true);
    let preferredMetadataProvider = $state<'anilist' | 'myanimelist' | 'kitsu'>('anilist');
    let titleLanguage = $state<'romaji' | 'english' | 'native'>('romaji');
    let defaultHomeSection = $state<'anime' | 'manga' | 'novel'>('anime');

    // Marketplace State
    let extConfig = $state({ repoUrl: appConfig.data?.extensions?.repoUrl || "" });

    const themes = [
        { id: 'light', label: 'Light', classes: 'bg-zinc-50 text-zinc-950 border-zinc-200' },
        { id: 'dark', label: 'Dark', classes: 'bg-zinc-900 text-zinc-50 border-zinc-800' },
        { id: 'oled', label: 'OLED', classes: 'bg-black text-white border-zinc-900' }
    ];

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

    const metadataOptions = [
        { value: "anilist", label: "AniList" },
        { value: "myanimelist", label: "MyAnimeList" },
        { value: "kitsu", label: "Kitsu" }
    ];

    const languageOptions = $derived([
        { value: "romaji", label: i18n.t('setup.content.romaji') },
        { value: "english", label: i18n.t('setup.content.english') },
        { value: "native", label: i18n.t('setup.content.native') }
    ]);

    const sectionOptions = $derived([
        { value: "anime", label: i18n.t('setup.content.anime') },
        { value: "manga", label: i18n.t('setup.content.manga') },
        { value: "novel", label: i18n.t('setup.content.novel') }
    ]);

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

    function nextStep() {
        if (currentStepId === 'appearance' && !username.trim()) {
            toast.error(i18n.t('setup.profile.validation_error'));
            return;
        }
        if (currentIndex < availableSteps.length - 1) currentIndex++;
    }

    function prevStep() {
        if (currentIndex > 0) currentIndex--;
    }

    function skipStep() {
        if (currentStepId === 'appearance') {
            toast.error(i18n.t('setup.profile.require_profile'));
            return;
        }

        if (currentIndex < availableSteps.length - 1) currentIndex++;
        else finishSetup();
    }

    async function finishSetup() {
        if (!username.trim()) {
            currentIndex = availableSteps.indexOf('appearance');
            toast.error(i18n.t('setup.profile.validation_error'));
            return;
        }

        isSaving = true;
        try {
            const registerData = {
                username
            };

            await auth.register(registerData, avatarFile);

            await appConfig.update({
                general: {
                    showAdultContent,
                    blurAdultContent,
                    needSetup: false
                },
                ui: {
                    sidebarCollapsed: appConfig.data?.ui?.sidebarCollapsed ?? false,
                    disableCardTrailers: appConfig.data?.ui?.disableCardTrailers ?? false,
                    defaultHomeSection,
                    titleLanguage
                },
                content: {
                    preferredMetadataProvider,
                    autoUpdateProgress: appConfig.data?.content?.autoUpdateProgress ?? true
                },
                extensions: extConfig
            });

            toast.success(i18n.t('setup.server_setup_complete'));
            goto("/");
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            isSaving = false;
        }
    }

    function handleAvatarChange(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files[0]) {
            avatarFile = input.files[0];
            avatarPreview = URL.createObjectURL(avatarFile);
        }
    }
</script>

<svelte:head><title>{i18n.t("setup.title")}</title></svelte:head>

<div class="min-h-screen bg-background text-foreground flex flex-col">
    <div class="w-full max-w-3xl mx-auto flex flex-col flex-1 py-12 px-6">

        <header class="mb-12">
            <h1 class="text-4xl font-extrabold tracking-tight text-center mb-8">
                {i18n.t('setup.welcome_app')}
            </h1>
            <div class="flex items-center justify-center gap-2">
                {#each availableSteps as _, i}
                    <div class="h-2 rounded-full transition-all duration-300 {currentIndex >= i ? 'w-12 bg-primary' : 'w-4 bg-muted'}"></div>
                {/each}
            </div>
        </header>

        <main class="flex-1 relative pb-12">

            {#if currentStepId === 'appearance'}
                <div
                        in:fly={{ x: 50, duration: 300, delay: 150 }}
                        out:fly={{ x: -50, duration: 150 }}
                        class="space-y-8 col-start-1 row-start-1 max-w-lg mx-auto w-full"
                >
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.appearance.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.appearance.description')}</p>
                    </div>

                    <div class="flex items-center gap-6 pb-6 mb-6 border-b border-border/20">
                        <div class="relative shrink-0">
                            <div class="size-16 rounded-xl bg-muted/40 border border-border/50 flex items-center justify-center overflow-hidden shadow-sm">
                                {#if avatarPreview}
                                    <img src={avatarPreview} alt="Preview" class="w-full h-full object-cover" />
                                {:else}
                                    <UserCircle2 class="size-8 text-muted-foreground/40" />
                                {/if}
                            </div>
                            <label
                                    for="avatar-upload"
                                    class="absolute -bottom-1 -right-1 size-6 bg-primary text-primary-foreground rounded-md flex items-center justify-center cursor-pointer shadow-md hover:scale-105 transition-transform"
                            >
                                <Camera class="size-3" />
                                <input id="avatar-upload" type="file" accept="image/*" class="hidden" onchange={handleAvatarChange} />
                            </label>
                        </div>

                        <div class="flex-1 space-y-1.5">
                            <Label class="text-[10px] font-bold uppercase tracking-wider text-muted-foreground ml-1">
                                {i18n.t('setup.profile.username')}
                            </Label>
                            <div class="relative">
                                <User class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground/60" />
                                <Input
                                        bind:value={username}
                                        placeholder="Spike"
                                        class="h-10 pl-10 bg-muted/20 border-border/40 rounded-xl focus-visible:ring-primary/20"
                                />
                            </div>
                        </div>
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

            {#if currentStepId === 'content'}
                <div
                        in:fly={{ x: 50, duration: 300, delay: 150 }}
                        out:fly={{ x: -50, duration: 150 }}
                        class="space-y-8 col-start-1 row-start-1"
                >
                    <div class="text-center space-y-2">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.content.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.content.description')}</p>
                    </div>

                    <div class="space-y-6 max-w-lg mx-auto">
                        <div class="space-y-2">
                            <Label class="text-base font-bold">{i18n.t('setup.content.metadata_provider')}</Label>
                            <ResponsiveSelect
                                    bind:value={preferredMetadataProvider}
                                    items={metadataOptions}
                                    label={i18n.t('setup.content.metadata_provider')}
                            />
                        </div>

                        <div class="space-y-2">
                            <Label class="text-base font-bold">{i18n.t('setup.content.title_language')}</Label>
                            <ResponsiveSelect
                                    bind:value={titleLanguage}
                                    items={languageOptions}
                                    label={i18n.t('setup.content.title_language')}
                            />
                        </div>

                        <div class="space-y-2">
                            <Label class="text-base font-bold">{i18n.t('setup.content.default_home_section')}</Label>
                            <ResponsiveSelect
                                    bind:value={defaultHomeSection}
                                    items={sectionOptions}
                                    label={i18n.t('setup.content.default_home_section')}
                            />
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

            {#if currentStepId === 'marketplace'}
                <div
                        in:fly={{ x: 50, duration: 300, delay: 150 }}
                        out:fly={{ x: -50, duration: 150 }}
                        class="space-y-8 col-start-1 row-start-1"
                >
                    <div class="text-center space-y-2 mb-8">
                        <h2 class="text-2xl font-bold">{i18n.t('setup.marketplace.title')}</h2>
                        <p class="text-muted-foreground">{i18n.t('setup.marketplace.description')}</p>
                    </div>

                    <div class="max-w-2xl mx-auto w-full">
                        <Marketplace bind:config={extConfig} onSave={async () => {}} />
                    </div>
                </div>
            {/if}

        </main>

        <footer class="mt-auto pt-6 flex items-center justify-between border-t border-border/30">
            <div>
                {#if currentIndex > 0}
                    <Button variant="ghost" onclick={prevStep} class="rounded-xl font-bold h-12 px-6">
                        <ChevronLeft class="mr-2 h-5 w-5" /> {i18n.t('setup.navigation.back')}
                    </Button>
                {/if}
            </div>

            <div class="flex items-center gap-3">
                {#if currentStepId !== 'appearance'}
                    <Button variant="ghost" onclick={skipStep} class="rounded-xl font-bold h-12 px-6 text-muted-foreground hover:text-foreground transition-colors">
                        {i18n.t('setup.navigation.skip')}
                    </Button>
                {/if}

                {#if currentIndex < availableSteps.length - 1}
                    <Button onclick={nextStep} class="rounded-xl font-bold h-12 px-8 shadow-sm">
                        {i18n.t('setup.navigation.next')} <ChevronRight class="ml-2 h-5 w-5" />
                    </Button>
                {:else}
                    <Button onclick={finishSetup} disabled={isSaving} class="rounded-xl font-bold h-12 px-8 shadow-sm bg-primary text-primary-foreground hover:bg-primary/90">
                        {#if isSaving}
                            <Spinner class="mr-2 h-5 w-5 animate-spin" /> {i18n.t('setup.navigation.saving')}
                        {:else}
                            {i18n.t('setup.navigation.finish')} <Check class="ml-2 h-5 w-5" />
                        {/if}
                    </Button>
                {/if}
            </div>
        </footer>
    </div>
</div>