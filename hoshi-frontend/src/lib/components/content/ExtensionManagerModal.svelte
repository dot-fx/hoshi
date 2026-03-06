<script lang="ts">
    import { contentApi } from '$lib/api/content/content';
    import type { ExtensionSource } from '$lib/api/content/types';
    import * as Dialog from '$lib/components/ui/dialog';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { Loader2, Pencil, X, Search, Link as LinkIcon, Component } from 'lucide-svelte';
    import { toast } from "svelte-sonner";
    import { i18n } from "$lib/i18n/index.svelte"; // <-- Importar i18n

    let {
        open = $bindable(false),
        cid,
        extensions
    }: {
        open: boolean;
        cid: string;
        extensions: ExtensionSource[];
    } = $props();

    let isLoading = $state(false);
    let editingExtName = $state<string | null>(null);
    let searchQuery = $state("");
    let isSearching = $state(false);
    let searchResults = $state<any[]>([]);

    function startEdit(ext: ExtensionSource) {
        editingExtName = ext.extensionName;
        searchQuery = (ext.metadata as any)?.title || "";
        searchResults = [];
    }

    function cancelEdit() {
        editingExtName = null;
        searchQuery = "";
        searchResults = [];
    }

    async function handleSearch(e?: Event) {
        if (e) e.preventDefault();
        if (!editingExtName || !searchQuery.trim()) return;

        isSearching = true;
        try {
            const res = await contentApi.searchExtension(editingExtName, { query: searchQuery });
            if (res.success) {
                searchResults = res.results as any[];
                if (searchResults.length === 0) {
                    toast.info(i18n.t('no_results_found_ext'));
                }
            } else {
                toast.error(i18n.t('failed_fetch_ext'));
            }
        } catch (error) {
            console.error("Search failed", error);
            toast.error(i18n.t('error_searching_ext'));
        } finally {
            isSearching = false;
        }
    }

    async function handleUpdate(result: any) {
        if (!editingExtName) return;
        isLoading = true;

        const newExtensionId = result.id || result.url || result.extensionId;

        if (!newExtensionId) {
            toast.error(i18n.t('missing_id_error'));
            isLoading = false;
            return;
        }

        try {
            await contentApi.updateExtensionMapping(cid, {
                extensionName: editingExtName,
                extensionId: newExtensionId.toString()
            });
            toast.success(i18n.t('update_ext_success').replace('{extension}', editingExtName));
            window.location.reload();
        } catch (error) {
            console.error("Update failed", error);
            toast.error(i18n.t('update_ext_failed').replace('{extension}', editingExtName));
            isLoading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[550px] bg-card border-border/40 max-h-[85vh] flex flex-col">
        <Dialog.Header>
            <Dialog.Title class="text-xl">{i18n.t('manage_extensions_title')}</Dialog.Title>
            <Dialog.Description class="text-base">
                {i18n.t('manage_extensions_desc')}
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex-1 overflow-y-auto space-y-5 py-4 pr-2">
            {#if extensions && extensions.length > 0}
                {#each extensions as ext}
                    {@const meta = ext.metadata || {}}
                    <div class="flex flex-col p-4 rounded-xl border border-border/50 bg-muted/10 transition-colors {editingExtName === ext.extensionName ? 'border-primary/50 bg-primary/5' : ''}">
                        <div class="flex items-start justify-between gap-4">
                            <div class="flex items-center gap-4 overflow-hidden">
                                {#if meta.image}
                                    <img src={meta.image} alt={meta.title} class="w-16 h-24 object-cover rounded-md shadow-sm bg-muted shrink-0" />
                                {:else}
                                    <div class="w-16 h-24 bg-muted rounded-md flex items-center justify-center shrink-0">
                                        <Component class="w-8 h-8 text-muted-foreground/50" />
                                    </div>
                                {/if}

                                <div class="flex flex-col overflow-hidden py-1">
                                    <span class="font-bold text-base line-clamp-2">{meta.title || i18n.t('unknown_title')}</span>
                                    <span class="text-sm text-primary font-semibold uppercase tracking-wider mt-1">{ext.extensionName}</span>
                                    <span class="text-xs text-muted-foreground font-mono truncate mt-1" title={ext.extensionId}>{ext.extensionId}</span>
                                </div>
                            </div>

                            <div class="flex flex-col items-end gap-2 shrink-0">
                                {#if meta.url}
                                    <a href={meta.url} target="_blank" rel="noopener noreferrer" class="text-muted-foreground hover:text-primary p-2">
                                        <LinkIcon class="h-5 w-5" />
                                    </a>
                                {/if}
                                <Button variant="ghost" size="icon" class="h-10 w-10 text-muted-foreground hover:text-primary" onclick={() => startEdit(ext)} disabled={isLoading || editingExtName === ext.extensionName}>
                                    <Pencil class="h-5 w-5" />
                                </Button>
                            </div>
                        </div>

                        {#if editingExtName === ext.extensionName}
                            <div class="mt-5 pt-5 border-t border-border/40 flex flex-col gap-4">
                                <div class="flex items-center justify-between">
                                    <span class="text-sm font-semibold text-primary">{i18n.t('search_alternative_source')}</span>
                                    <Button variant="ghost" size="sm" class="h-8 text-xs px-3 text-muted-foreground" onclick={cancelEdit} disabled={isLoading}>
                                        <X class="h-4 w-4 mr-1.5" /> {i18n.t('cancel')}
                                    </Button>
                                </div>

                                <form onsubmit={handleSearch} class="flex items-center gap-2">
                                    <Input class="h-10 text-sm bg-background" placeholder={i18n.t('search_title_placeholder')} bind:value={searchQuery} disabled={isLoading || isSearching} />
                                    <Button type="submit" class="h-10 px-4" disabled={!searchQuery || isLoading || isSearching}>
                                        {#if isSearching}
                                            <Loader2 class="h-4 w-4 animate-spin" />
                                        {:else}
                                            <Search class="h-4 w-4" />
                                        {/if}
                                    </Button>
                                </form>

                                {#if searchResults.length > 0}
                                    <div class="flex flex-col gap-3 mt-2 max-h-[350px] overflow-y-auto pr-2">
                                        {#each searchResults as result}
                                            <div class="flex items-center justify-between bg-background p-3 rounded-lg border border-border/40 hover:border-primary/40 transition-colors">
                                                <div class="flex items-center gap-3 overflow-hidden">
                                                    {#if result.image}
                                                        <img src={result.image} alt={result.title} class="w-10 h-14 object-cover rounded-md shrink-0" />
                                                    {/if}
                                                    <div class="flex flex-col overflow-hidden">
                                                        <span class="text-sm font-semibold line-clamp-1">{result.title}</span>
                                                        <span class="text-xs text-muted-foreground font-mono truncate mt-0.5">{result.id || result.url}</span>
                                                    </div>
                                                </div>
                                                <Button size="sm" variant="secondary" class="h-8 px-4 text-xs shrink-0 ml-3" onclick={() => handleUpdate(result)} disabled={isLoading}>
                                                    {i18n.t('select')}
                                                </Button>
                                            </div>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    </div>
                {/each}
            {:else}
                <p class="text-base text-muted-foreground text-center py-10">{i18n.t('no_extensions_configured')}</p>
            {/if}
        </div>
    </Dialog.Content>
</Dialog.Root>