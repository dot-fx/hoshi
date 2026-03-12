<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { UiConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: UiConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">Interface</h2>
        <p class="text-sm text-muted-foreground mt-1">Customize the layout and default behavior of the application.</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">Default Home Section</Label>
            <p class="text-sm text-muted-foreground">The primary content type to show when you first open the app.</p>
        </div>
        <Select.Root type="single" bind:value={config.defaultHomeSection} onValueChange={onSave}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">
                {config.defaultHomeSection}
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="anime">Anime</Select.Item>
                <Select.Item value="manga">Manga</Select.Item>
                <Select.Item value="novel">Novel</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="disableCardTrailers">Disable Card Trailers</Label>
            <p class="text-sm text-muted-foreground">Prevent YouTube trailers from auto-playing when hovering over content cards.</p>
        </div>
        <Switch id="disableCardTrailers" bind:checked={config.disableCardTrailers} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>