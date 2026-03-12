<script lang="ts">
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import type { NotificationsConfig } from "@/api/config/types";

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
        <h2 class="text-2xl font-bold tracking-tight">Notifications</h2>
        <p class="text-sm text-muted-foreground mt-1">Manage how and when the app alerts you to new content.</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="notificationsEnabled">Enable Notifications</Label>
            <p class="text-sm text-muted-foreground">Master switch to allow or block all app notifications.</p>
        </div>
        <Switch id="notificationsEnabled" bind:checked={config.enabled} onCheckedChange={onSave} class="shrink-0" />
    </div>

    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40 transition-opacity {config.enabled ? 'opacity-100' : 'opacity-50'}">
        <div class="space-y-1 pr-4">
            <Label class="text-base font-bold" for="notifyNewEpisodes">New Releases</Label>
            <p class="text-sm text-muted-foreground">Get alerted when new episodes or chapters of your tracked content are released.</p>
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
            <Label class="text-base font-bold" for="notifyStatusChanges">Status Changes</Label>
            <p class="text-sm text-muted-foreground">Receive updates when a show or manga changes status (e.g., from Airing to Completed).</p>
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