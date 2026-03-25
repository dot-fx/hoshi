<script lang="ts">
    import { Badge } from "$lib/components/ui/badge";
    import { Calendar, Building2, Hash, Pencil, Component, AlertTriangle } from "lucide-svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import type { ExtensionSource } from "$lib/api/content/types";

    import ExtensionManager from "@/components/modals/ExtensionManager.svelte";

    let {
        cid,
        metadata,
        extensions = []
    }: {
        cid: string;
        metadata: any;
        extensions: ExtensionSource[]
    } = $props();

    let showExtensionModal = $state(false);

    function formatDate(dateStr?: string | null) {
        if (!dateStr) return i18n.t('content.tba');
        return new Date(dateStr).toLocaleDateString(i18n.locale || 'en-US', { year: 'numeric', month: 'short', day: 'numeric' });
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

<div class="space-y-8 lg:sticky lg:top-24">

    <!-- INFO DATOS DUROS -->
    <!-- Rediseñado para móvil: sin bordes gruesos de Card, usando una lista limpia -->
    <div class="space-y-3">
        <h3 class="font-bold text-lg tracking-tight text-foreground hidden lg:block">{i18n.t('content.information') || 'Information'}</h3>
        <div class="flex flex-col text-sm bg-muted/10 rounded-2xl border border-border/40 px-4 py-1 shadow-sm">
            <div class="flex items-center justify-between py-3 border-b border-border/40 last:border-0">
                <span class="text-muted-foreground flex items-center gap-2"><Building2 class="h-4 w-4"/> {i18n.t('content.studio')}</span>
                <span class="font-semibold text-right">{metadata.studio || i18n.t('content.tba')}</span>
            </div>
            <div class="flex items-center justify-between py-3 border-b border-border/40 last:border-0">
                <span class="text-muted-foreground flex items-center gap-2"><Calendar class="h-4 w-4"/> {i18n.t('content.aired')}</span>
                <span class="font-semibold text-right">{formatDate(metadata.releaseDate)}</span>
            </div>
            <div class="flex items-center justify-between py-3 border-b border-border/40 last:border-0">
                <span class="text-muted-foreground flex items-center gap-2"><Calendar class="h-4 w-4"/> {i18n.t('content.ended')}</span>
                <span class="font-semibold text-right">{formatDate(metadata.endDate)}</span>
            </div>
            {#if metadata.nsfw}
                <div class="flex items-center justify-between py-3 border-b border-border/40 last:border-0">
                    <span class="text-muted-foreground flex items-center gap-2"><AlertTriangle class="h-4 w-4 text-destructive"/> NSFW</span>
                    <Badge variant="destructive" class="text-[10px] uppercase font-black tracking-wider px-2 py-0.5 shadow-sm">18+</Badge>
                </div>
            {/if}
        </div>
    </div>

    <!-- EXTERNAL IDS -->
    {#if metadata.externalIds && Object.keys(metadata.externalIds).length > 0}
        <div class="space-y-4 pt-6 border-t border-border/20">
            <h3 class="font-bold text-xs uppercase tracking-wider text-muted-foreground flex items-center gap-2">
                <Hash class="h-3.5 w-3.5" /> {i18n.t('content.external_ids')}
            </h3>
            <div class="grid grid-cols-2 gap-2.5">
                {#each Object.entries(metadata.externalIds) as [key, value]}
                    {#if value && !['slug', 'anilist', 'mal', 'simkl', 'trakt', 'kitsu', 'anidb'].includes(key.toLowerCase())}
                        {@const style = getPlatformStyle(key)}
                        <div class="flex flex-col p-2.5 rounded-xl border transition-colors {style.color} shadow-sm overflow-hidden bg-background">
                            <span class="text-[9px] font-black uppercase tracking-tight opacity-70 mb-0.5">
                                {style.label}
                            </span>
                            <span class="text-xs font-mono truncate select-all font-semibold">
                                {value}
                            </span>
                        </div>
                    {/if}
                {/each}
            </div>
        </div>
    {/if}
</div>