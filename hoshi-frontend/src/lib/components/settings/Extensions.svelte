<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import * as Avatar from "$lib/components/ui/avatar";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { toast } from "svelte-sonner";
    import { Trash2, Settings2, Save, Loader2, X } from "lucide-svelte";

    import type { ExtensionsConfig } from "@/api/config/types";
    import type { Extension } from "@/api/extensions/types";
    import { extensionsApi } from "@/api/extensions/extensions";
    import { extensions } from "$lib/extensions.svelte";
    import { i18n } from "$lib/i18n/index.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    // --- ESTADOS LOCALES ---
    let uninstallingIds = $state<Set<string>>(new Set());
    let savingIds = $state<Set<string>>(new Set());
    let expandedSettings = $state<Set<string>>(new Set());
    let draftSettings = $state<Record<string, Record<string, any>>>({});

    $effect(() => {
        extensions.load();
    });

    // --- FUNCIONES ---
    async function handleUninstall(id: string) {
        uninstallingIds = new Set(uninstallingIds).add(id);
        try {
            await extensions.uninstall(id);
            toast.success(i18n.t('settings.extension_uninstalled'));
        } catch (error: any) {
            toast.error(error?.message);
        } finally {
            const newSet = new Set(uninstallingIds);
            newSet.delete(id);
            uninstallingIds = newSet;
        }
    }

    function toggleSettings(ext: Extension) {
        const newExpanded = new Set(expandedSettings);
        if (newExpanded.has(ext.id)) {
            newExpanded.delete(ext.id);
        } else {
            newExpanded.add(ext.id);
            // Inicializar un borrador con los ajustes actuales O los valores por defecto
            if (!draftSettings[ext.id]) {
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
        expandedSettings = newExpanded;
    }

    async function handleSaveSettings(ext: Extension) {
        savingIds = new Set(savingIds).add(ext.id);
        try {
            const newSettings = draftSettings[ext.id];
            const res = await extensionsApi.updateSettings(ext.id, newSettings);
            if (res.ok) {
                // Actualizar el estado global con las nuevas settings
                const index = extensions.installed.findIndex(e => e.id === ext.id);
                if (index !== -1) {
                    extensions.installed[index].settings = { ...newSettings };
                }
                toast.success(i18n.t('settings.changes_updated'));

                // Cerrar el panel
                const newExpanded = new Set(expandedSettings);
                newExpanded.delete(ext.id);
                expandedSettings = newExpanded;
            }
        } catch (error: any) {
            toast.error(error?.message);
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
            case 'booru': return 'bg-orange-500/10 text-orange-500 border-orange-500/20';
            default: return 'bg-muted text-muted-foreground border-border';
        }
    }
</script>

<section class="space-y-8">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.extensions')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.extensions_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold" for="repoUrl">{i18n.t('settings.repo_url')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.repo_url_desc')}</p>
        </div>
        <div class="w-full sm:max-w-md space-y-3">
            <Input
                    id="repoUrl"
                    bind:value={config.repoUrl}
                    placeholder="https://raw.githubusercontent.com/..."
                    class="rounded-xl h-11"
            />
            <div class="flex justify-end">
                <Button variant="secondary" size="sm" class="rounded-lg font-bold" onclick={onSave}>
                    {i18n.t('settings.update_repo')}
                </Button>
            </div>
        </div>
    </div>

    <div class="space-y-4">
        <h3 class="text-lg font-bold">{i18n.t('settings.installed_extensions')}</h3>

        {#if extensions.loading && extensions.installed.length === 0}
            <div class="flex justify-center py-8">
                <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        {:else if extensions.installed.length === 0}
            <div class="py-12 text-center border border-dashed border-border/40 rounded-2xl bg-muted/5">
                <p class="text-muted-foreground font-medium">{i18n.t('settings.no_installed')}</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {#each extensions.installed as ext (ext.id)}
                    <div class="flex flex-col rounded-2xl border border-border/60 bg-card overflow-hidden transition-all">

                        <div class="flex items-start p-5 gap-4">
                            <Avatar.Root class="h-12 w-12 rounded-xl border border-border/50 shrink-0 bg-muted/30">
                                {#if ext.icon}<Avatar.Image src={ext.icon} alt={ext.name} class="object-cover" />{/if}
                                <Avatar.Fallback class="bg-primary/10 text-primary font-black rounded-xl">{ext.name.slice(0, 2).toUpperCase()}</Avatar.Fallback>
                            </Avatar.Root>

                            <div class="space-y-0.5 flex-1 min-w-0">
                                <div class="flex items-center justify-between gap-2">
                                    <h3 class="font-black text-base truncate">{ext.name}</h3>
                                    <Badge variant="outline" class="text-[10px] uppercase font-black tracking-wider h-5 shrink-0 {getTypeColor(ext.ext_type)}">{ext.ext_type}</Badge>
                                </div>
                                <div class="flex items-center gap-1.5 text-xs font-bold text-muted-foreground/80">
                                    <span>v{ext.version}</span>
                                    <span>•</span>
                                    <span class="truncate">{ext.author}</span>
                                </div>
                            </div>
                        </div>

                        <div class="px-5 pb-5 flex gap-2 justify-end">
                            {#if ext.setting_definitions && ext.setting_definitions.length > 0}
                                <Button
                                        variant={expandedSettings.has(ext.id) ? "default" : "secondary"}
                                        size="sm"
                                        class="rounded-xl font-bold"
                                        onclick={() => toggleSettings(ext)}
                                >
                                    {#if expandedSettings.has(ext.id)}
                                        <X class="h-4 w-4 mr-2" /> {i18n.t('settings.close')}
                                    {:else}
                                        <Settings2 class="h-4 w-4 mr-2" /> {i18n.t('settings.title')}
                                    {/if}
                                </Button>
                            {/if}

                            <Button
                                    variant="destructive"
                                    size="sm"
                                    class="rounded-xl font-bold bg-destructive/10 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                                    onclick={() => handleUninstall(ext.id)}
                                    disabled={uninstallingIds.has(ext.id)}
                            >
                                {#if uninstallingIds.has(ext.id)}<Loader2 class="h-4 w-4 animate-spin" />{:else}<Trash2 class="h-4 w-4" />{/if}
                            </Button>
                        </div>

                        {#if expandedSettings.has(ext.id) && draftSettings[ext.id]}
                            <div class="p-5 border-t border-border/40 bg-muted/10 space-y-4">
                                {#each ext.setting_definitions as def}
                                    <div class="space-y-1.5">
                                        <Label class="text-sm font-bold">{def.label || def.key}</Label>

                                        {#if def.type === 'string'}
                                            <Input
                                                    type="text"
                                                    bind:value={draftSettings[ext.id][def.key]}
                                                    placeholder={`${i18n.t('settings.enter')} ${def.label || def.key}`}
                                                    class="rounded-lg bg-background"
                                            />
                                        {:else if def.type === 'number'}
                                            <Input
                                                    type="number"
                                                    bind:value={draftSettings[ext.id][def.key]}
                                                    class="rounded-lg bg-background"
                                            />
                                        {:else if def.type === 'boolean'}
                                            <div class="flex items-center space-x-2 mt-2">
                                                <Switch
                                                        id={`switch-${ext.id}-${def.key}`}
                                                        bind:checked={draftSettings[ext.id][def.key]}
                                                />
                                                <Label
                                                        for={`switch-${ext.id}-${def.key}`}
                                                        class="text-sm font-medium text-muted-foreground cursor-pointer"
                                                >
                                                    {i18n.t('settings.enabled')}
                                                </Label>
                                            </div>
                                        {:else if def.type === 'select' && def.options}
                                            <Select.Root type="single" bind:value={draftSettings[ext.id][def.key]}>
                                                <Select.Trigger class="w-full bg-background rounded-lg border-border">
                                                    {def.options.find(o => o.value === draftSettings[ext.id][def.key])?.label || i18n.t('settings.select_option', { defaultValue: 'Select an option' })}
                                                </Select.Trigger>
                                                <Select.Content>
                                                    {#each def.options as option}
                                                        <Select.Item value={String(option.value)}>{option.label}</Select.Item>
                                                    {/each}
                                                </Select.Content>
                                            </Select.Root>
                                        {:else if def.type === 'multiselect' && def.options}
                                            <div class="flex flex-wrap gap-2 pt-1">
                                                {#each def.options as option}
                                                    <button
                                                            type="button"
                                                            class="px-3 py-1.5 text-xs font-bold rounded-lg border transition-colors shadow-sm {
                                                            (draftSettings[ext.id][def.key] || []).includes(option.value)
                                                                ? 'bg-primary text-primary-foreground border-primary'
                                                                : 'bg-background hover:bg-muted border-border/60'
                                                        }"
                                                            onclick={() => {
                                                            const currentVals = draftSettings[ext.id][def.key] || [];
                                                            if (currentVals.includes(option.value)) {
                                                                draftSettings[ext.id][def.key] = currentVals.filter(v => v !== option.value);
                                                            } else {
                                                                draftSettings[ext.id][def.key] = [...currentVals, option.value];
                                                            }
                                                        }}
                                                    >
                                                        {option.label}
                                                    </button>
                                                {/each}
                                            </div>
                                        {:else}
                                            <p class="text-xs text-muted-foreground">{i18n.t('settings.unsupported_type')} {def.type}</p>
                                        {/if}
                                    </div>
                                {/each}

                                <div class="pt-2 flex justify-end">
                                    <Button
                                            size="sm"
                                            class="rounded-xl font-bold"
                                            onclick={() => handleSaveSettings(ext)}
                                            disabled={savingIds.has(ext.id)}
                                    >
                                        {#if savingIds.has(ext.id)}<Loader2 class="h-4 w-4 mr-2 animate-spin" />{:else}<Save class="h-4 w-4 mr-2" />{/if}
                                        {i18n.t('settings.save')}
                                    </Button>
                                </div>
                            </div>
                        {/if}

                    </div>
                {/each}
            </div>
        {/if}
    </div>
</section>