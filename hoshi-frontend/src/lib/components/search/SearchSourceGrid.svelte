<script lang="ts">
    import { Plug } from "lucide-svelte";
    import { searchState } from "@/stores/search.svelte.js";

    let {
        isMobile = false,
        availableExtensions = [],
        onSelectSource
    }: {
        isMobile?: boolean;
        availableExtensions: any[];
        onSelectSource: (mode: "tracker" | "extension", extId: string, tracker: "anilist" | "mal" | "kitsu", isMobile: boolean) => void;
    } = $props();

    function getTrackerFavicon(trackerName: string) {
        const domains: Record<string, string> = {
            'anilist': 'anilist.co',
            'mal': 'myanimelist.net',
            'kitsu': 'kitsu.io',
            'simkl': 'simkl.com'
        };
        const domain = domains[trackerName.toLowerCase()] || 'google.com';
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }
</script>

<div class="grid {isMobile ? 'grid-cols-4 sm:grid-cols-5 md:grid-cols-6' : 'grid-cols-4'} gap-3">
    <button onclick={() => onSelectSource('tracker', '', 'anilist', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
        <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchState.searchMode === 'tracker' && searchState.tracker === 'anilist' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
            <img src={getTrackerFavicon('anilist')} alt="AniList" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchState.searchMode === 'tracker' && searchState.tracker === 'anilist' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
        </div>
        <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">AniList</span>
    </button>

    <button onclick={() => onSelectSource('tracker', '', 'mal', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
        <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchState.searchMode === 'tracker' && searchState.tracker === 'mal' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
            <img src={getTrackerFavicon('mal')} alt="MyAnimeList" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchState.searchMode === 'tracker' && searchState.tracker === 'mal' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
        </div>
        <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">MAL</span>
    </button>

    <button onclick={() => onSelectSource('tracker', '', 'kitsu', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
        <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border transition-all duration-300 {searchState.searchMode === 'tracker' && searchState.tracker === 'kitsu' ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
            <img src={getTrackerFavicon('kitsu')} alt="Kitsu" class="w-6 h-6 rounded-sm object-contain transition-all duration-300 {searchState.searchMode === 'tracker' && searchState.tracker === 'kitsu' ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" />
        </div>
        <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">Kitsu</span>
    </button>

    {#each availableExtensions as ext}
        <button onclick={() => onSelectSource('extension', ext.id, 'anilist', isMobile)} class="flex flex-col items-center gap-2 group outline-none w-full">
            <div class="w-14 h-14 shrink-0 rounded-xl flex items-center justify-center bg-background shadow-sm border overflow-hidden transition-all duration-300 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? 'border-primary ring-2 ring-primary/20 scale-105' : 'border-border/50 group-hover:border-primary/50 group-hover:scale-105'}">
                {#if ext.icon}
                    <img src={ext.icon} class="w-8 h-8 rounded-md object-contain transition-all duration-300 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? '' : 'grayscale opacity-70 group-hover:grayscale-0 group-hover:opacity-100'}" alt={ext.name} />
                {:else}
                    <Plug class="w-6 h-6 {searchState.searchMode === 'extension' && searchState.selectedExtension === ext.id ? 'text-primary' : 'text-muted-foreground'}" />
                {/if}
            </div>
            <span class="text-[10px] sm:text-xs font-bold text-center text-foreground/90 w-full truncate">{ext.name}</span>
        </button>
    {/each}
</div>