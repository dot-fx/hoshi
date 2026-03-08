<script lang="ts">
    import { i18n } from "$lib/i18n/index.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";

    let {
        appLanguage = $bindable(),
        showAdultContent = $bindable(),
        blurAdultContent = $bindable(),
        onSave
    }: {
        appLanguage: string,
        showAdultContent: boolean,
        blurAdultContent: boolean,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section>
    <div class="mb-2">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('general')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('general_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('language')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('language_help')}</p>
        </div>
        <Select.Root type="single" bind:value={appLanguage} onValueChange={onSave}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">
                {appLanguage === 'en' ? i18n.t('english') : i18n.t('spanish')}
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
        <Switch id="showAdultContent" bind:checked={showAdultContent} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="blurAdultContent">{i18n.t('blur_adult_content')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('blur_adult_content_help')}</p>
        </div>
        <Switch id="blurAdultContent" bind:checked={blurAdultContent} disabled={!showAdultContent} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>