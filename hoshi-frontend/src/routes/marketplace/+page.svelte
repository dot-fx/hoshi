<script lang="ts">
    import { extensionsApi } from "@/api/extensions/extensions";
    import type { Extension } from "@/api/extensions/types";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from "$lib/i18n/index.svelte";

    import * as Tabs from "$lib/components/ui/tabs";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Empty from "$lib/components/ui/empty";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Badge } from "$lib/components/ui/badge";

    import {
        Puzzle,
        Search,
        RefreshCw,
        Download,
        Trash2,
        Link as LinkIcon,
        Server,
        Globe, Loader2
    } from "lucide-svelte";

    // --- ESTADOS ---
    let activeTab = $state<string>("installed");

    // Extensiones instaladas
    let installedExtensions = $state<Extension[]>([]);
    let isLoadingInstalled = $state(true);
    let uninstallingIds = $state<Set<string>>(new Set());
    let installedSearchQuery = $state("");

    // Marketplace
    let repoUrl = $state("");
    let marketplaceItems = $state<(Extension & { manifestUrl?: string })[]>([]);
    let isLoadingRepo = $state(false);
    let installingIds = $state<Set<string>>(new Set());
    let marketSearchQuery = $state("");

    // --- EFECTOS ---
    $effect(() => {
        loadInstalled();
    });

    // --- DERIVADOS ---
    let filteredInstalled = $derived(
        installedExtensions.filter(ext =>
            ext.name.toLowerCase().includes(installedSearchQuery.toLowerCase())
        )
    );

    let filteredMarketplace = $derived(
        marketplaceItems.filter(item =>
            item.name.toLowerCase().includes(marketSearchQuery.toLowerCase())
        )
    );

    // --- FUNCIONES ---
    async function loadInstalled() {
        isLoadingInstalled = true;
        try {
            installedExtensions = await extensionsApi.getAll() || [];
        } catch (error: any) {
            toast.error(i18n.t('failed_load_extensions'));
        } finally {
            isLoadingInstalled = false;
        }
    }

    async function handleUninstall(id: string) {
        uninstallingIds = new Set(uninstallingIds).add(id);
        try {
            const res = await extensionsApi.uninstall(id);
            if (res.ok) {
                toast.success(i18n.t('extension_uninstalled'));
                await loadInstalled();
            } else {
                toast.error(i18n.t('failed_uninstall_extension'));
            }
        } catch (error: any) {
            toast.error(error?.message || i18n.t('error_uninstalling'));
        } finally {
            const newSet = new Set(uninstallingIds);
            newSet.delete(id);
            uninstallingIds = newSet;
        }
    }

    async function loadRepository(e?: Event) {
        if (e) e.preventDefault();
        if (!repoUrl) {
            toast.error(i18n.t('enter_valid_repo_url'));
            return;
        }

        isLoadingRepo = true;
        try {
            const res = await fetch(repoUrl);
            if (!res.ok) throw new Error(i18n.t('failed_fetch_repo'));

            const data = await res.json();

            const items = Array.isArray(data) ? data : (data.extensions || []);
            marketplaceItems = items.map((item: any) => ({
                ...item,
                ext_type: item.ext_type || item.ext_type,
                manifestUrl: item.manifestUrl || `${repoUrl.replace(/\/[^\/]*$/, '')}/${item.id}.json`
            }));

            toast.success(`${i18n.t('loaded')} ${marketplaceItems.length} ${i18n.t('extensions')}`);
        } catch (error: any) {
            toast.error(error?.message || i18n.t('invalid_repo_url'));
            marketplaceItems = [];
        } finally {
            isLoadingRepo = false;
        }
    }

    async function handleInstall(item: Extension & { manifestUrl?: string }) {
        const manifest = item.manifestUrl;
        if (!manifest) {
            toast.error(i18n.t('missing_manifest_url'));
            return;
        }

        installingIds = new Set(installingIds).add(item.id);
        try {
            const res = await extensionsApi.install(manifest);
            if (res.ok) {
                toast.success(`${item.name} ${i18n.t('installed_successfully')}`);
                await loadInstalled();
            } else {
                toast.error(`${i18n.t('failed_install_extension')} ${item.name}`);
            }
        } catch (error: any) {
            toast.error(error?.message || `${i18n.t('error_installing')} ${item.name}`);
        } finally {
            const newSet = new Set(installingIds);
            newSet.delete(item.id);
            installingIds = newSet;
        }
    }

    function isInstalled(id: string) {
        return installedExtensions.some(ext => ext.id === id);
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
    <title>{i18n.t('marketplace')}</title>
</svelte:head>

<main class="min-h-screen bg-background pb-28 md:pb-10 pt-6 md:pt-8 px-4 md:px-6 lg:px-8 xl:px-10 w-full max-w-[2400px] mx-auto space-y-6 md:space-y-8">

    <!-- Header -->
    <header class="flex flex-col md:flex-row md:items-center justify-between gap-5 md:gap-6 border-b border-border/40 pb-6">
        <div class="space-y-1">
            <h1 class="text-3xl md:text-4xl font-black tracking-tight flex items-center gap-3">
                <Puzzle class="h-8 w-8 md:h-10 md:w-10 text-primary" />
                {i18n.t('marketplace')}
            </h1>
            <p class="text-sm md:text-base text-muted-foreground font-medium opacity-80">
                {i18n.t('marketplace_desc')}
            </p>
        </div>

        {#if activeTab === "installed"}
            <div class="relative w-full sm:w-80 group">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                <Input
                        placeholder={i18n.t('search_installed')}
                        class="pl-10 bg-muted/20 border-none h-11 rounded-xl focus-visible:ring-1 focus-visible:ring-primary/50"
                        bind:value={installedSearchQuery}
                />
            </div>
        {:else if activeTab === "browse" && marketplaceItems.length > 0}
            <div class="relative w-full sm:w-80 group">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                <Input
                        placeholder={i18n.t('search_repository')}
                        class="pl-10 bg-muted/20 border-none h-11 rounded-xl focus-visible:ring-1 focus-visible:ring-primary/50"
                        bind:value={marketSearchQuery}
                />
            </div>
        {/if}
    </header>

    <section class="space-y-6">
        <Tabs.Root bind:value={activeTab} class="w-full">
            <div class="flex items-center justify-between w-full overflow-hidden mb-6">
                <Tabs.List class="bg-transparent h-auto p-0 flex justify-start overflow-x-auto flex-nowrap hide-scrollbar gap-1.5 sm:gap-2 w-full">
                    <Tabs.Trigger
                            value="installed"
                            class="relative px-4 sm:px-6 py-2.5 rounded-xl text-xs sm:text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-transparent data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/20 whitespace-nowrap shrink-0"
                    >
                        <Server class="h-3.5 w-3.5 sm:h-4 sm:w-4 mr-1.5 sm:mr-2 inline-block" />
                        {i18n.t('installed')}
                    </Tabs.Trigger>

                    <Tabs.Trigger
                            value="browse"
                            class="relative px-4 sm:px-6 py-2.5 rounded-xl text-xs sm:text-sm font-bold transition-all data-[state=active]:bg-primary data-[state=active]:text-primary-foreground border border-transparent data-[state=inactive]:bg-muted/10 data-[state=inactive]:hover:bg-muted/20 whitespace-nowrap shrink-0"
                    >
                        <Globe class="h-3.5 w-3.5 sm:h-4 sm:w-4 mr-1.5 sm:mr-2 inline-block" />
                        {i18n.t('browse')}
                    </Tabs.Trigger>
                </Tabs.List>

                {#if activeTab === "installed"}
                    <Button variant="ghost" size="icon" onclick={loadInstalled} disabled={isLoadingInstalled} class="h-10 w-10 rounded-xl bg-muted/20 hover:bg-muted/40 shrink-0">
                        <RefreshCw class="h-4 w-4 {isLoadingInstalled ? 'animate-spin' : ''}" />
                    </Button>
                {/if}
            </div>

            <!-- INSTALLED TAB -->
            <Tabs.Content value="installed" class="outline-none">
                {#if isLoadingInstalled}
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 md:gap-6">
                        {#each Array(8) as _}
                            <Skeleton class="h-32 w-full rounded-2xl" />
                        {/each}
                    </div>
                {:else if installedExtensions.length === 0}
                    <Empty.Root class="border border-dashed border-border/60 rounded-2xl py-16 bg-muted/5 min-h-[40vh] flex items-center justify-center">
                        <Empty.Header>
                            <Empty.Media variant="icon">
                                <Puzzle class="size-8" />
                            </Empty.Media>
                            <Empty.Title>{i18n.t('no_extensions_installed')}</Empty.Title>
                            <Empty.Description>
                                {i18n.t('go_to_browse_extensions')}
                            </Empty.Description>
                        </Empty.Header>
                    </Empty.Root>
                {:else if filteredInstalled.length === 0}
                    <div class="py-12 text-center text-muted-foreground">
                        {i18n.t('no_installed_match')} "{installedSearchQuery}"
                    </div>
                {:else}
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 md:gap-5">
                        {#each filteredInstalled as ext (ext.id)}
                            <div in:fade={{ duration: 200 }} class="flex flex-col p-5 rounded-2xl shadow-sm border border-border/60 bg-card hover:border-primary/40 transition-colors group">
                                <div class="flex items-start gap-4">
                                    <Avatar.Root class="h-12 w-12 rounded-xl border border-border/50 shrink-0 bg-muted/30">
                                        {#if ext.icon}
                                            <Avatar.Image src={ext.icon} alt={ext.name} class="object-cover" />
                                        {/if}
                                        <Avatar.Fallback class="bg-primary/10 text-primary font-black rounded-xl">
                                            {ext.name.slice(0, 2).toUpperCase()}
                                        </Avatar.Fallback>
                                    </Avatar.Root>

                                    <div class="space-y-0.5 flex-1 min-w-0">
                                        <h3 class="font-black text-base truncate">{ext.name}</h3>
                                        <div class="flex items-center gap-1.5 text-xs font-bold text-muted-foreground/80">
                                            <span>v{ext.version}</span>
                                            <span>•</span>
                                            <span class="truncate">{ext.author || i18n.t('unknown_author')}</span>
                                        </div>
                                    </div>

                                    <Badge variant="outline" class="text-[10px] uppercase font-black tracking-wider h-5 {getTypeColor(ext.ext_type || (ext).ext_type)}">
                                        {i18n.t(ext.ext_type || (ext).ext_type) || (ext.ext_type || (ext).ext_type)}
                                    </Badge>
                                </div>

                                <div class="mt-5 pt-4 border-t border-border/30 flex justify-end opacity-0 group-hover:opacity-100 transition-opacity focus-within:opacity-100 md:opacity-100">
                                    <Button
                                            variant="secondary"
                                            class="text-destructive hover:bg-destructive hover:text-destructive-foreground rounded-xl h-9 px-4 font-bold transition-all w-full sm:w-auto bg-muted/30"
                                            onclick={() => handleUninstall(ext.id)}
                                            disabled={uninstallingIds.has(ext.id)}
                                    >
                                        {#if uninstallingIds.has(ext.id)}
                                            <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                                        {:else}
                                            <Trash2 class="h-4 w-4 mr-2" />
                                        {/if}
                                        {i18n.t('uninstall')}
                                    </Button>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </Tabs.Content>

            <!-- BROWSE TAB -->
            <Tabs.Content value="browse" class="outline-none space-y-8">

                <div class="p-6 md:p-8 rounded-3xl border border-border/50 bg-muted/10 shadow-sm relative overflow-hidden">
                    <div class="relative z-10 max-w-2xl">
                        <h2 class="text-xl md:text-2xl font-black mb-2">{i18n.t('load_repository')}</h2>
                        <p class="text-sm md:text-base text-muted-foreground mb-6 font-medium">
                            {i18n.t('load_repository_desc')}
                        </p>

                        <form onsubmit={loadRepository} class="flex flex-col sm:flex-row gap-3">
                            <div class="relative flex-1">
                                <LinkIcon class="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/70" />
                                <Input
                                        bind:value={repoUrl}
                                        placeholder={i18n.t('repo_url_placeholder')}
                                        class="rounded-xl h-12 pl-11 w-full bg-background border-border/60 focus-visible:ring-primary/50 text-base"
                                        required
                                />
                            </div>
                            <Button type="submit" disabled={isLoadingRepo} class="rounded-xl h-12 px-8 font-black shadow-sm shrink-0">
                                {#if isLoadingRepo}
                                    <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                                {:else}
                                    <Search class="h-4 w-4 mr-2" />
                                {/if}
                                {i18n.t('load_repository')}
                            </Button>
                        </form>
                    </div>
                </div>

                {#if marketplaceItems.length > 0}
                    <div in:fade={{ duration: 300 }}>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 md:gap-5">
                            {#each filteredMarketplace as item (item.id)}
                                <div class="flex flex-col p-5 rounded-2xl shadow-sm border border-border/60 bg-card hover:border-primary/40 transition-colors">
                                    <div class="flex items-start gap-4">
                                        <Avatar.Root class="h-12 w-12 rounded-xl border border-border/50 shrink-0 bg-muted/30">
                                            {#if item.icon}
                                                <Avatar.Image src={item.icon} alt={item.name} class="object-cover" />
                                            {/if}
                                            <Avatar.Fallback class="bg-primary/10 text-primary font-black rounded-xl">
                                                {item.name.slice(0, 2).toUpperCase()}
                                            </Avatar.Fallback>
                                        </Avatar.Root>

                                        <div class="space-y-0.5 flex-1 min-w-0">
                                            <h3 class="font-black text-base truncate">{item.name}</h3>
                                            <div class="flex items-center gap-1.5 text-xs font-bold text-muted-foreground/80">
                                                <span>v{item.version}</span>
                                                {#if item.author}
                                                    <span>•</span>
                                                    <span class="truncate">{item.author}</span>
                                                {/if}
                                            </div>
                                        </div>

                                        <Badge variant="outline" class="text-[10px] uppercase font-black tracking-wider h-5 {getTypeColor(item.ext_type || (item as any).ext_type)}">
                                            {i18n.t(item.ext_type) || (item.ext_type || (item).ext_type)}
                                        </Badge>
                                    </div>

                                    <div class="mt-5 pt-4 border-t border-border/30 flex justify-end">
                                        {#if isInstalled(item.id)}
                                            <Button variant="secondary" class="rounded-xl h-9 px-6 font-bold w-full sm:w-auto bg-muted/40 text-muted-foreground" disabled>
                                                {i18n.t('installed')}
                                            </Button>
                                        {:else}
                                            <Button
                                                    class="rounded-xl h-9 px-6 font-bold shadow-sm w-full sm:w-auto"
                                                    onclick={() => handleInstall(item)}
                                                    disabled={installingIds.has(item.id)}
                                            >
                                                {#if installingIds.has(item.id)}
                                                    <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                                                {:else}
                                                    <Download class="h-4 w-4 mr-2" />
                                                {/if}
                                                {i18n.t('install')}
                                            </Button>
                                        {/if}
                                    </div>
                                </div>
                            {/each}

                            {#if filteredMarketplace.length === 0}
                                <div class="col-span-full py-12 text-center text-muted-foreground">
                                    {i18n.t('no_extensions_found_match')} "{marketSearchQuery}"
                                </div>
                            {/if}
                        </div>
                    </div>
                {/if}
            </Tabs.Content>

        </Tabs.Root>
    </section>
</main>