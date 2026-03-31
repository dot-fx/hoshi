<script lang="ts">
    import { extensions } from "$lib/extensions.svelte";
    import type { Extension } from "@/api/extensions/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";

    import * as Avatar from "$lib/components/ui/avatar";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Search, Download, Link as LinkIcon } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import type { ExtensionsConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let installingIds = $state<Set<string>>(new Set());
    let marketSearchQuery = $state("");

    let repoUrlLocal = $state(config.repoUrl || "");
    let lastLoadedUrl = $state("");
    let marketplaceItems = $state<(Extension & { manifestUrl?: string })[]>([]);
    let isLoadingRepo = $state(false);
    let debounceTimer: ReturnType<typeof setTimeout>;

    $effect(() => {
        if (repoUrlLocal && repoUrlLocal !== lastLoadedUrl) {
            clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {
                loadRepository();
            }, 800);
        }
    });

    let filteredMarketplace = $derived(
        marketplaceItems.filter(item =>
            item.name.toLowerCase().includes(marketSearchQuery.toLowerCase())
        )
    );

    async function handleInstall(item: Extension & { manifestUrl?: string }) {
        const manifest = item.manifestUrl;
        if (!manifest) {
            toast.error(i18n.t('marketplace.missing_manifest'));
            return;
        }

        installingIds = new Set(installingIds).add(item.id);
        try {
            await extensions.install(manifest);
            toast.success(i18n.t('marketplace.installed'));
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
        } finally {
            const newSet = new Set(installingIds);
            newSet.delete(item.id);
            installingIds = newSet;
        }
    }

    async function loadRepository() {
        if (!repoUrlLocal) return;

        isLoadingRepo = true;
        lastLoadedUrl = repoUrlLocal;

        // Auto-guardamos la URL en la config si cambió
        if (config.repoUrl !== repoUrlLocal) {
            config.repoUrl = repoUrlLocal;
            if (onSave) onSave();
        }

        try {
            const res = await fetch(repoUrlLocal);
            if (!res.ok) throw new Error(i18n.t('errors.network'));

            const data = await res.json();
            const items = Array.isArray(data) ? data : (data.extensions || []);
            marketplaceItems = items.map((item: any) => ({
                ...item,
                manifestUrl: item.manifestUrl || `${repoUrlLocal.replace(/\/[^\/]*$/, '')}/${item.id}.json`
            }));

        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
            marketplaceItems = [];
        } finally {
            isLoadingRepo = false;
        }
    }

    function isInstalled(id: string) {
        return extensions.installed.some(ext => ext.id === id);
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
    <div class="flex flex-col md:flex-row gap-4 items-center bg-muted/5 p-4 rounded-2xl border border-border/40">
        <div class="relative flex-1 w-full">
            <LinkIcon class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/70" />
            <Input bind:value={repoUrlLocal} placeholder={i18n.t('marketplace.repo_url_placeholder')} class="pl-9 bg-background h-10 rounded-xl w-full border-border/60" />

            {#if isLoadingRepo}
                <Spinner class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-muted-foreground" />
            {/if}
        </div>

        <div class="relative w-full md:w-64 shrink-0 group">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <Input
                    placeholder={i18n.t('marketplace.search_repository')}
                    class="pl-9 bg-background border-border/60 h-10 rounded-xl focus-visible:ring-1 focus-visible:ring-primary/40 text-sm"
                    bind:value={marketSearchQuery}
            />
        </div>
    </div>

    {#if marketplaceItems.length > 0}
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-3" in:fade>
            {#each filteredMarketplace as item (item.id)}
                <div class="flex items-center p-3 rounded-xl border border-border/60 bg-card hover:border-primary/40 transition-colors shadow-sm gap-3">
                    <Avatar.Root class="relative h-10 w-10 rounded-lg border border-border/50 shrink-0 bg-muted/30 overflow-hidden flex items-center justify-center">

                        <div data-fallback class="bg-primary/10 text-primary font-black rounded-lg text-xs w-full h-full flex items-center justify-center absolute inset-0 z-0">
                            {item.name.slice(0, 2).toUpperCase()}
                        </div>

                        {#if item.icon}
                            <img
                                    src={item.icon}
                                    alt={item.name}
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
                            <h3 class="font-bold text-sm truncate">{item.name}</h3>

                            <div class="flex items-center gap-1 shrink-0">
                                <Badge variant="outline" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 {getTypeColor(item.ext_type)}">
                                    {item.ext_type}
                                </Badge>

                                {#if item.language}
                                    <Badge variant="secondary" class="text-[9px] px-1 uppercase font-black tracking-wider h-4 bg-muted/80 text-muted-foreground">
                                        {item.language}
                                    </Badge>
                                {/if}
                            </div>
                        </div>
                        <div class="flex items-center gap-1.5 text-[11px] font-semibold text-muted-foreground/80 mt-0.5">
                            <span>v{item.version}</span>
                            {#if item.author}
                                <span class="opacity-50">•</span>
                                <span class="truncate">{item.author}</span>
                            {/if}
                        </div>
                    </div>

                    <div class="flex shrink-0 items-center">
                        {#if isInstalled(item.id)}
                            <Button variant="secondary" size="sm" class="rounded-lg h-8 px-4 text-xs font-bold bg-muted/40 text-muted-foreground" disabled>
                                {i18n.t('marketplace.installed')}
                            </Button>
                        {:else}
                            <Button size="sm" class="rounded-lg h-8 px-4 text-xs font-bold shadow-sm" onclick={() => handleInstall(item)} disabled={installingIds.has(item.id)}>
                                {#if installingIds.has(item.id)}
                                    <Spinner class="h-3 w-3 mr-1.5 animate-spin" />
                                {:else}
                                    <Download class="h-3 w-3 mr-1.5" />
                                {/if}
                                {i18n.t('marketplace.install')}
                            </Button>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    {:else if !isLoadingRepo && repoUrlLocal && repoUrlLocal === lastLoadedUrl}
        <div class="py-12 text-center border border-dashed border-border/40 rounded-2xl bg-muted/5">
            <p class="text-muted-foreground font-medium text-sm">No extensions on repository.</p>
        </div>
    {/if}
</div>