<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { ContentConfig } from "@/api/config/types";

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
        <h2 class="text-2xl font-bold tracking-tight">Content</h2>
        <p class="text-sm text-muted-foreground mt-1">Manage metadata sources and tracking behavior across all content types.</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">Metadata Provider</Label>
            <p class="text-sm text-muted-foreground">Primary source for titles, descriptions, and artwork.</p>
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
            <Label class="text-base font-bold" for="autoUpdateProgress">Auto-Update Progress</Label>
            <p class="text-sm text-muted-foreground">Automatically sync your watch and read progress with connected trackers.</p>
        </div>
        <Switch id="autoUpdateProgress" bind:checked={config.autoUpdateProgress} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>