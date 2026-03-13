<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { i18n } from "$lib/i18n/index.svelte";

    let {
        searchMode,
        dbStatus = $bindable(),
        dbGenre = $bindable(),
        dbFormat = $bindable(),
        dbNsfw = $bindable(),
        extFiltersSchema,
        extFilterValues = $bindable(),
        onClear
    }: {
        searchMode: "database" | "extension";
        dbStatus: string;
        dbGenre: string;
        dbFormat: string;
        dbNsfw: boolean;
        extFiltersSchema: Record<string, any>;
        extFilterValues: Record<string, any>;
        onClear: () => void;
    } = $props();

    const formatLabel = (key: string) => key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());

    const toggleMultiSelect = (key: string, value: string) => {
        if (!extFilterValues[key]) extFilterValues[key] = [];
        const index = extFilterValues[key].indexOf(value);
        if (index > -1) {
            extFilterValues[key] = extFilterValues[key].filter((v: string) => v !== value);
        } else {
            extFilterValues[key] = [...extFilterValues[key], value];
        }
    };
</script>

<div class="space-y-6 w-full">
    {#if searchMode === "database"}
        <div class="space-y-5">
            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.status')}</Label>
                <Select.Root type="single" bind:value={dbStatus}>
                    <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                        {dbStatus ? i18n.t(dbStatus.toLowerCase()) || dbStatus : i18n.t('search.any_status')}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="">{i18n.t('search.any_status')}</Select.Item>
                        <Select.Item value="Completed">{i18n.t('search.completed')}</Select.Item>
                        <Select.Item value="Ongoing">{i18n.t('search.ongoing')}</Select.Item>
                        <Select.Item value="Planned">{i18n.t('search.planned')}</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>
            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.genre')}</Label>
                <Select.Root type="single" bind:value={dbGenre}>
                    <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                        {dbGenre ? i18n.t(dbGenre.toLowerCase().replace('-', '_')) || dbGenre : i18n.t('search.any_genre')}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="">{i18n.t('search.any_genre')}</Select.Item>
                        <Select.Item value="Action">{i18n.t('search.action')}</Select.Item>
                        <Select.Item value="Romance">{i18n.t('search.romance')}</Select.Item>
                        <Select.Item value="Fantasy">{i18n.t('search.fantasy')}</Select.Item>
                        <Select.Item value="Sci-Fi">{i18n.t('search.sci_fi')}</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>
            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.format')}</Label>
                <Select.Root type="single" bind:value={dbFormat}>
                    <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                        {dbFormat ? i18n.t(dbFormat.toLowerCase()) || dbFormat : i18n.t('search.any_format')}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="">{i18n.t('search.any_format')}</Select.Item>
                        <Select.Item value="TV">{i18n.t('search.tv')}</Select.Item>
                        <Select.Item value="MOVIE">{i18n.t('search.movie')}</Select.Item>
                        <Select.Item value="OVA">{i18n.t('search.ova')}</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>
            <div class="flex items-center space-x-3 pt-2">
                <Switch id="nsfw-mode" bind:checked={dbNsfw} />
                <Label for="nsfw-mode" class="text-sm font-bold text-foreground/90">{i18n.t('search.nsfw_only')}</Label>
            </div>
        </div>

    {:else if searchMode === "extension" && Object.keys(extFiltersSchema).length > 0}
        <div class="space-y-5">
            {#each Object.entries(extFiltersSchema) as [key, filterDef]}
                <div class="space-y-2.5">
                    <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>

                    {#if filterDef.type === 'select'}
                        <Select.Root type="single" bind:value={extFilterValues[key]}>
                            <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                                {filterDef.options.find((o) => o.value === extFilterValues[key])?.label || i18n.t('search.select')}
                            </Select.Trigger>
                            <Select.Content class="max-h-[300px]">
                                {#each filterDef.options as option}
                                    <Select.Item value={String(option.value)}>{option.label}</Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>

                    {:else if filterDef.type === 'multiselect'}
                        <div class="flex flex-wrap gap-2 pt-1">
                            {#each filterDef.options as option}
                                <button
                                        type="button"
                                        class="px-3.5 py-1.5 text-xs font-bold rounded-lg border transition-colors shadow-sm
                                    {extFilterValues[key]?.includes(option.value)
                                        ? 'bg-primary text-primary-foreground border-primary'
                                        : 'bg-background hover:bg-muted border-border/60'}"
                                        onclick={() => toggleMultiSelect(key, option.value)}
                                >
                                    {option.label}
                                </button>
                            {/each}
                        </div>

                    {:else if filterDef.type === 'boolean'}
                        <div class="flex items-center space-x-3 pt-2">
                            <Switch id={`filter-${key}`} bind:checked={extFilterValues[key]} />
                            <Label for={`filter-${key}`} class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                        </div>

                    {:else}
                        <Input
                                type="text"
                                placeholder={i18n.t('search.enter_filter', { filter: filterDef.label?.toLowerCase() || formatLabel(key).toLowerCase() })}
                                class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50"
                                bind:value={extFilterValues[key]}
                        />
                    {/if}
                </div>
            {/each}
        </div>
    {:else}
        <div class="py-8 text-center bg-muted/5 rounded-xl border border-dashed border-border/50">
            <p class="text-muted-foreground text-sm font-medium">{i18n.t('search.no_filters')}</p>
        </div>
    {/if}

    <div class="pt-6 border-t border-border/40">
        <Button type="button" variant="secondary" class="w-full h-11 rounded-xl font-bold hover:bg-destructive hover:text-destructive-foreground transition-colors" onclick={onClear}>
            {i18n.t('search.clear_filters')}
        </Button>
    </div>
</div>