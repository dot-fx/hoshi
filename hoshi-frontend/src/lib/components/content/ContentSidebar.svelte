<script lang="ts">
    import * as Card from "$lib/components/ui/card";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import { Calendar, Building2, Database, Hash, Pencil } from "lucide-svelte";

    // Asumiendo que guardaste el modal en esta ruta
    import TrackerManagerModal from "$lib/components/content/TrackerManagerModal.svelte";

    let { metadata, trackers }: { metadata: any, trackers: any[] } = $props();

    let showTrackerModal = $state(false);

    function formatDate(dateStr?: string | null) {
        if (!dateStr) return "TBA";
        return new Date(dateStr).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
    }

    // Mapa de dominios para obtener los iconos (favicons)
    const trackerDomains: Record<string, string> = {
        anilist: 'anilist.co',
        myanimelist: 'myanimelist.net',
        mal: 'myanimelist.net',
        simkl: 'simkl.com',
        anidb: 'anidb.net',
        kitsu: 'kitsu.io',
        trakt: 'trakt.tv',
        trakttvslug: 'trakt.tv',
        animeplanet: 'anime-planet.com',
        imdb: 'imdb.com',
        tmdb: 'themoviedb.org',
        tvdb: 'thetvdb.com'
    };

    function getFavicon(key: string) {
        const domain = trackerDomains[key.toLowerCase()] || `${key}.com`;
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }

    function getPlatformStyle(key: string) {
        const platforms: Record<string, { label: string, color: string }> = {
            anilist: { label: 'AniList', color: 'text-[#3db4f2] border-[#3db4f2]/30 bg-[#3db4f2]/5' },
            myanimelist: { label: 'MAL', color: 'text-[#2e51a2] border-[#2e51a2]/30 bg-[#2e51a2]/5' },
            mal: { label: 'MAL', color: 'text-[#2e51a2] border-[#2e51a2]/30 bg-[#2e51a2]/5' },
            simkl: { label: 'Simkl', color: 'text-[#ff9800] border-[#ff9800]/30 bg-[#ff9800]/5' },
            anidb: { label: 'AniDB', color: 'text-[#002147] border-[#002147]/40 bg-[#002147]/10' },
            kitsu: { label: 'Kitsu', color: 'text-[#ef5a42] border-[#ef5a42]/30 bg-[#ef5a42]/5' },
            trakt: { label: 'Trakt', color: 'text-[#ed1c24] border-[#ed1c24]/30 bg-[#ed1c24]/5' },
            animeplanet: { label: 'Anime-Planet', color: 'text-[#9333ea] border-[#9333ea]/30 bg-[#9333ea]/5' },
            imdb: { label: 'IMDb', color: 'text-[#f5c518] border-[#f5c518]/30 bg-[#f5c518]/5' },
            tmdb: { label: 'TMDB', color: 'text-[#01b4e4] border-[#01b4e4]/30 bg-[#01b4e4]/5' },
            tvdb: { label: 'TVDB', color: 'text-[#376ad4] border-[#376ad4]/30 bg-[#376ad4]/5' },
        };
        const normalizedKey = key.toLowerCase().replace('trakttvslug', 'trakt');
        return platforms[normalizedKey] || {
            label: key.toUpperCase(),
            color: 'text-muted-foreground border-border/50 bg-muted/20'
        };
    }
</script>

<div class="space-y-6 sticky top-24">
    <Card.Root class="bg-card/50 backdrop-blur-sm border-border/50 shadow-sm">
        <Card.Header>
            <Card.Title class="text-lg font-bold tracking-tight">Information</Card.Title>
        </Card.Header>
        <Card.Content class="space-y-4 text-sm">
            <div class="flex items-start justify-between">
                <span class="text-muted-foreground flex items-center gap-2"><Building2 class="h-4 w-4"/> Studio</span>
                <span class="font-medium text-right">{metadata.studio || "Unknown"}</span>
            </div>
            <Separator class="bg-border/50" />
            <div class="flex items-start justify-between">
                <span class="text-muted-foreground flex items-center gap-2"><Calendar class="h-4 w-4"/> Aired</span>
                <span class="font-medium text-right">{formatDate(metadata.releaseDate)}</span>
            </div>
            <div class="flex items-start justify-between">
                <span class="text-muted-foreground flex items-center gap-2"><Calendar class="h-4 w-4"/> Ended</span>
                <span class="font-medium text-right">{formatDate(metadata.endDate)}</span>
            </div>
            {#if metadata.nsfw}
                <Separator class="bg-border/50" />
                <div class="flex justify-end">
                    <Badge variant="destructive" class="text-xs">18+ NSFW</Badge>
                </div>
            {/if}
        </Card.Content>
    </Card.Root>

    <div class="space-y-3">
        <div class="flex items-center justify-between">
            <h3 class="font-semibold text-[11px] uppercase tracking-widest text-muted-foreground/60 flex items-center gap-2">
                <Database class="h-3 w-3" /> Trackers
            </h3>
            <button
                    class="text-muted-foreground hover:text-primary transition-colors p-1 rounded-md hover:bg-muted/50"
                    onclick={() => showTrackerModal = true}
                    aria-label="Manage Trackers"
            >
                <Pencil class="h-3.5 w-3.5" />
            </button>
        </div>

        {#if trackers && trackers.length > 0}
            <div class="flex flex-col gap-2">
                {#each trackers as tracker}
                    {@const style = getPlatformStyle(tracker.trackerName)}
                    <a href={tracker.trackerUrl || '#'} target={tracker.trackerUrl ? "_blank" : "_self"} rel="noopener noreferrer" class="block group">
                        <Badge variant="outline" class="w-full justify-between py-2 px-3 transition-all {style.color} group-hover:brightness-110">
                            <div class="flex items-center gap-2">
                                <img src={getFavicon(tracker.trackerName)} alt={style.label} class="w-4 h-4 rounded-sm bg-white/80 p-0.5" />
                                <span class="font-bold text-xs">{style.label}</span>
                            </div>
                            <span class="text-[10px] font-mono opacity-60 bg-foreground/5 px-1.5 py-0.5 rounded">
                                {tracker.trackerId}
                            </span>
                        </Badge>
                    </a>
                {/each}
            </div>
        {:else}
            <p class="text-xs text-muted-foreground border border-dashed rounded-lg p-4 text-center">No trackers assigned.</p>
        {/if}
    </div>

    {#if metadata.genres && metadata.genres.length > 0}
        <div class="space-y-3">
            <h3 class="font-semibold text-sm text-foreground">Genres</h3>
            <div class="flex flex-wrap gap-2">
                {#each metadata.genres as genre}
                    <Badge variant="secondary" class="hover:bg-primary hover:text-primary-foreground transition-colors cursor-default">
                        {genre}
                    </Badge>
                {/each}
            </div>
        </div>
    {/if}

    {#if metadata.tags && metadata.tags.length > 0}
        <div class="space-y-3">
            <h3 class="font-semibold text-sm text-foreground">Themes & Tags</h3>
            <div class="flex flex-wrap gap-1.5">
                {#each metadata.tags.slice(0, 8) as tag}
                    <Badge variant="outline" class="text-xs font-normal text-muted-foreground hover:text-foreground">
                        {tag}
                    </Badge>
                {/each}
                {#if metadata.tags.length > 8}
                    <Badge variant="outline" class="text-xs font-normal text-muted-foreground border-dashed">
                        +{metadata.tags.length - 8} more
                    </Badge>
                {/if}
            </div>
        </div>
    {/if}

    {#if metadata.externalIds && Object.keys(metadata.externalIds).length > 0}
        <div class="space-y-3 pt-4 border-t border-border/40">
            <h3 class="font-semibold text-[11px] uppercase tracking-widest text-muted-foreground/60 flex items-center gap-2">
                <Hash class="h-3 w-3" /> External IDs
            </h3>
            <div class="grid grid-cols-2 gap-2">
                {#each Object.entries(metadata.externalIds) as [key, value]}
                    {#if value && !['slug', 'anilist', 'mal', 'simkl', 'trakt', 'kitsu', 'anidb'].includes(key.toLowerCase())}
                        {@const style = getPlatformStyle(key)}
                        <div class="flex flex-col p-2 rounded-md border transition-colors {style.color} overflow-hidden">
                            <span class="text-[9px] font-black uppercase tracking-tight opacity-70">
                                {style.label}
                            </span>
                            <span class="text-[11px] font-mono truncate select-all font-medium">
                                {value}
                            </span>
                        </div>
                    {/if}
                {/each}
            </div>
        </div>
    {/if}
</div>

<TrackerManagerModal
        bind:open={showTrackerModal}
        cid={metadata.cid}
        {trackers}
/>