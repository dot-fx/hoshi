<script lang="ts">
    import { i18n } from "$lib/i18n/index.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Palette, Check } from "lucide-svelte";

    import type { GeneralConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: GeneralConfig,
        onSave: () => Promise<void> | void
    } = $props();

    // Theme definitions with specific preview styles
    const themes = [
        { id: 'light', label: 'Light', classes: 'bg-zinc-50 text-zinc-950 border-zinc-200' },
        { id: 'dark', label: 'Dark', classes: 'bg-zinc-900 text-zinc-50 border-zinc-800' },
        { id: 'oled', label: 'OLED', classes: 'bg-black text-white border-zinc-900' }
    ];

    const colorPresets = [
        { name: 'Purple', value: '#a855f7' },
        { name: 'Blue', value: '#3b82f6' },
        { name: 'Rose', value: '#f43f5e' },
        { name: 'Green', value: '#22c55e' },
        { name: 'Orange', value: '#f97316' },
    ];

    function changeLanguage(value: string) {
        config.language = value;
        i18n.setLocale(value as 'en' | 'es');
        onSave();
    }

    function setPresetColor(color: string) {
        config.accentColor = color;
        onSave();
    }
</script>

<section class="space-y-2">
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('general')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('general_desc')}</p>
    </div>

    <div class="flex flex-col gap-4 py-6 border-b border-border/40">
        <div class="space-y-1">
            <Label class="text-base font-bold flex items-center gap-2">
                <Palette class="size-4" /> Theme
            </Label>
            <p class="text-sm text-muted-foreground">Select your visual environment.</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
            {#each themes as theme}
                <button
                        type="button"
                        onclick={() => { config.theme = theme.id as any; onSave(); }}
                        class="relative flex items-center justify-center h-14 rounded-xl border-2 font-bold transition-all overflow-hidden {theme.classes}
                    {config.theme === theme.id
                        ? 'ring-2 ring-primary ring-offset-2 ring-offset-background border-transparent'
                        : 'opacity-80 hover:opacity-100 border-transparent'}"
                >
                    {theme.label}
                    {#if config.theme === theme.id}
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
            <Label class="text-base font-bold">Accent Color</Label>
            <p class="text-sm text-muted-foreground">Primary color used for buttons and active states.</p>
        </div>

        <div class="flex flex-wrap items-center gap-3">
            <div class="relative flex items-center gap-3 bg-muted/20 p-2 rounded-2xl border border-border/50">
                <Input
                        type="color"
                        bind:value={config.accentColor}
                        onchange={onSave}
                        class="w-10 h-10 p-0 rounded-lg border-none cursor-pointer bg-transparent shrink-0"
                />
                <span class="text-xs font-mono font-bold pr-2 uppercase opacity-70">{config.accentColor}</span>
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
                        {#if config.accentColor.toLowerCase() === preset.value.toLowerCase()}
                            <Check class="size-5 text-white drop-shadow-md" />
                        {/if}
                    </button>
                {/each}
            </div>
        </div>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('language')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('language_help')}</p>
        </div>
        <Select.Root type="single" bind:value={config.language} onValueChange={changeLanguage}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">
                {config.language === 'en' ? i18n.t('english') : i18n.t('spanish')}
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="en">{i18n.t('english')}</Select.Item>
                <Select.Item value="es">{i18n.t('spanish')}</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="showAdultContent">{i18n.t('show_adult_content')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('show_adult_content_help')}</p>
        </div>
        <Switch id="showAdultContent" bind:checked={config.showAdultContent} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="blurAdultContent">{i18n.t('blur_adult_content')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('blur_adult_content_help')}</p>
        </div>
        <Switch id="blurAdultContent" bind:checked={config.blurAdultContent} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>