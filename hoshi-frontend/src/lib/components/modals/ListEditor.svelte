<script lang="ts">
    import * as Dialog from "@/components/ui/dialog";
    import { Input } from "@/components/ui/input";
    import { Label } from "@/components/ui/label";
    import { Button } from "@/components/ui/button";
    import * as Popover from "@/components/ui/popover";
    import { Calendar } from "@/components/ui/calendar";
    import { Textarea } from "@/components/ui/textarea";
    import { Checkbox } from "@/components/ui/checkbox";
    import { listApi } from "@/api/list/list";
    import type { ListStatus, UpsertEntryBody } from "@/api/list/types";
    import { toast } from "svelte-sonner";
    import { Trash2, Save, Star, CheckCircle, Calendar as CalendarIcon } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import {
        CalendarDate,
        DateFormatter,
        getLocalTimeZone,
        parseDate
    } from "@internationalized/date";
    import { cn } from "@/utils";
    import { i18n } from "@/i18n/index.svelte.js";
    import type { CoreError } from "@/api/client";

    let {
        open = $bindable(false),
        cid,
        title = i18n.t('list.modal.default_title'),
        contentType = "anime",
        coverImage = ""
    }: {
        open: boolean;
        cid: string;
        title?: string;
        contentType?: string;
        coverImage?: string;
    } = $props();

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
    let progressLabel = $derived(isAnime ? i18n.t('list.modal.episodes') : i18n.t('list.modal.chapters'));

    let statusOptions = $derived([
        { value: "CURRENT", label: isAnime ? i18n.t('list.modal.watching') : i18n.t('list.modal.reading') },
        { value: "COMPLETED", label: i18n.t('list.completed') },
        { value: "PLANNING", label: i18n.t('list.planning') },
        { value: "PAUSED", label: i18n.t('list.paused') },
        { value: "DROPPED", label: i18n.t('list.dropped') },
        { value: "REPEATING", label: i18n.t('list.repeating') }
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
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
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
            toast.success(isNew ? i18n.t('list.modal.added_to_list') : i18n.t('list.modal.entry_updated'));
            open = false;
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            submitting = false;
        }
    }

    async function handleDelete() {
        if (!confirm(i18n.t('list.modal.confirm_delete'))) return;
        submitting = true;
        try {
            await listApi.delete(cid);
            toast.success(i18n.t('list.modal.removed'));
            open = false;
        } catch (err) {
            const error = err as CoreError;
            toast.error(i18n.t(error.key));
        } finally {
            submitting = false;
        }
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-xl bg-background border-border p-0 overflow-hidden sm:rounded-2xl shadow-lg z-[100]">

        {#if loading}
            <div class="h-64 flex flex-col items-center justify-center gap-4 text-muted-foreground">
                <Spinner class="h-8 w-8 animate-spin text-primary" />
                <p class="font-bold">{i18n.t('list.modal.loading')}</p>
            </div>
        {:else}
            <div class="relative h-32 md:h-40 w-full overflow-hidden bg-muted flex items-end">
                {#if coverImage}
                    <img src={coverImage} alt={title} class="absolute inset-0 w-full h-full object-cover opacity-40 blur-sm" />
                    <div class="absolute inset-0 bg-linear-to-t from-background via-background/80 to-transparent"></div>
                {/if}
                <div class="relative z-10 p-6 flex items-center gap-5 w-full">
                    {#if coverImage}
                        <img src={coverImage} alt={title} class="w-16 h-24 md:w-20 md:h-28 object-cover rounded-lg shadow-lg border border-border/50 hidden sm:block" />
                    {/if}
                    <div>
                        <h2 class="text-xl md:text-2xl font-black text-foreground line-clamp-2 leading-tight drop-shadow-md tracking-tight">{title}</h2>
                        <p class="text-sm text-muted-foreground font-bold mt-1.5 uppercase tracking-wider">{isNew ? i18n.t('list.add_to_list') : i18n.t('list.modal.edit')}</p>
                    </div>
                </div>
            </div>

            <form onsubmit={handleSubmit} class="p-6 pt-4 space-y-6 overflow-y-auto max-h-[60vh] hide-scrollbar">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                    <div class="space-y-2">
                        <Label for="status" class="font-bold text-foreground/90">{i18n.t('list.modal.status')}</Label>
                        <select id="status" bind:value={status} class="flex h-11 w-full rounded-xl border border-border/50 bg-muted/10 px-3 py-2 text-sm font-semibold focus:outline-none focus:ring-1 focus:ring-primary/50">
                            {#each statusOptions as opt}
                                <option value={opt.value}>{opt.label}</option>
                            {/each}
                        </select>
                    </div>
                    <div class="space-y-2">
                        <Label for="score" class="font-bold text-foreground/90">{i18n.t('list.modal.score')}</Label>
                        <div class="relative flex items-center">
                            <Star class="absolute left-3.5 h-4 w-4 text-muted-foreground" />
                            <Input id="score" type="number" step="0.1" min="0" max="10" bind:value={score} class="pl-10 h-11 rounded-xl bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-semibold" />
                        </div>
                    </div>

                    <div class="space-y-2">
                        <Label for="progress" class="font-bold text-foreground/90">{progressLabel} {#if totalUnits}<span class="text-muted-foreground font-medium text-xs ml-1">(({i18n.t('list.modal.of_total', { count: totalUnits })}))</span>{/if}</Label>
                        <div class="relative flex items-center">
                            <CheckCircle class="absolute left-3.5 h-4 w-4 text-muted-foreground" />
                            <Input id="progress" type="number" min="0" bind:value={progress} class="pl-10 h-11 rounded-xl bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-semibold" />
                        </div>
                    </div>
                    <div class="space-y-2">
                        <Label for="repeat" class="font-bold text-foreground/90">{isAnime ? i18n.t('list.modal.times_rewatched') : i18n.t('list.modal.times_reread')}</Label>
                        <Input id="repeat" type="number" min="0" bind:value={repeatCount} class="h-11 rounded-xl bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-semibold" />
                    </div>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                    <div class="flex flex-col gap-2">
                        <Label class="font-bold text-foreground/90 px-1">{i18n.t('list.modal.start_date')}</Label>
                        <Popover.Root>
                            <Popover.Trigger>
                                {#snippet child({ props })}
                                    <Button
                                            variant="outline"
                                            class={cn("w-full justify-start text-left font-semibold h-11 rounded-xl bg-muted/10 border-border/50 hover:bg-muted/20", !startValue && "text-muted-foreground font-medium")}
                                            {...props}
                                    >
                                        <CalendarIcon class="mr-2 h-4 w-4" />
                                        {startValue ? df.format(startValue.toDate(getLocalTimeZone())) : i18n.t('list.modal.select_date')}
                                    </Button>
                                {/snippet}
                            </Popover.Trigger>
                            <Popover.Content class="w-auto p-0 rounded-xl z-[110]" align="start">
                                <Calendar type="single" bind:value={startValue} initialFocus captionLayout="dropdown" />
                            </Popover.Content>
                        </Popover.Root>
                    </div>

                    <div class="flex flex-col gap-2">
                        <Label class="font-bold text-foreground/90 px-1">{i18n.t('list.modal.end_date')}</Label>
                        <Popover.Root>
                            <Popover.Trigger>
                                {#snippet child({ props })}
                                    <Button
                                            variant="outline"
                                            class={cn("w-full justify-start text-left font-semibold h-11 rounded-xl bg-muted/10 border-border/50 hover:bg-muted/20", !endValue && "text-muted-foreground font-medium")}
                                            {...props}
                                    >
                                        <CalendarIcon class="mr-2 h-4 w-4" />
                                        {endValue ? df.format(endValue.toDate(getLocalTimeZone())) : i18n.t('list.modal.select_date')}
                                    </Button>
                                {/snippet}
                            </Popover.Trigger>
                            <Popover.Content class="w-auto p-0 rounded-xl z-[110]" align="start">
                                <Calendar type="single" bind:value={endValue} initialFocus captionLayout="dropdown" />
                            </Popover.Content>
                        </Popover.Root>
                    </div>
                </div>

                <div class="space-y-4">
                    <div class="space-y-2">
                        <Label for="notes" class="font-bold text-foreground/90">{i18n.t('list.modal.notes')}</Label>
                        <Textarea id="notes" bind:value={notes} class="min-h-[100px] rounded-xl bg-muted/10 border-border/50 focus-visible:ring-1 focus-visible:ring-primary/50 font-medium resize-none" />
                    </div>

                    <div class="flex items-center space-x-3 bg-muted/10 p-3 rounded-xl border border-border/50 w-fit">
                        <Checkbox id="isPrivate" bind:checked={isPrivate} />
                        <Label for="isPrivate" class="font-bold cursor-pointer text-sm">{i18n.t('list.modal.private')}</Label>
                    </div>
                </div>
            </form>

            <Dialog.Footer class="p-5 border-t border-border bg-muted/10 flex flex-col-reverse sm:flex-row sm:justify-between gap-3">
                <div class="flex w-full sm:w-auto justify-center sm:justify-start">
                    {#if !isNew}
                        <Button type="button" variant="destructive" size="icon" class="h-11 w-11 rounded-xl shadow-sm" onclick={handleDelete} disabled={submitting}>
                            <Trash2 class="h-5 w-5" />
                        </Button>
                    {/if}
                </div>

                <div class="flex flex-col sm:flex-row gap-3 w-full sm:w-auto">
                    <Button
                            type="button"
                            variant="outline"
                            class="w-full sm:w-32 h-11 rounded-xl font-bold border-border/50 hover:bg-muted/20"
                            disabled={submitting}
                            onclick={() => open = false}
                    >
                        {i18n.t('list.modal.cancel')}
                    </Button>
                    <Button
                            type="submit"
                            onclick={handleSubmit}
                            class="w-full sm:w-32 h-11 rounded-xl font-bold shadow-sm"
                            disabled={submitting}
                    >
                        {#if submitting}
                            <Spinner class="h-4 w-4 mr-2 animate-spin" />
                            {i18n.t('list.modal.saving')}
                        {:else}
                            <Save class="h-4 w-4 mr-2" />
                            {isNew ? i18n.t('list.modal.save') : i18n.t('list.modal.update')}
                        {/if}
                    </Button>
                </div>
            </Dialog.Footer>
        {/if}
    </Dialog.Content>
</Dialog.Root>

<style>
    :global([data-dialog-close]) {
        display: none !important;
    }
</style>