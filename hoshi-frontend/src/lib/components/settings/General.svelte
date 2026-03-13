<script lang="ts">
    import { i18n } from "$lib/i18n/index.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Palette, Check } from "lucide-svelte";
    import { themeManager } from "$lib/theme.svelte";
    import Backups from "./Backups.svelte";

    import type { GeneralConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: GeneralConfig,
        onSave: () => Promise<void> | void
    } = $props();

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

    function changeLanguage(value: string) {
        config.language = value;
        i18n.setLocale(value as 'en' | 'es');
        onSave();
    }

    function changeTheme(themeId: string) {
        themeManager.setTheme(themeId);
    }

    function setPresetColor(color: string) {
        themeManager.setAccentColor(color);
    }

    function handleCustomColor(event: Event) {
        const input = event.target as HTMLInputElement;
        themeManager.setAccentColor(input.value);
    }
</script>

<section class="space-y-2">
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.general')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.general_desc')}</p>
    </div>

    <div class="flex flex-col gap-4 py-6 border-b border-border/40">
        <div class="space-y-1">
            <Label class="text-base font-bold flex items-center gap-2">
                <Palette class="size-4" /> {i18n.t('settings.theme')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.theme_desc')}</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
            {#each themes as theme}
                <button
                        type="button"
                        onclick={() => changeTheme(theme.id)}
                        class="relative flex items-center justify-center h-14 rounded-xl border-2 font-bold transition-all overflow-hidden {theme.classes}
        {themeManager.theme === theme.id
            ? 'ring-2 ring-primary ring-offset-2 ring-offset-background border-transparent'
            : 'opacity-80 hover:opacity-100 border-transparent'}"
                >
                    {theme.label}
                    {#if themeManager.theme === theme.id}
                        <div class="absolute top-1.5 right-1.5 bg-primary rounded-full p-0.5">
                            <Check class="size-3 text-primary-foreground" />
                        </div>
                    {/if}
                </button>
            {/each}
        </div>
    </div>

    <div class="flex flex-col gap-4 py-6 border-b border-border/40">
        <div class="space-y-1">
            <Label class="text-base font-bold">{i18n.t('settings.accent_color')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.accent_color_desc')}</p>
        </div>

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

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold" for="language">{i18n.t('settings.language')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.language_desc')}</p>
        </div>
        <div class="w-full sm:max-w-[200px]">
            <Select.Root type="single" value={config.language} onValueChange={changeLanguage}>
                <Select.Trigger id="language" class="w-full h-11 rounded-xl font-bold bg-muted/20 border-transparent hover:bg-muted/30 transition-colors">
                    {config.language === 'es' ? 'Español' : 'English'}
                </Select.Trigger>
                <Select.Content>
                    <Select.Item value="en">English</Select.Item>
                    <Select.Item value="es">Español</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold cursor-pointer" for="showAdultContent">{i18n.t('settings.show_nsfw')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.show_nsfw_desc')}</p>
        </div>
        <Switch id="showAdultContent" bind:checked={config.showAdultContent} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 transition-opacity { !config.showAdultContent ? 'opacity-50' : '' }">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold { config.showAdultContent ? 'cursor-pointer' : 'cursor-not-allowed' }" for="blurAdultContent">
                {i18n.t('settings.blur_nsfw')}
            </Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.blur_nsfw_desc')}</p>
        </div>
        <Switch
                id="blurAdultContent"
                bind:checked={config.blurAdultContent}
                disabled={!config.showAdultContent}
                onCheckedChange={onSave}
                class="shrink-0"
        />
    </div>

    <div class="pt-8 mt-2 border-t border-border/40 w-full">
        <div class="mb-6">
            <h3 class="text-xl font-bold tracking-tight">
                {i18n.t('settings.backups')}
            </h3>
            <p class="text-sm text-muted-foreground mt-1">
                {i18n.t('settings.backups_desc')}
            </p>
        </div>

        <Backups />
    </div>
</section>