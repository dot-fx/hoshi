<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { ContentConfig } from "@/api/config/types";
    import {i18n} from "@/i18n/index.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ContentConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.content')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.content_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('settings.metadata_provider')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.metadata_provider_desc')}</p>
        </div>
        <Select.Root type="single" bind:value={config.preferredMetadataProvider} onValueChange={onSave}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">
                {config.preferredMetadataProvider === 'myanimelist' ? 'MyAnimeList' : config.preferredMetadataProvider}
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="anilist">AniList</Select.Item>
                <Select.Item value="myanimelist">MyAnimeList</Select.Item>
                <Select.Item value="kitsu">Kitsu</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="autoUpdateProgress">{i18n.t('settings.auto_update_progress')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.auto_update_progress_desc')}</p>
        </div>
        <Switch id="autoUpdateProgress" bind:checked={config.autoUpdateProgress} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>