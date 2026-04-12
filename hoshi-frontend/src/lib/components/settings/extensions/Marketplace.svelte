<script lang="ts">
    import { extensions } from "@/stores/extensions.svelte.js";
    import type { Extension } from "@/api/extensions/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";
    import { Input } from "$lib/components/ui/input";
    import { Search, Link as LinkIcon } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import type { ExtensionsConfig } from "@/api/config/types";
    import Card from "./Card.svelte";

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
                <Card
                        ext={item}
                        mode="marketplace"
                        isMarketplaceInstalled={isInstalled(item.id)}
                        isActionLoading={installingIds.has(item.id)}
                        onAction={handleInstall}
                />
            {/each}
        </div>
    {:else if !isLoadingRepo && repoUrlLocal && repoUrlLocal === lastLoadedUrl}
        <div class="py-12 text-center border border-dashed border-border/40 rounded-2xl bg-muted/5">
            <p class="text-muted-foreground font-medium text-sm">No extensions on repository.</p>
        </div>
    {/if}
</div>