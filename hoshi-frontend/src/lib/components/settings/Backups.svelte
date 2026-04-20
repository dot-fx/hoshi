<script lang="ts">
    import { backupsApi } from "@/api/backups/backups";
    import type { ListBackupMeta } from "@/api/backups/types";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Button } from "$lib/components/ui/button";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import {
        Archive, Download, RotateCcw, Trash2, Plus,
        Database, CalendarClock, CloudDownload
    } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";

    let backups = $state<ListBackupMeta[]>([]);
    let isLoading = $state(true);
    let isCreating = $state(false);
    let activeAction = $state<{ id: number, type: 'restore' | 'delete' | 'download' } | null>(null);

    $effect(() => {
        loadBackups();
    });

    async function loadBackups() {
        isLoading = true;
        try {
            const res = await backupsApi.getAll();
            backups = res.sort((a, b) => b.createdAt - a.createdAt);
        } catch (error) {
            console.error(error);
            toast.error(i18n.t('settings.general_section.backups_load_error', { defaultValue: 'Failed to load backups' }));
        } finally {
            isLoading = false;
        }
    }

    async function handleCreate() {
        isCreating = true;
        try {
            const newBackup = await backupsApi.createManual();
            backups = [newBackup, ...backups];
            toast.success(i18n.t('settings.general_section.changes_updated'));
        } catch (error) {
            console.error(error);
            toast.error(i18n.t('errors.network'));
        } finally {
            isCreating = false;
        }
    }

    async function handleRestore(id: number) {
        if (!confirm(i18n.t('settings.general_section.confirm_restore'))) return;
        activeAction = { id, type: 'restore' };
        try {
            await backupsApi.restore_b(id);
            toast.success(i18n.t('settings.general_section.backup_restored'));
        } catch (error) {
            console.error(error);
            toast.error(i18n.t('errors.network'));
        } finally {
            activeAction = null;
        }
    }

    async function handleDelete(id: number) {
        if (!confirm(i18n.t('settings.general_section.confirm_delete_backup'))) return;
        activeAction = { id, type: 'delete' };
        try {
            await backupsApi.remove_b(id);
            backups = backups.filter(b => b.id !== id);
            toast.success(i18n.t('settings.general_section.changes_updated'));
        } catch (error) {
            console.error(error);
            toast.error(i18n.t('errors.network'));
        } finally {
            activeAction = null;
        }
    }

    async function handleDownload(id: number) {
        activeAction = { id, type: 'download' };
        try {
            await backupsApi.download(id);
            toast.success(i18n.t('settings.general_section.backup_downloading'));
        } catch (error) {
            console.error(error);
            toast.error(i18n.t('errors.network'));
        } finally {
            activeAction = null;
        }
    }

    function formatDate(timestamp: number) {
        return new Date(timestamp).toLocaleString(undefined, {
            year: 'numeric', month: 'short', day: 'numeric',
            hour: '2-digit', minute: '2-digit'
        });
    }
</script>

<div class="space-y-6">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-muted/20 border border-border/50 p-4 rounded-2xl">
        <div class="space-y-1">
            <h4 class="text-sm font-bold flex items-center gap-2">
                <Database class="size-4 text-primary" />
                {i18n.t('settings.general_section.manual_backup')}
            </h4>
            <p class="text-xs text-muted-foreground">
                {i18n.t('settings.general_section.manual_backup_desc')}
            </p>
        </div>
        <Button onclick={handleCreate} disabled={isCreating} class="shrink-0 gap-2 font-bold rounded-xl">
            {#if isCreating}
                <Spinner class="size-4" />
            {:else}
                <Plus class="size-4" />
            {/if}
            {i18n.t('settings.general_section.create_backup')}
        </Button>
    </div>

    <div class="space-y-4">
        <h4 class="text-sm font-bold uppercase tracking-wider text-muted-foreground ml-1">
            {i18n.t('settings.general_section.backup_history')}
        </h4>

        {#if isLoading}
            <div class="flex items-center justify-center p-8 text-muted-foreground" in:fade>
                <Spinner class="size-6" />
            </div>
        {:else if backups.length === 0}
            <div class="flex flex-col items-center justify-center p-10 text-center bg-muted/10 border border-dashed border-border/50 rounded-2xl" in:fade>
                <Archive class="size-10 text-muted-foreground/50 mb-3" />
                <p class="text-sm font-bold">{i18n.t('settings.general_section.no_backups')}</p>
                <p class="text-xs text-muted-foreground mt-1 max-w-sm">
                    {i18n.t('settings.general_section.no_backups_desc')}
                </p>
            </div>
        {:else}
            <div class="grid gap-3" in:fade>
                {#each backups as backup (backup.id)}
                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-card border border-border/50 p-4 rounded-xl shadow-sm hover:border-border transition-colors">

                        <div class="flex items-center gap-4 min-w-0">
                            <div class="size-10 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0">
                                {#if backup.trigger === 'REMOTE_SYNC'}
                                    <CloudDownload class="size-5" />
                                {:else}
                                    <Archive class="size-5" />
                                {/if}
                            </div>
                            <div class="space-y-0.5 min-w-0">
                                <div class="flex items-center gap-2">
                                    <span class="text-sm font-bold truncate">
                                        {#if backup.trigger === 'MANUAL'}
                                            {i18n.t('settings.general_section.manual_backup')}
                                        {:else if backup.trigger === 'REMOTE_SYNC'}
                                            {i18n.t('settings.general_section.raw_backup') || 'Copia Raw'}
                                        {:else}
                                            {i18n.t('settings.general_section.auto_backup')}
                                        {/if}
                                    </span>
                                    {#if backup.trackerName}
                                        <span class="text-[10px] font-black uppercase tracking-wider bg-muted px-2 py-0.5 rounded-md text-muted-foreground">
                                            {backup.trackerName}
                                        </span>
                                    {/if}
                                </div>
                                <div class="flex items-center gap-3 text-xs text-muted-foreground font-medium">
                                    <span class="flex items-center gap-1">
                                        <CalendarClock class="size-3" /> {formatDate(backup.createdAt)}
                                    </span>
                                    <span class="flex items-center gap-1">
                                        <Database class="size-3" /> {backup.entryCount} {i18n.t('settings.general_section.entries_count')}
                                    </span>
                                </div>
                            </div>
                        </div>

                        <div class="flex items-center gap-2 sm:ml-auto">
                            <Button
                                    variant="outline"
                                    size="sm"
                                    class="h-9 w-9 p-0 rounded-lg"
                                    title={i18n.t('settings.general_section.download_backup')}
                                    disabled={activeAction?.id === backup.id}
                                    onclick={() => handleDownload(backup.id)}
                            >
                                {#if activeAction?.id === backup.id && activeAction?.type === 'download'}
                                    <Spinner class="size-4" />
                                {:else}
                                    <Download class="size-4" />
                                {/if}
                            </Button>

                            {#if backup.trigger !== 'REMOTE_SYNC'}
                                <Button
                                        variant="outline"
                                        size="sm"
                                        class="h-9 w-9 p-0 rounded-lg text-primary hover:text-primary hover:bg-primary/10 border-primary/20"
                                        title={i18n.t('settings.general_section.restore_backup')}
                                        disabled={activeAction?.id === backup.id}
                                        onclick={() => handleRestore(backup.id)}
                                >
                                    {#if activeAction?.id === backup.id && activeAction?.type === 'restore'}
                                        <Spinner class="size-4" />
                                    {:else}
                                        <RotateCcw class="size-4" />
                                    {/if}
                                </Button>
                            {/if}

                            <Button
                                    variant="outline"
                                    size="sm"
                                    class="h-9 w-9 p-0 rounded-lg text-destructive hover:text-destructive hover:bg-destructive/10 border-destructive/20"
                                    title={i18n.t('settings.general_section.delete_backup')}
                                    disabled={activeAction?.id === backup.id}
                                    onclick={() => handleDelete(backup.id)}
                            >
                                {#if activeAction?.id === backup.id && activeAction?.type === 'delete'}
                                    <Spinner class="size-4 text-destructive" />
                                {:else}
                                    <Trash2 class="size-4 text-destructive" />
                                {/if}
                            </Button>
                        </div>

                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>