<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import type { PlayerConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: PlayerConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">Player</h2>
        <p class="text-sm text-muted-foreground mt-1">Video playback and anime-specific preferences.</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold">Preferred Subtitle Language</Label>
            <p class="text-sm text-muted-foreground">Comma-separated list of language codes (e.g. <b>en, es, it</b>) in order of priority.</p>
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
            <Label class="text-base font-bold">Preferred Audio (Dub) Language</Label>
            <p class="text-sm text-muted-foreground">Comma-separated list of language codes (e.g. <b>ja, en</b>) in order of priority.</p>
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
            <Label class="text-base font-bold">Default Episode Layout</Label>
            <p class="text-sm text-muted-foreground">How episodes are displayed on the details page.</p>
        </div>
        <Select.Root type="single" bind:value={config.defaultEpisodeLayout} onValueChange={onSave}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">{config.defaultEpisodeLayout}</Select.Trigger>
            <Select.Content>
                <Select.Item value="grid">Grid View</Select.Item>
                <Select.Item value="list">List View</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">Seek Step</Label>
            <p class="text-sm text-muted-foreground">Seconds to skip using keyboard arrows or double tap.</p>
        </div>
        <Select.Root type="single" value={config.seekStep.toString()} onValueChange={(v) => { config.seekStep = parseInt(v); onSave(); }}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md">{config.seekStep} seconds</Select.Trigger>
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
            <Label class="text-base font-bold" for="autoNext">Autoplay Next Episode</Label>
            <p class="text-sm text-muted-foreground">Automatically start the next episode when the current one ends.</p>
        </div>
        <Switch id="autoNext" bind:checked={config.autoplayNextEpisode} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="resumeFromLastPos">Resume Playback</Label>
            <p class="text-sm text-muted-foreground">Remember where you left off and resume from your last position.</p>
        </div>
        <Switch id="resumeFromLastPos" bind:checked={config.resumeFromLastPos} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoSkipIntro">Auto-Skip Intro</Label>
            <p class="text-sm text-muted-foreground">Automatically skip known opening themes.</p>
        </div>
        <Switch id="autoSkipIntro" bind:checked={config.autoSkipIntro} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoSkipOutro">Auto-Skip Outro</Label>
            <p class="text-sm text-muted-foreground">Automatically skip known ending themes.</p>
        </div>
        <Switch id="autoSkipOutro" bind:checked={config.autoSkipOutro} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>