<script lang="ts">
    import { Badge } from "$lib/components/ui/badge";
    import {
        Calendar, Building2, AlertTriangle, CalendarDays,
        Link as LinkIcon, ChevronDown, ChevronUp
    } from "lucide-svelte";
    import { i18n } from "$lib/i18n/index.svelte";

    let {
        metadata,
        trackerMappings = []
    }: {
        metadata: any;
        trackerMappings: any[];
    } = $props();

    let expanded = $state(false);
    const LIMIT = 8;
    const hasMore = $derived(trackerMappings.length > LIMIT);
    const displayedTrackers = $derived(expanded ? trackerMappings : trackerMappings.slice(0, LIMIT));

    function formatDate(dateStr?: string | null) {
        if (!dateStr) return i18n.t('content.tba');
        return new Date(dateStr).toLocaleDateString(i18n.locale || 'en-US', { year: 'numeric', month: 'short', day: 'numeric' });
    }

    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            anilist: 'anilist.co',
            myanimelist: 'myanimelist.net',
            mal: 'myanimelist.net',
            kitsu: 'kitsu.io',
            simkl: 'simkl.com',
            trakt: 'trakt.tv',
            anidb: 'anidb.net',
            animeplanet: 'anime-planet.com'
        };
        const domain = domains[trackerName.toLowerCase()] || `${trackerName}.com`;
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }

    function getTrackerLink(tracker: any) {
        if (tracker.trackerUrl) return tracker.trackerUrl;

        const id = tracker.trackerId;
        switch (tracker.trackerName.toLowerCase()) {
            case 'anilist': return `https://anilist.co/anime/${id}`;
            case 'anidb': return `https://anidb.net/anime/${id}`;
            case 'animeplanet': return `https://www.anime-planet.com/anime/${id}`;
            case 'myanimelist': return `https://myanimelist.net/anime/${id}`;
            default: return '#';
        }
    }
</script>

<div class="space-y-8 lg:top-28 self-start h-fit pb-10">

    <!-- Portada desktop -->
    <div class="hidden lg:block w-full aspect-[2/2.8] rounded-3xl overflow-hidden shadow-2xl bg-muted relative group">
        <img src={metadata?.coverImage} alt="Cover" class="w-full h-full object-cover transition-transform duration-700 ease-out group-hover:scale-105" />
    </div>

    <!-- Información -->
    <div class="space-y-4">
        <h3 class="font-bold text-lg tracking-tight text-foreground flex items-center gap-2">
            {i18n.t('content.information')}
        </h3>

        <div class="bg-muted/10 rounded-3xl border border-border/40 overflow-hidden shadow-sm divide-y divide-border/30">
            <div class="flex items-center justify-between p-5 hover:bg-muted/30 transition-colors">
                <span class="text-muted-foreground flex items-center gap-3 text-sm font-medium">
                    <Building2 class="h-4 w-4 text-primary/70"/>
                    {i18n.t('content.studio')}
                </span>
                <span class="font-semibold text-sm text-right text-foreground truncate max-w-[150px]" title={metadata.studio}>
                    {metadata.studio || i18n.t('content.tba')}
                </span>
            </div>

            <div class="flex items-center justify-between p-5 hover:bg-muted/30 transition-colors">
                <span class="text-muted-foreground flex items-center gap-3 text-sm font-medium">
                    <CalendarDays class="h-4 w-4 text-primary/70"/>
                    {i18n.t('content.aired')}
                </span>
                <span class="font-semibold text-sm text-right text-foreground">
                    {formatDate(metadata.releaseDate)}
                </span>
            </div>

            <div class="flex items-center justify-between p-5 hover:bg-muted/30 transition-colors">
                <span class="text-muted-foreground flex items-center gap-3 text-sm font-medium">
                    <Calendar class="h-4 w-4 text-primary/70"/>
                    {i18n.t('content.ended')}
                </span>
                <span class="font-semibold text-sm text-right text-foreground">
                    {formatDate(metadata.endDate)}
                </span>
            </div>
        </div>

        {#if metadata.nsfw}
            <div class="flex items-center justify-between p-5 bg-destructive/5 rounded-3xl border border-destructive/20">
                <span class="text-destructive flex items-center gap-3 text-sm font-bold">
                    <AlertTriangle class="h-4 w-4"/>
                    NSFW
                </span>
                <Badge variant="destructive" class="text-xs uppercase font-black tracking-wider px-3 py-1">18+</Badge>
            </div>
        {/if}
    </div>

    {#if trackerMappings.length > 0}
        <div class="space-y-4">
            <h3 class="font-bold text-lg tracking-tight text-foreground flex items-center gap-2">
                <LinkIcon class="h-5 w-5 text-primary" />
                {i18n.t('content.connected_trackers')}
            </h3>

            <div class="flex flex-col gap-2">
                {#each displayedTrackers as tracker}
                    <a
                            href={getTrackerLink(tracker)}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="group flex items-center justify-between p-3 rounded-2xl bg-muted/30 border border-border/50 hover:bg-primary/5 hover:border-primary/30 transition-all duration-200"
                    >
                        <div class="flex items-center gap-3 min-w-0">
                            <img
                                    src={getTrackerFavicon(tracker.trackerName)}
                                    alt={tracker.trackerName}
                                    class="w-5 h-5 rounded-md flex-shrink-0"
                            />
                            <div class="flex items-baseline gap-2 overflow-hidden">
                            <span class="font-semibold text-sm text-foreground capitalize truncate">
                                {tracker.trackerName}
                            </span>
                                <span class="text-[10px] font-mono text-muted-foreground/60 truncate">
                                #{tracker.trackerId}
                            </span>
                            </div>
                        </div>
                        <LinkIcon class="h-3 w-3 opacity-0 group-hover:opacity-100 text-primary/70 transition-opacity" />
                    </a>
                {/each}

                {#if hasMore}
                    <button
                            onclick={() => expanded = !expanded}
                            class="flex items-center justify-center gap-2 py-2 text-xs font-bold text-muted-foreground hover:text-primary transition-colors"
                    >
                        {#if expanded}
                            {i18n.t('content.show_less')} <ChevronUp class="h-4 w-4" />
                        {:else}
                            {i18n.t('content.show_more')} ({trackerMappings.length - LIMIT}) <ChevronDown class="h-4 w-4" />
                        {/if}
                    </button>
                {/if}
            </div>
        </div>
    {/if}
</div>