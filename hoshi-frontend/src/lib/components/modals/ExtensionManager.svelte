<script lang="ts">
    import { contentApi } from '@/api/content/content';
    import type { ExtensionSource, ContentMetadata } from '@/api/content/types';
    import { extensions as extensionsStore } from "$lib/extensions.svelte";
    import * as Dialog from '@/components/ui/dialog';
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import { Pencil, X, Search, Component, Plus } from 'lucide-svelte';
    import { toast } from "svelte-sonner";
    import { i18n } from "@/i18n/index.svelte.js";
    import { Spinner } from "@/components/ui/spinner";
    import type { CoreError } from "@/api/client";

    let {
        open = $bindable(false),
        cid,
        metadata,
        isNsfw = false,
        extensions,
        contentType = "anime",
        onSuccess
    }: {
        open?: boolean;
        cid: string;
        metadata: ContentMetadata;
        isNsfw: boolean;
        extensions: ExtensionSource[];
        contentType: "anime" | "manga" | "novel";
        onSuccess?: () => void;
    } = $props();

    let isLoading = $state(false);
    let editingExtName = $state<string | null>(null);
    let searchQuery = $state("");
    let isSearching = $state(false);
    let searchResults = $state<any[]>([]);

    const installedExtensions = $derived(extensionsStore[contentType] || []);
    const availableExtensions = $derived(installedExtensions.map(ext => {
        const mapping = extensions.find(m => m.extensionName === ext.id);
        return {
            name: ext.id,
            icon: ext.icon,
            mapping: mapping || null,
            isLinked: !!mapping
        };
    }));

    function startEdit(extName: string) {
        editingExtName = extName;
        searchQuery = metadata?.title || "";
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

            searchResults = Array.isArray(res) ? res : (res.results || res.data || []);

        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            isSearching = false;
        }
    }

    async function handleUpdate(result: any) {
        if (!editingExtName) return;
        isLoading = true;

        const currentItem = availableExtensions.find(ext => ext.name === editingExtName);
        const newExtensionId = (result.id || result.url || result.extensionId).toString();

        try {
            if (currentItem?.isLinked) {
                await contentApi.updateExtensionMapping(cid, {
                    extensionName: editingExtName,
                    extensionId: newExtensionId
                });
            } else {
                await contentApi.addExtensionSource(cid, {
                    cid: cid,
                    extensionName: editingExtName,
                    extensionId: newExtensionId,
                    nsfw: result.nsfw ?? isNsfw,
                    createdAt: Date.now(),
                    updatedAt: Date.now()
                });
            }

            toast.success(i18n.t('content.extension_manager.update_ext_success').replace('{extension}', editingExtName));

            if (onSuccess) {
                onSuccess();
            } else {
                window.location.reload();
            }
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            isLoading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[550px] bg-card border-border/40 max-h-[85vh] flex flex-col">
        <Dialog.Header>
            <Dialog.Title class="text-xl">{i18n.t('content.extension_manager.manage_extensions_title')}</Dialog.Title>
            <Dialog.Description class="text-base">
                {i18n.t('content.extension_manager.manage_extensions_desc')}
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex-1 overflow-y-auto space-y-5 py-4 pr-2">
            {#each availableExtensions as item}
                <div class="flex flex-col p-4 rounded-xl border border-border/50 bg-muted/10 transition-colors {editingExtName === item.name ? 'border-primary/50 bg-primary/5' : ''}">
                    <div class="flex items-start justify-between gap-4">
                        <div class="flex items-center gap-4 overflow-hidden">
                            {#if metadata?.coverImage}
                                <img src={metadata.coverImage} alt={metadata.title} class="w-16 h-24 object-cover rounded-md shadow-sm bg-muted shrink-0" />
                            {:else}
                                <div class="w-16 h-24 bg-muted rounded-md flex items-center justify-center shrink-0">
                                    <Component class="w-8 h-8 text-muted-foreground/50" />
                                </div>
                            {/if}

                            <div class="flex flex-col overflow-hidden py-1">
                                <span class="font-bold text-base line-clamp-2">{metadata?.title || i18n.t('content.extension_manager.unknown_title')}</span>
                                <span class="text-sm text-primary font-semibold uppercase tracking-wider mt-1">{item.name}</span>
                                {#if item.isLinked}
                                    <span class="text-xs text-muted-foreground font-mono truncate mt-1">{item.mapping?.extensionId}</span>
                                {:else}
                                    <span class="text-xs text-destructive font-bold uppercase mt-1 italic opacity-70">{i18n.t('content.extension_manager.not_linked')}</span>
                                {/if}
                            </div>
                        </div>

                        <div class="flex flex-col items-end gap-2 shrink-0">
                            <Button
                                    variant="ghost"
                                    size="icon"
                                    class="h-10 w-10 {item.isLinked ? 'text-muted-foreground hover:text-primary' : 'text-primary hover:bg-primary/10'}"
                                    onclick={() => startEdit(item.name)}
                                    disabled={isLoading || editingExtName === item.name}
                            >
                                {#if item.isLinked}
                                    <Pencil class="h-5 w-5" />
                                {:else}
                                    <Plus class="h-5 w-5" />
                                {/if}
                            </Button>
                        </div>
                    </div>

                    {#if editingExtName === item.name}
                        <div class="mt-5 pt-5 border-t border-border/40 flex flex-col gap-4 animate-in slide-in-from-top-2">
                            <div class="flex items-center justify-between">
                                <span class="text-sm font-semibold text-primary">{i18n.t('content.extension_manager.search_alternative_source')}</span>
                                <Button variant="ghost" size="sm" class="h-8 text-xs px-3 text-muted-foreground" onclick={cancelEdit} disabled={isLoading}>
                                    <X class="h-4 w-4 mr-1.5" /> {i18n.t('content.extension_manager.cancel')}
                                </Button>
                            </div>

                            <form onsubmit={handleSearch} class="flex items-center gap-2">
                                <Input class="h-10 text-sm bg-background" placeholder={i18n.t('content.extension_manager.search_title_placeholder')} bind:value={searchQuery} disabled={isLoading || isSearching} />
                                <Button type="submit" class="h-10 px-4" disabled={!searchQuery || isLoading || isSearching}>
                                    {#if isSearching}
                                        <Spinner class="h-4 w-4 animate-spin" />
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
                                                {i18n.t('content.extension_manager.select')}
                                            </Button>
                                        </div>
                                    {/each}
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    </Dialog.Content>
</Dialog.Root>