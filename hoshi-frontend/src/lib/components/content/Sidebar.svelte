<script lang="ts">
    import { Badge } from "$lib/components/ui/badge";
    import { Calendar, Building2, AlertTriangle, CalendarDays } from "lucide-svelte";
    import { i18n } from "$lib/i18n/index.svelte";
    import type { ExtensionSource } from "$lib/api/content/types";

    let {
        cid,
        metadata,
        extensions = []
    }: {
        cid: string;
        metadata: any;
        extensions: ExtensionSource[]
    } = $props();

    function formatDate(dateStr?: string | null) {
        if (!dateStr) return i18n.t('content.tba');
        return new Date(dateStr).toLocaleDateString(i18n.locale || 'en-US', { year: 'numeric', month: 'short', day: 'numeric' });
    }
</script>

<div class="space-y-8 lg:top-28 self-start h-fit pb-10">

    <div class="space-y-4">
        <h3 class="font-bold text-lg tracking-tight text-foreground flex items-center gap-2">
            {i18n.t('content.information')}
        </h3>

        <div class="bg-muted/10 rounded-2xl border border-border/40 overflow-hidden shadow-sm">
            <div class="flex items-center justify-between p-4 border-b border-border/40 hover:bg-muted/30 transition-colors">
                <span class="text-muted-foreground flex items-center gap-2.5 text-sm font-medium">
                    <Building2 class="h-4 w-4 text-primary/70"/>
                    {i18n.t('content.studio')}
                </span>
                <span class="font-bold text-sm text-right text-foreground truncate max-w-[140px]" title={metadata.studio}>
                    {metadata.studio || i18n.t('content.tba')}
                </span>
            </div>

            <div class="flex items-center justify-between p-4 border-b border-border/40 hover:bg-muted/30 transition-colors">
                <span class="text-muted-foreground flex items-center gap-2.5 text-sm font-medium">
                    <CalendarDays class="h-4 w-4 text-primary/70"/>
                    {i18n.t('content.aired')}
                </span>
                <span class="font-bold text-sm text-right text-foreground">
                    {formatDate(metadata.releaseDate)}
                </span>
            </div>

            <div class="flex items-center justify-between p-4 hover:bg-muted/30 transition-colors">
                <span class="text-muted-foreground flex items-center gap-2.5 text-sm font-medium">
                    <Calendar class="h-4 w-4 text-primary/70"/>
                    {i18n.t('content.ended')}
                </span>
                <span class="font-bold text-sm text-right text-foreground">
                    {formatDate(metadata.endDate)}
                </span>
            </div>
        </div>

        {#if metadata.nsfw}
            <div class="flex items-center justify-between p-4 bg-destructive/5 rounded-2xl border border-destructive/20 mt-3 shadow-sm">
                <span class="text-destructive flex items-center gap-2.5 text-sm font-bold">
                    <AlertTriangle class="h-4 w-4"/>
                    NSFW
                </span>
                <Badge variant="destructive" class="text-[10px] uppercase font-black tracking-wider px-2 shadow-sm">
                    18+
                </Badge>
            </div>
        {/if}
    </div>

</div>