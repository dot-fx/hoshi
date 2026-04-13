<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import type { PlayerConfig } from "@/api/config/types";
    import { i18n } from "@/i18n/index.svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: PlayerConfig,
        onSave: () => Promise<void> | void
    } = $props();

    const seekSteps = [
        { value: "5", label: i18n.t('settings.player_section.seconds', {num: 5}) },
        { value: "10", label: i18n.t('settings.player_section.seconds', {num: 10}) },
        { value: "15", label: i18n.t('settings.player_section.seconds', {num: 15}) },
        { value: "30", label: i18n.t('settings.player_section.seconds', {num: 30}) }
    ];

    function handleSeekStepChange(val: string) {
        config.seekStep = parseInt(val);
        onSave();
    }
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.player')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.player_section.player_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_sub_lang')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_sub_lang_desc')}</p>
        </div>
        <div class="w-full sm:max-w-md">
            <Input
                    bind:value={config.preferredSubLang}
                    onchange={onSave}
                    placeholder="en, es, ja"
                    class="rounded-xl h-11"
            />
        </div>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold">{i18n.t('settings.player_section.preferred_dub_lang')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.preferred_dub_lang_desc')}</p>
        </div>
        <div class="w-full sm:max-w-md">
            <Input
                    bind:value={config.preferredDubLang}
                    onchange={onSave}
                    placeholder="ja, en"
                    class="rounded-xl h-11"
            />
        </div>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('settings.player_section.seek_step')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.seek_step_desc')}</p>
        </div>

        <ResponsiveSelect
                value={config.seekStep.toString()}
                items={seekSteps}
                class="rounded-xl h-11 w-full sm:max-w-md"
                onValueChange={handleSeekStepChange}
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoNext">{i18n.t('settings.player_section.autoplay')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.autoplay_desc')}</p>
        </div>
        <Switch id="autoNext" bind:checked={config.autoplayNextEpisode} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="resumeFromLastPos">{i18n.t('settings.player_section.resume_playback')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.resume_playback_desc')}</p>
        </div>
        <Switch id="resumeFromLastPos" bind:checked={config.resumeFromLastPos} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoSkipIntro">{i18n.t('settings.player_section.auto_skip_intro')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_intro_desc')}</p>
        </div>
        <Switch id="autoSkipIntro" bind:checked={config.autoSkipIntro} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoSkipOutro">{i18n.t('settings.player_section.auto_skip_outro')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.player_section.auto_skip_outro_desc')}</p>
        </div>
        <Switch id="autoSkipOutro" bind:checked={config.autoSkipOutro} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>