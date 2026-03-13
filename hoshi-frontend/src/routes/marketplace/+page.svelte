<script lang="ts">
    import { extensions } from "$lib/extensions.svelte";
    import { auth } from "$lib/auth.svelte";
    import type { Extension } from "@/api/extensions/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";

    import * as Avatar from "$lib/components/ui/avatar";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import {
        Search,
        Download,
        Link as LinkIcon,
        Loader2
    } from "lucide-svelte";
    import { layoutState } from '$lib/layoutState.svelte';

    $effect(() => {
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    let installingIds = $state<Set<string>>(new Set());

    let marketSearchQuery = $state("");
    let repoUrl = $state("");
    let marketplaceItems = $state<(Extension & { manifestUrl?: string })[]>([]);
    let isLoadingRepo = $state(false);

    $effect(() => {
        extensions.load();
    });

    // --- DERIVADOS ---
    let filteredMarketplace = $derived(
        marketplaceItems.filter(item =>
            item.name.toLowerCase().includes(marketSearchQuery.toLowerCase())
        )
    );

    // --- FUNCIONES ---
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
            toast.error(error?.message);
        } finally {
            const newSet = new Set(installingIds);
            newSet.delete(item.id);
            installingIds = newSet;
        }
    }

    async function loadRepository(e?: Event) {
        if (e) e.preventDefault();
        if (!repoUrl) {
            toast.error(i18n.t('marketplace.enter_valid_repo'));
            return;
        }

        isLoadingRepo = true;
        try {
            const res = await fetch(repoUrl);
            if (!res.ok) throw new Error(i18n.t('errors.network'));

            const data = await res.json();
            const items = Array.isArray(data) ? data : (data.extensions || []);
            marketplaceItems = items.map((item: any) => ({
                ...item,
                manifestUrl: item.manifestUrl || `${repoUrl.replace(/\/[^\/]*$/, '')}/${item.id}.json`
            }));
            toast.success(i18n.t('marketplace.loaded_extensions', { count: marketplaceItems.length }));
        } catch (error: any) {
            toast.error(error?.message);
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
            case 'booru': return 'bg-orange-500/10 text-orange-500 border-orange-500/20';
            default: return 'bg-muted text-muted-foreground border-border';
        }
    }
</script>

<svelte:head>
    <title>{i18n.t('marketplace.title')}</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-12 pt-8 md:pt-12 px-4 md:px-8 lg:px-12 w-full max-w-[2000px] mx-auto space-y-10">

    <header class="flex flex-col md:flex-row md:items-center justify-between gap-6 border-b border-border/40 pb-8 w-full">
        <div class="flex items-center gap-5">
            <Avatar.Root class="h-12 w-12 md:h-16 md:w-16 border border-border/50 shadow-sm">
                {#if auth.user?.avatar}
                    <Avatar.Image src={auth.user.avatar} alt={auth.user.username} class="object-cover" />
                {/if}
                <Avatar.Fallback class="bg-primary/10 text-primary font-black uppercase">
                    {auth.user?.username?.charAt(0) || 'U'}
                </Avatar.Fallback>
            </Avatar.Root>

            <div class="space-y-0.5">
                <h1 class="text-2xl md:text-3xl font-black tracking-tight">{i18n.t('marketplace.title')}</h1>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-70 uppercase tracking-wider">
                    {i18n.t('marketplace.store_title', { name: auth.user?.username || i18n.t('marketplace.user') })}
                </p>
            </div>
        </div>

        <div class="relative w-full md:w-80 group">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <Input
                    placeholder={i18n.t('marketplace.search_repository')}
                    class="pl-11 bg-muted/10 border-none shadow-sm h-11 rounded-xl focus-visible:ring-2 focus-visible:ring-primary/40 transition-all text-sm font-medium"
                    bind:value={marketSearchQuery}
            />
        </div>
    </header>

    <section class="space-y-8">
        <div class="p-8 rounded-3xl border border-border/40 bg-muted/5 shadow-sm relative overflow-hidden">
            <div class="relative z-10 max-w-2xl">
                <h2 class="text-xl md:text-2xl font-black mb-2">{i18n.t('marketplace.load_repo')}</h2>
                <p class="text-sm md:text-base text-muted-foreground mb-6 font-medium">{i18n.t('marketplace.load_repo_desc')}</p>
                <form onsubmit={loadRepository} class="flex flex-col sm:flex-row gap-3">
                    <div class="relative flex-1">
                        <LinkIcon class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/70" />
                        <Input bind:value={repoUrl} placeholder={i18n.t('marketplace.repo_url_placeholder')} class="rounded-xl h-12 pl-11 w-full bg-background border-border/60 focus-visible:ring-primary/50 text-base" required />
                    </div>
                    <Button type="submit" disabled={isLoadingRepo} class="rounded-xl h-12 px-8 font-black shadow-sm shrink-0">
                        {#if isLoadingRepo}<Loader2 class="h-4 w-4 mr-2 animate-spin" />{:else}<Search class="h-4 w-4 mr-2" />{/if}
                        {i18n.t('marketplace.load_repo_button')}
                    </Button>
                </form>
            </div>
        </div>

        {#if marketplaceItems.length > 0}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5" in:fade>
                {#each filteredMarketplace as item (item.id)}
                    <div class="flex flex-col p-5 rounded-2xl shadow-sm border border-border/60 bg-card hover:border-primary/40 transition-colors">
                        <div class="flex items-start gap-4">
                            <Avatar.Root class="h-12 w-12 rounded-xl border border-border/50 shrink-0 bg-muted/30">
                                {#if item.icon}<Avatar.Image src={item.icon} alt={item.name} class="object-cover" />{/if}
                                <Avatar.Fallback class="bg-primary/10 text-primary font-black rounded-xl">{item.name.slice(0, 2).toUpperCase()}</Avatar.Fallback>
                            </Avatar.Root>
                            <div class="space-y-0.5 flex-1 min-w-0">
                                <h3 class="font-black text-base truncate">{item.name}</h3>
                                <div class="flex items-center gap-1.5 text-xs font-bold text-muted-foreground/80">
                                    <span>v{item.version}</span>
                                    {#if item.author}<span>•</span><span class="truncate">{item.author}</span>{/if}
                                </div>
                            </div>
                            <Badge variant="outline" class="text-[10px] uppercase font-black tracking-wider h-5 {getTypeColor(item.ext_type)}">{item.ext_type}</Badge>
                        </div>
                        <div class="mt-5 pt-4 border-t border-border/30 flex justify-end">
                            {#if isInstalled(item.id)}
                                <Button variant="secondary" class="rounded-xl h-9 px-6 font-bold w-full sm:w-auto bg-muted/40 text-muted-foreground" disabled>{i18n.t('marketplace.installed')}</Button>
                            {:else}
                                <Button class="rounded-xl h-9 px-6 font-bold shadow-sm w-full sm:w-auto" onclick={() => handleInstall(item)} disabled={installingIds.has(item.id)}>
                                    {#if installingIds.has(item.id)}<Loader2 class="h-4 w-4 mr-2 animate-spin" />{:else}<Download class="h-4 w-4 mr-2" />{/if}
                                    {i18n.t('marketplace.install')}
                                </Button>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</main>