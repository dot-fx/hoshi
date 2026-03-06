<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import * as Popover from "$lib/components/ui/popover";
    import { Calendar } from "$lib/components/ui/calendar";
    import { Textarea } from "$lib/components/ui/textarea";
    import { Checkbox } from "$lib/components/ui/checkbox";

    import { listApi } from "@/api/list/list";
    import type { ListStatus, UpsertEntryBody } from "@/api/list/types";
    import { toast } from "svelte-sonner";
    import { Loader2, Trash2, Save, Star, CheckCircle, Calendar as CalendarIcon } from "lucide-svelte";
    import {
        CalendarDate,
        DateFormatter,
        getLocalTimeZone,
        parseDate
    } from "@internationalized/date";
    import { cn } from "$lib/utils";
    import { i18n } from "$lib/i18n/index.svelte"; // <-- Importamos i18n

    let {
        open = $bindable(false),
        cid,
        title = "Content",
        contentType = "anime",
        coverImage = ""
    }: {
        open: boolean;
        cid: string;
        title?: string;
        contentType?: string;
        coverImage?: string;
    } = $props();

    // Podemos hacer que el formatter escuche el idioma activo si es necesario
    const df = $derived(new DateFormatter(i18n.locale === 'es' ? 'es-ES' : 'en-US', { dateStyle: "long" }));

    let loading = $state(true);
    let submitting = $state(false);
    let isNew = $state(true);
    let totalUnits = $state<number | null>(null);

    let status = $state<ListStatus>("PLANNING");
    let progress = $state<number>(0);
    let score = $state<number | string>("");
    let repeatCount = $state<number>(0);
    let notes = $state<string>("");
    let isPrivate = $state<boolean>(false);

    let startValue = $state<CalendarDate | undefined>();
    let endValue = $state<CalendarDate | undefined>();

    let isAnime = $derived(contentType === "anime");
    let progressLabel = $derived(isAnime ? i18n.t('episodes') : i18n.t('chapters'));
    let statusOptions = $derived([
        { value: "CURRENT", label: isAnime ? i18n.t('watching_status') : i18n.t('reading_status') },
        { value: "COMPLETED", label: i18n.t('completed_status') },
        { value: "PLANNING", label: isAnime ? i18n.t('plan_to_watch') : i18n.t('plan_to_read') },
        { value: "PAUSED", label: i18n.t('paused_status') },
        { value: "DROPPED", label: i18n.t('dropped_status') },
        { value: "REPEATING", label: i18n.t('repeating_status') }
    ]);

    $effect(() => {
        if (open && cid) {
            loadEntry();
        } else if (!open) {
            resetForm();
        }
    });

    async function loadEntry() {
        loading = true;
        try {
            const res = await listApi.getEntry(cid);
            if (res.found && res.entry) {
                isNew = false;
                const e = res.entry;
                status = e.status;
                progress = e.progress;
                score = e.score ?? "";
                startValue = e.startDate ? parseDate(e.startDate.split('T')[0]) : undefined;
                endValue = e.endDate ? parseDate(e.endDate.split('T')[0]) : undefined;
                repeatCount = e.repeatCount;
                notes = e.notes || "";
                isPrivate = e.isPrivate;
                totalUnits = e.totalUnits ?? null;
            } else {
                isNew = true;
                resetForm();
            }
        } catch (err) {
            toast.error(i18n.t('failed_load_list'));
        } finally {
            loading = false;
        }
    }

    function resetForm() {
        status = "PLANNING";
        progress = 0;
        score = "";
        startValue = undefined;
        endValue = undefined;
        repeatCount = 0;
        notes = "";
        isPrivate = false;
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        submitting = true;

        try {
            const body: UpsertEntryBody = {
                cid,
                status,
                progress: progress || 0,
                score: typeof score === 'number' ? score : (parseFloat(score) || undefined),
                startDate: startValue?.toString(),
                endDate: endValue?.toString(),
                repeatCount: repeatCount || 0,
                notes: notes.trim() || undefined,
                isPrivate
            };

            await listApi.upsert(body);
            toast.success(isNew ? i18n.t('added_to_list') : i18n.t('list_entry_updated'));
            open = false;
        } catch (err: any) {
            toast.error(err?.message || i18n.t('failed_save_entry'));
        } finally {
            submitting = false;
        }
    }

    async function handleDelete() {
        if (!confirm(i18n.t('are_you_sure_remove'))) return;
        submitting = true;
        try {
            await listApi.delete(cid);
            toast.success(i18n.t('removed_from_list'));
            open = false;
        } catch (err) {
            toast.error(i18n.t('failed_remove_entry'));
        } finally {
            submitting = false;
        }
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-xl bg-background border-border p-0 overflow-hidden">
        {#if loading}
            <div class="h-64 flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Loader2 class="h-8 w-8 animate-spin text-primary" />
                <p>{i18n.t('loading_list')}</p>
            </div>
        {:else}
            <div class="relative h-32 md:h-40 w-full overflow-hidden bg-muted flex items-end">
                {#if coverImage}
                    <img src={coverImage} alt={title} class="absolute inset-0 w-full h-full object-cover opacity-40 blur-sm" />
                    <div class="absolute inset-0 bg-linear-to-t from-background via-background/80 to-transparent"></div>
                {/if}
                <div class="relative z-10 p-6 flex items-center gap-4 w-full">
                    {#if coverImage}
                        <img src={coverImage} alt={title} class="w-16 h-24 md:w-20 md:h-28 object-cover rounded shadow-lg border border-border/50 hidden sm:block" />
                    {/if}
                    <div>
                        <h2 class="text-xl md:text-2xl font-bold text-foreground line-clamp-2 leading-tight drop-shadow-md">{title}</h2>
                        <p class="text-sm text-muted-foreground font-medium mt-1">{isNew ? i18n.t('add_to_list') : i18n.t('edit_entry')}</p>
                    </div>
                </div>
            </div>

            <form onsubmit={handleSubmit} class="p-6 pt-2 space-y-6 overflow-y-auto max-h-[60vh]">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div class="space-y-2">
                        <Label for="status">{i18n.t('status')}</Label>
                        <select id="status" bind:value={status} class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring">
                            {#each statusOptions as opt}
                                <option value={opt.value}>{opt.label}</option>
                            {/each}
                        </select>
                    </div>
                    <div class="space-y-2">
                        <Label for="score">{i18n.t('score_label')}</Label>
                        <div class="relative">
                            <Star class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                            <Input id="score" type="number" step="0.1" min="0" max="10" bind:value={score} class="pl-9" />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <Label for="progress">{progressLabel} {#if totalUnits}<span class="text-muted-foreground text-xs ml-1">({i18n.t('of_total')} {totalUnits})</span>{/if}</Label>
                        <div class="relative">
                            <CheckCircle class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                            <Input id="progress" type="number" min="0" bind:value={progress} class="pl-9" />
                        </div>
                    </div>
                    <div class="space-y-2">
                        <Label for="repeat">{isAnime ? i18n.t('times_rewatched') : i18n.t('times_reread')}</Label>
                        <Input id="repeat" type="number" min="0" bind:value={repeatCount} />
                    </div>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div class="flex flex-col gap-2">
                        <Label class="px-1">{i18n.t('start_date')}</Label>
                        <Popover.Root>
                            <Popover.Trigger>
                                {#snippet child({ props })}
                                    <Button
                                            variant="outline"
                                            class={cn("w-full justify-start text-left font-normal", !startValue && "text-muted-foreground")}
                                            {...props}
                                    >
                                        <CalendarIcon class="mr-2 h-4 w-4" />
                                        {startValue ? df.format(startValue.toDate(getLocalTimeZone())) : i18n.t('select_date')}
                                    </Button>
                                {/snippet}
                            </Popover.Trigger>
                            <Popover.Content class="w-auto p-0" align="start">
                                <Calendar type="single" bind:value={startValue} initialFocus captionLayout="dropdown" />
                            </Popover.Content>
                        </Popover.Root>
                    </div>

                    <div class="flex flex-col gap-2">
                        <Label class="px-1">{i18n.t('finish_date')}</Label>
                        <Popover.Root>
                            <Popover.Trigger>
                                {#snippet child({ props })}
                                    <Button
                                            variant="outline"
                                            class={cn("w-full justify-start text-left font-normal", !endValue && "text-muted-foreground")}
                                            {...props}
                                    >
                                        <CalendarIcon class="mr-2 h-4 w-4" />
                                        {endValue ? df.format(endValue.toDate(getLocalTimeZone())) : i18n.t('select_date')}
                                    </Button>
                                {/snippet}
                            </Popover.Trigger>
                            <Popover.Content class="w-auto p-0" align="start">
                                <Calendar type="single" bind:value={endValue} initialFocus captionLayout="dropdown" />
                            </Popover.Content>
                        </Popover.Root>
                    </div>
                </div>

                <!-- NOTA: Tenías el campo 'notes' duplicado en tu código original.
                     He conservado el segundo que usa Textarea porque se ve más limpio -->
                <div class="space-y-4">
                    <div class="space-y-2">
                        <Label for="notes">{i18n.t('notes')}</Label>
                        <Textarea id="notes" bind:value={notes} class="min-h-[80px]" />
                    </div>

                    <div class="flex items-center space-x-2">
                        <Checkbox id="isPrivate" bind:checked={isPrivate} />
                        <Label for="isPrivate" class="font-normal cursor-pointer">{i18n.t('keep_private')}</Label>
                    </div>
                </div>
            </form>

            <Dialog.Footer class="p-6 pt-4 border-t border-border bg-muted/20 flex flex-col-reverse sm:flex-row sm:justify-between gap-4">
                <div class="flex w-full sm:w-auto justify-center sm:justify-start">
                    {#if !isNew}
                        <Button type="button" variant="destructive" size="icon" onclick={handleDelete} disabled={submitting}>
                            <Trash2 class="h-4 w-4" />
                        </Button>
                    {/if}
                </div>

                <div class="flex flex-col sm:flex-row gap-2 w-full sm:w-auto">
                    <Button
                            type="button"
                            variant="outline"
                            class="w-full sm:w-auto"
                            disabled={submitting}
                            onclick={() => open = false}
                    >
                        {i18n.t('cancel')}
                    </Button>
                    <Button
                            type="submit"
                            onclick={handleSubmit}
                            class="w-full sm:w-auto gap-2"
                            disabled={submitting}
                    >
                        {#if submitting}
                            <Loader2 class="h-4 w-4 animate-spin" />
                            {i18n.t('saving')}
                        {:else}
                            <Save class="h-4 w-4" />
                            {isNew ? i18n.t('save') : i18n.t('update')}
                        {/if}
                    </Button>
                </div>
            </Dialog.Footer>
        {/if}
    </Dialog.Content>
</Dialog.Root>