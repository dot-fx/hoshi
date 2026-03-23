<script lang="ts">
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { i18n } from "$lib/i18n/index.svelte";

    let {
        searchMode,
        dbTracker, // <-- Añadimos el tracker como propiedad
        dbStatus = $bindable(),
        dbGenre = $bindable(),
        dbFormat = $bindable(),
        dbNsfw = $bindable(),
        extFiltersSchema,
        extFilterValues = $bindable(),
        onClear
    }: {
        searchMode: "database" | "extension";
        dbTracker: "anilist" | "mal" | "kitsu";
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

    // Diccionarios con los valores exactos que espera cada API
    const TRACKER_FILTERS = {
        anilist: {
            status: [
                { value: "FINISHED", label: "search.completed" },
                { value: "RELEASING", label: "search.ongoing" },
                { value: "NOT_YET_RELEASED", label: "search.planned" }
            ],
            formats: [
                { value: "TV", label: "search.tv" },
                { value: "MOVIE", label: "search.movie" },
                { value: "OVA", label: "search.ova" }
            ],
            genres: [
                { value: "Action", label: "search.action" },
                { value: "Romance", label: "search.romance" },
                { value: "Fantasy", label: "search.fantasy" },
                { value: "Sci-Fi", label: "search.sci_fi" }
            ]
        },
        mal: {
            status: [
                { value: "complete", label: "search.completed" },
                { value: "airing", label: "search.ongoing" },
                { value: "upcoming", label: "search.planned" }
            ],
            formats: [
                { value: "tv", label: "search.tv" },
                { value: "movie", label: "search.movie" },
                { value: "ova", label: "search.ova" }
            ],
            genres: [
                { value: "1", label: "search.action" },      // MAL usa IDs numéricos
                { value: "22", label: "search.romance" },
                { value: "10", label: "search.fantasy" },
                { value: "24", label: "search.sci_fi" }
            ]
        },
        kitsu: {
            status: [
                { value: "finished", label: "search.completed" },
                { value: "current", label: "search.ongoing" },
                { value: "upcoming", label: "search.planned" }
            ],
            formats: [
                { value: "TV", label: "search.tv" },
                { value: "movie", label: "search.movie" },
                { value: "OVA", label: "search.ova" }
            ],
            genres: [
                { value: "action", label: "search.action" },
                { value: "romance", label: "search.romance" },
                { value: "fantasy", label: "search.fantasy" },
                { value: "sci-fi", label: "search.sci_fi" }
            ]
        }
    };

    let activeFilters = $derived(TRACKER_FILTERS[dbTracker] || TRACKER_FILTERS.anilist);

    // Limpiar los filtros si el usuario cambia de tracker, ya que los IDs no coinciden
    $effect(() => {
        const _tracker = dbTracker;
        dbStatus = "";
        dbGenre = "";
        dbFormat = "";
    });
</script>

<div class="space-y-6 w-full">
    {#if searchMode === "database"}
        <div class="space-y-5">
            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.status')}</Label>
                <Select.Root type="single" bind:value={dbStatus}>
                    <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                        {dbStatus ? activeFilters.status.find(f => f.value === dbStatus)?.label ? i18n.t(activeFilters.status.find(f => f.value === dbStatus)!.label) : dbStatus : i18n.t('search.any_status')}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="">{i18n.t('search.any_status')}</Select.Item>
                        {#each activeFilters.status as status}
                            <Select.Item value={status.value}>{i18n.t(status.label)}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.genre')}</Label>
                <Select.Root type="single" bind:value={dbGenre}>
                    <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                        {dbGenre ? activeFilters.genres.find(f => f.value === dbGenre)?.label ? i18n.t(activeFilters.genres.find(f => f.value === dbGenre)!.label) : dbGenre : i18n.t('search.any_genre')}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="">{i18n.t('search.any_genre')}</Select.Item>
                        {#each activeFilters.genres as genre}
                            <Select.Item value={genre.value}>{i18n.t(genre.label)}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            <div class="space-y-2.5">
                <Label class="text-sm font-bold text-foreground/90">{i18n.t('search.format')}</Label>
                <Select.Root type="single" bind:value={dbFormat}>
                    <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                        {dbFormat ? activeFilters.formats.find(f => f.value === dbFormat)?.label ? i18n.t(activeFilters.formats.find(f => f.value === dbFormat)!.label) : dbFormat : i18n.t('search.any_format')}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="">{i18n.t('search.any_format')}</Select.Item>
                        {#each activeFilters.formats as format}
                            <Select.Item value={format.value}>{i18n.t(format.label)}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            <div class="flex items-center space-x-3 pt-2">
                <Switch id="nsfw-mode" bind:checked={dbNsfw} />
                <Label for="nsfw-mode" class="text-sm font-bold text-foreground/90">{i18n.t('search.nsfw_only')}</Label>
            </div>
        </div>

    {:else if searchMode === "extension" && Object.keys(extFiltersSchema).length > 0}
        <div class="space-y-4">
            {#each Object.entries(extFiltersSchema) as [key, filterDef]}
                <div class="space-y-2.5">
                    {#if filterDef.type === 'select'}
                        <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                        <Select.Root type="single" bind:value={extFilterValues[key]}>
                            <Select.Trigger class="w-full bg-muted/20 border-none h-11 rounded-xl text-sm font-semibold focus-visible:ring-1 focus-visible:ring-primary/50">
                                {extFilterValues[key] ? filterDef.options?.find((o: any) => o.value === extFilterValues[key])?.label || extFilterValues[key] : i18n.t('search.any_genre')}
                            </Select.Trigger>
                            <Select.Content>
                                <Select.Item value="">{i18n.t('search.any_genre')}</Select.Item>
                                {#each filterDef.options || [] as opt}
                                    <Select.Item value={opt.value}>{opt.label}</Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>

                    {:else if filterDef.type === 'multiselect'}
                        <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                        <div class="flex flex-wrap gap-2">
                            {#each filterDef.options || [] as opt}
                                <button
                                        type="button"
                                        class="px-3 py-1.5 text-xs font-semibold rounded-lg border transition-colors {extFilterValues[key]?.includes(opt.value) ? 'bg-primary text-primary-foreground border-primary' : 'bg-muted/20 border-border/50 text-muted-foreground hover:bg-muted/50'}"
                                        onclick={() => toggleMultiSelect(key, opt.value)}
                                >
                                    {opt.label}
                                </button>
                            {/each}
                        </div>

                    {:else if filterDef.type === 'boolean'}
                        <div class="flex items-center space-x-3 pt-2">
                            <Switch id={`filter-${key}`} bind:checked={extFilterValues[key]} />
                            <Label for={`filter-${key}`} class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
                        </div>

                    {:else}
                        <Label class="text-sm font-bold text-foreground/90">{filterDef.label || formatLabel(key)}</Label>
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