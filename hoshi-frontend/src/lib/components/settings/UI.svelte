<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { UiConfig } from "@/api/config/types";
    import {i18n} from "@/i18n/index.svelte";

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
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.interface')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.interface')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold">{i18n.t('settings.default_home')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.default_home_desc')}</p>
        </div>
        <Select.Root type="single" bind:value={config.defaultHomeSection} onValueChange={onSave}>
            <Select.Trigger class="rounded-xl h-11 w-full sm:max-w-md capitalize">
                {config.defaultHomeSection}
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="anime">{i18n.t('settings.default_home_anime')}</Select.Item>
                <Select.Item value="manga">{i18n.t('settings.default_home_manga')}</Select.Item>
                <Select.Item value="novel">{i18n.t('settings.default_home_novel')}</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="disableCardTrailers">{i18n.t('settings.disable_card_trailers')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.disable_card_trailers_desc')}</p>
        </div>
        <Switch id="disableCardTrailers" bind:checked={config.disableCardTrailers} onCheckedChange={onSave} class="shrink-0" />
    </div>
</section>