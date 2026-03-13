<script lang="ts">
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { NotificationsConfig } from "@/api/config/types";
    import { i18n } from "@/i18n/index.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: NotificationsConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.notifications')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.notifications_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="notificationsEnabled">{i18n.t('settings.enable_notifications')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.enalbe_notifications_desc')}</p>
        </div>
        <Switch id="notificationsEnabled" bind:checked={config.enabled} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40 transition-opacity {config.enabled ? 'opacity-100' : 'opacity-50'}">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="notifyNewEpisodes">{i18n.t('settings.new_releases')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.new_releases_desc')}</p>
        </div>
        <Switch
                id="notifyNewEpisodes"
                bind:checked={config.notifyNewEpisodes}
                disabled={!config.enabled}
                onCheckedChange={onSave}
                class="shrink-0"
        />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40 transition-opacity {config.enabled ? 'opacity-100' : 'opacity-50'}">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="notifyStatusChanges">{i18n.t('settings.status_changes')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.status_changes_desc')}</p>
        </div>
        <Switch
                id="notifyStatusChanges"
                bind:checked={config.notifyStatusChanges}
                disabled={!config.enabled}
                onCheckedChange={onSave}
                class="shrink-0"
        />
    </div>
</section>