<script lang="ts">
    import { i18n } from "$lib/i18n/index.svelte";
    import { themeManager } from "$lib/theme.svelte";
    import { configApi } from "@/api/config/config";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import LanguageSelector from "@/components/LanguageSelector.svelte";
    import { auth } from "$lib/auth.svelte";

    import { Check, ChevronRight, ChevronLeft, Loader2, UploadCloud } from "lucide-svelte";
    import { slide } from "svelte/transition";
    import { toast } from "svelte-sonner";
    import { goto } from "$app/navigation";
    import { layoutState } from "@/layoutState.svelte";
    import {contentApi} from "@/api/content/content";
    import {onMount} from "svelte";

    const availableSteps = ['appearance', 'profile', 'content', 'notifications'];

    let currentIndex = $state(0);
    let currentStepId = $derived(availableSteps[currentIndex]);
    let isSaving = $state(false);

    let language = $state(i18n.locale);
    let username = $state("");
    let password = $state("");
    let avatarFile = $state<File | null>(null);
    let avatarPreview = $state<string | null>(null);
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

    function nextStep() {
        if (currentStepId === 'profile' && !username.trim()) {
            toast.error(i18n.t('setup.profile.validation_error'));
            return;
        }
        if (currentIndex < availableSteps.length - 1) currentIndex++;
    }

    function prevStep() {
        if (currentIndex > 0) currentIndex--;
    }

    function skipStep() {
        if (currentStepId === 'profile') {
            toast.error(i18n.t('setup.profile.require_profile'));
            return;
        }

        if (currentIndex < availableSteps.length - 1) currentIndex++;
        else finishSetup();
    }

    async function finishSetup() {
        if (!username.trim()) {
            currentIndex = availableSteps.indexOf('profile');
            toast.error(i18n.t('setup.profile.validation_error'));
            return;
        }

        isSaving = true;
        try {
            const registerData = {
                username,
                ...(password.trim() ? { password } : {})
            };
            await auth.register(registerData, avatarFile);

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
            toast.success(i18n.t('setup.server_setup_complete'));
            goto("/");
        } catch (error: any) {
            toast.error(error?.message || i18n.t('errors.network'));
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

    onMount(() => {
        contentApi.getHome()
    });

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
                <div in:slide={{ axis: 'x', duration: 300 }} class="space-y-8">
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

                    <div class="space-y-8 max-w-md mx-auto">

                        <div class="flex flex-col items-center justify-center gap-3">
                            <Label for="avatar" class="relative cursor-pointer group rounded-full">
                                <div class="size-24 rounded-full border-2 border-dashed border-border/60 flex items-center justify-center bg-muted/5 overflow-hidden group-hover:border-primary group-hover:bg-muted/10 transition-all shadow-sm">
                                    {#if avatarPreview}
                                        <img src={avatarPreview} alt="Avatar" class="w-full h-full object-cover" />
                                    {:else}
                                        <UploadCloud class="size-8 text-muted-foreground group-hover:text-primary transition-colors" />
                                    {/if}
                                </div>
                                <div class="absolute inset-0 bg-black/50 rounded-full opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                    <span class="text-xs text-white font-bold tracking-wider">{i18n.t('setup.profile.avatar_upload')}</span>
                                </div>
                            </Label>
                            <input id="avatar" type="file" accept="image/*" onchange={handleAvatarChange} class="hidden" />

                            <div class="text-center">
                                <Label class="text-base font-bold">{i18n.t('setup.profile.avatar')}</Label>
                                <p class="text-xs text-muted-foreground mt-1">{i18n.t('setup.profile.avatar_desc')}</p>
                            </div>
                        </div>

                        <div class="space-y-4">
                            <div class="space-y-2">
                                <Label for="username" class="font-semibold">{i18n.t('setup.profile.username')}</Label>
                                <Input id="username" bind:value={username} placeholder={i18n.t('setup.profile.username_placeholder')} class="h-11 rounded-xl" />
                            </div>
                            <div class="space-y-2">
                                <Label for="password" class="font-semibold">{i18n.t('setup.profile.password')}</Label>
                                <Input id="password" type="password" bind:value={password} placeholder="••••••••" class="h-11 rounded-xl" />
                            </div>
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

                    <div class="space-y-6 max-w-lg mx-auto">
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

                    <div class="space-y-6 max-w-lg mx-auto">
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
                {#if currentStepId !== 'profile'}
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
                            <Loader2 class="mr-2 h-5 w-5 animate-spin" /> {i18n.t('setup.navigation.saving')}
                        {:else}
                            {i18n.t('setup.navigation.finish')} <Check class="ml-2 h-5 w-5" />
                        {/if}
                    </Button>
                {/if}
            </div>
        </footer>
    </div>
</div>