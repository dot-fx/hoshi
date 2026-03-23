<script lang="ts">
    import { extensions as extensionsStore } from "$lib/extensions.svelte";
    import { extensionsApi } from "$lib/api/extensions/extensions";
    import { i18n } from "$lib/i18n/index.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { PuzzleIcon, Settings2, Mic2, Server } from "lucide-svelte";

    let { onSettingsChange } = $props<{
        onSettingsChange: (settings: { extension: string, server: string | null, isDub: boolean }) => void
    }>();

    let selectedExtension = $state<string | null>(null);
    let servers = $state<string[]>([]);
    let selectedServer = $state<string | null>(null);
    let supportsDub = $state(false);
    let isDub = $state(false);

    $effect(() => {
        extensionsStore.load().then(() => {
            if (extensionsStore.anime.length > 0 && !selectedExtension) {
                handleExtensionChange(extensionsStore.anime[0].id);
            }
        });
    });

    async function handleExtensionChange(extId: string) {
        selectedExtension = extId;
        servers = [];
        supportsDub = false;
        selectedServer = null;
        isDub = false;

        try {
            const s = await extensionsApi.getSettings(extId);
            servers = s.episodeServers ?? [];
            supportsDub = s.supportsDub ?? false;
            selectedServer = servers.length > 0 ? servers[0] : null;
            notifyChange();
        } catch (err) {
            console.error("Error loading extension settings", err);
        }
    }

    function notifyChange() {
        if (selectedExtension) {
            onSettingsChange({
                extension: selectedExtension,
                server: selectedServer,
                isDub: isDub
            });
        }
    }
</script>

<div class="flex flex-col gap-6 p-5">
    <div class="flex items-center gap-3 pb-4 border-b border-border/40">
        <div class="p-2.5 bg-primary/10 rounded-xl">
            <Settings2 class="w-5 h-5 text-primary" />
        </div>
        <div>
            <h3 class="font-black text-foreground text-lg leading-none mb-1">{i18n.t('watchparty.controls.settings_title')}</h3>
            <p class="text-xs font-medium text-muted-foreground">{i18n.t('watchparty.controls.settings_desc')}</p>
        </div>
    </div>

    <div class="space-y-5">
        <div class="space-y-2.5">
            <Label class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5 ml-1">
                <PuzzleIcon class="w-3.5 h-3.5" /> {i18n.t('watchparty.controls.extension')}
            </Label>
            <Select.Root type="single" value={selectedExtension ?? ""} onValueChange={handleExtensionChange}>
                <Select.Trigger class="h-12 bg-muted/20 hover:bg-muted/40 transition-colors border-border/50 rounded-xl font-semibold px-4">
                    <span class="truncate text-sm">{selectedExtension || i18n.t('watchparty.controls.select_extension')}</span>
                </Select.Trigger>
                <Select.Content class="rounded-xl border-border/40 bg-card/95 backdrop-blur-xl">
                    {#each extensionsStore.anime as ext}
                        <Select.Item value={ext.id} label={ext.name} class="rounded-lg font-medium cursor-pointer">
                            {ext.name}
                        </Select.Item>
                    {/each}
                </Select.Content>
            </Select.Root>
        </div>

        {#if servers.length > 0}
            <div class="space-y-2.5">
                <Label class="text-xs font-bold uppercase tracking-wider text-muted-foreground flex items-center gap-1.5 ml-1">
                    <Server class="w-3.5 h-3.5" /> {i18n.t('watchparty.controls.video_server')}
                </Label>
                <Select.Root type="single" value={selectedServer ?? ""} onValueChange={(v) => { selectedServer = v; notifyChange(); }}>
                    <Select.Trigger class="h-12 bg-muted/20 hover:bg-muted/40 transition-colors border-border/50 rounded-xl font-semibold px-4">
                        <span class="truncate text-sm">{selectedServer || i18n.t('watchparty.controls.server_auto')}</span>
                    </Select.Trigger>
                    <Select.Content class="rounded-xl border-border/40 bg-card/95 backdrop-blur-xl">
                        {#each servers as srv}
                            <Select.Item value={srv} label={srv} class="rounded-lg font-medium cursor-pointer">
                                {srv}
                            </Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>
        {/if}

        {#if supportsDub}
            <div class="flex items-center justify-between p-4 bg-muted/20 hover:bg-muted/30 transition-colors rounded-xl border border-border/50 mt-2">
                <div class="flex flex-col gap-1">
                    <Label class="text-sm font-bold flex items-center gap-2 cursor-pointer" for="dub-mode">
                        <Mic2 class="w-4 h-4 text-primary" /> {i18n.t('watchparty.controls.prioritize_dub')}
                    </Label>
                    <span class="text-xs font-medium text-muted-foreground">{i18n.t('watchparty.controls.dub_desc')}</span>
                </div>
                <Switch id="dub-mode" checked={isDub} onCheckedChange={(v) => { isDub = v; notifyChange(); }} />
            </div>
        {/if}
    </div>
</div>