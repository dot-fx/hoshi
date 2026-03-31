<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Drawer from "$lib/components/ui/drawer";
    import { toast } from "svelte-sonner";
    import { Trash2, Settings2, X } from "lucide-svelte";

    import type { ExtensionsConfig } from "@/api/config/types";
    import type { Extension } from "@/api/extensions/types";
    import { extensionsApi } from "@/api/extensions/extensions";
    import { extensions } from "$lib/extensions.svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import ExtensionSettingsForm from "./ExtensionSettingsForm.svelte";
    import {Spinner} from "@/components/ui/spinner";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let uninstallingIds = $state<Set<string>>(new Set());
    let savingIds = $state<Set<string>>(new Set());
    let expandedSettings = $state<Record<string, boolean>>({});
    let drawerSettings = $state<Record<string, boolean>>({});
    let draftSettings = $state<Record<string, Record<string, any>>>({});

    $effect(() => {
        for (const ext of extensions.installed) {
            const isOpen = expandedSettings[ext.id] || drawerSettings[ext.id];
            if (isOpen && !draftSettings[ext.id]) {
                const initialSettings: Record<string, any> = { ...ext.settings };
                if (ext.setting_definitions) {
                    ext.setting_definitions.forEach(def => {
                        if (initialSettings[def.key] === undefined) {
                            initialSettings[def.key] = Array.isArray(def.default)
                                ? [...def.default]
                                : def.default;
                        }
                    });
                }
                draftSettings[ext.id] = initialSettings;
            }
        }
    });

    async function handleUninstall(id: string) {
        uninstallingIds = new Set(uninstallingIds).add(id);
        try {
            await extensions.uninstall(id);
            toast.success(i18n.t('settings.extension_section.extension_uninstalled'));
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
        } finally {
            const newSet = new Set(uninstallingIds);
            newSet.delete(id);
            uninstallingIds = newSet;
        }
    }

    async function handleSaveSettings(ext: Extension) {
        savingIds = new Set(savingIds).add(ext.id);
        try {
            const newSettings = draftSettings[ext.id];
            const res = await extensionsApi.updateSettings(ext.id, newSettings);
            if (res.ok) {
                const index = extensions.installed.findIndex(e => e.id === ext.id);
                if (index !== -1) {
                    extensions.installed[index].settings = { ...newSettings };
                }
                toast.success(i18n.t('settings.extension_section.changes_updated'));
                expandedSettings[ext.id] = false;
                drawerSettings[ext.id] = false;
            }
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
        } finally {
            const newSet = new Set(savingIds);
            newSet.delete(ext.id);
            savingIds = newSet;
        }
    }

    function getTypeColor(type: string) {
        const t = (type || "").toLowerCase();
        switch(t) {
            case 'anime': return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
            case 'manga': return 'bg-green-500/10 text-green-500 border-green-500/20';
            case 'novel': return 'bg-purple-500/10 text-purple-500 border-purple-500/20';
            default: return 'bg-muted text-muted-foreground border-border';
        }
    }
</script>

<div class="space-y-6">

    <div class="space-y-4">
        {#if extensions.loading && extensions.installed.length === 0}
            <div class="flex justify-center py-8">
                <Spinner class="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        {:else if extensions.installed.length === 0}
            <div class="py-12 text-center border border-dashed border-border/40 rounded-2xl bg-muted/5">
                <p class="text-muted-foreground font-medium">{i18n.t('settings.extension_section.no_installed')}</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                {#each extensions.installed as ext (ext.id)}
                    <div class="flex flex-col rounded-xl border border-border/60 bg-card overflow-hidden transition-all shadow-sm">
                        <div class="flex items-center p-3 gap-3">
                            <Avatar.Root class="relative h-10 w-10 rounded-lg border border-border/50 shrink-0 bg-muted/30 overflow-hidden flex items-center justify-center">

                                <div data-fallback class="bg-primary/10 text-primary font-black rounded-lg text-xs w-full h-full flex items-center justify-center absolute inset-0 z-0">
                                    {ext.name.slice(0, 2).toUpperCase()}
                                </div>

                                {#if ext.icon}
                                    <img
                                            src={ext.icon}
                                            alt={ext.name}
                                            class="object-cover w-full h-full absolute inset-0 z-10"
                                            onload={(e) => {
                const fallback = e.currentTarget.parentElement?.querySelector('[data-fallback]');
                if (fallback) fallback.style.display = 'none';
            }}
                                            onerror={(e) => {
                e.currentTarget.style.display = 'none';
            }}
                                    />
                                {/if}
                            </Avatar.Root>

                            <div class="space-y-0.5 flex-1 min-w-0">
                                <div class="flex items-center gap-2">
                                    <h3 class="font-bold text-sm truncate">{ext.name}</h3>

                                    <div class="flex items-center gap-1 shrink-0">
                                        <Badge variant="outline" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 {getTypeColor(ext.ext_type)}">
                                            {ext.ext_type}
                                        </Badge>

                                        {#if ext.language}
                                            <Badge variant="secondary" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 bg-muted/80 text-muted-foreground">
                                                {ext.language}
                                            </Badge>
                                        {/if}
                                    </div>
                                </div>

                                <div class="flex items-center gap-1.5 text-[11px] font-semibold text-muted-foreground/80 mt-0.5">
                                    <span>v{ext.version}</span>
                                    {#if ext.author}
                                        <span class="opacity-50">•</span>
                                        <span class="truncate">{ext.author}</span>
                                    {/if}
                                </div>
                            </div>

                            <div class="flex gap-1.5 shrink-0 items-center">
                                {#if ext.setting_definitions && ext.setting_definitions.length > 0}
                                    <Button
                                            variant={expandedSettings[ext.id] ? "default" : "secondary"}
                                            size="sm"
                                            class="h-8 w-8 p-0 rounded-lg hidden md:flex"
                                            onclick={() => expandedSettings[ext.id] = !expandedSettings[ext.id]}
                                    >
                                        {#if expandedSettings[ext.id]}<X class="h-4 w-4" />{:else}<Settings2 class="h-4 w-4" />{/if}
                                    </Button>

                                    <div class="md:hidden">
                                        <Drawer.Root open={!!drawerSettings[ext.id]} onOpenChange={(v) => drawerSettings[ext.id] = v}>
                                            <Drawer.Trigger>
                                                {#snippet child({ props })}
                                                    <Button {...props} variant="secondary" size="sm" class="h-8 w-8 p-0 rounded-lg">
                                                        <Settings2 class="h-4 w-4" />
                                                    </Button>
                                                {/snippet}
                                            </Drawer.Trigger>
                                            <Drawer.Content class="h-[85vh] rounded-t-2xl border-border/50">
                                                <div class="p-5 overflow-y-auto hide-scrollbar">
                                                    <h3 class="font-black text-lg mb-6 tracking-tight flex items-center gap-2 border-b border-border/40 pb-4">
                                                        <Settings2 class="w-5 h-5 text-primary" />
                                                        {i18n.t('settings.extension_section.extension_config', { name: ext.name})}
                                                    </h3>
                                                    <ExtensionSettingsForm
                                                            {ext}
                                                            bind:settings={draftSettings[ext.id]}
                                                            isSaving={savingIds.has(ext.id)}
                                                            onSave={() => handleSaveSettings(ext)}
                                                    />
                                                </div>
                                            </Drawer.Content>
                                        </Drawer.Root>
                                    </div>
                                {/if}

                                <Button
                                        variant="destructive"
                                        size="sm"
                                        class="h-8 w-8 p-0 rounded-lg bg-destructive/10 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                                        onclick={() => handleUninstall(ext.id)}
                                        disabled={uninstallingIds.has(ext.id)}
                                >
                                    {#if uninstallingIds.has(ext.id)}<Spinner class="h-4 w-4 animate-spin" />{:else}<Trash2 class="h-4 w-4" />{/if}
                                </Button>
                            </div>
                        </div>

                        {#if expandedSettings[ext.id] && draftSettings[ext.id]}
                            <div class="p-4 border-t border-border/40 bg-muted/10 hidden md:block">
                                <ExtensionSettingsForm
                                        {ext}
                                        bind:settings={draftSettings[ext.id]}
                                        isSaving={savingIds.has(ext.id)}
                                        onSave={() => handleSaveSettings(ext)}
                                />
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>