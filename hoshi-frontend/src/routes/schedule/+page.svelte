<script lang="ts">
    import { scheduleApi } from "$lib/api/schedule/schedule";
    import { auth } from "$lib/auth.svelte";
    import type { AiringEntry } from "$lib/api/schedule/types";
    import { i18n } from "$lib/i18n/index.svelte";

    import * as Tabs from "$lib/components/ui/tabs";
    import * as Avatar from "$lib/components/ui/avatar";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Badge } from "$lib/components/ui/badge";

    import { CalendarDays, Clock, PlayCircle, Calendar as CalendarIcon, ChevronRight } from "lucide-svelte";
    import { fade } from "svelte/transition";
    import { layoutState } from '$lib/layoutState.svelte';

    $effect(() => {
        layoutState.title = "";
        layoutState.showBack = false;
        layoutState.backUrl = null;
    });

    let viewMode = $state<"week" | "month">("week");
    let isLoading = $state(true);
    let entries = $state<AiringEntry[]>([]);

    async function loadSchedule() {
        isLoading = true;
        try {
            const daysAhead = viewMode === "week" ? 7 : 30;
            const res = await scheduleApi.get({ daysBack: 0, daysAhead });

            entries = res?.data || [];
        } catch (error) {
            console.error(i18n.t('errors.network'));
            entries = [];
        } finally {
            isLoading = false;
        }
    }

    $effect(() => {
        viewMode;
        loadSchedule();
    });

    function getMs(timestamp: number) {
        return timestamp > 1e11 ? timestamp : timestamp * 1000;
    }

    function getDayLabel(date: Date) {
        const today = new Date();
        const tomorrow = new Date(today);
        tomorrow.setDate(tomorrow.getDate() + 1);

        if (date.toDateString() === today.toDateString()) return i18n.t('schedule.today');
        if (date.toDateString() === tomorrow.toDateString()) return i18n.t('schedule.tomorrow');
        return date.toLocaleDateString(i18n.locale, {
            weekday: 'long',
            month: 'long',
            day: 'numeric'
        });
    }

    let groupedEntries = $derived.by(() => {
        const groups: Record<string, AiringEntry[]> = {};

        entries.forEach(e => {
            const d = new Date(getMs(e.airingAt));
            const key = `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;

            if (!groups[key]) groups[key] = [];
            groups[key].push(e);
        });

        return Object.keys(groups).sort().map(key => {
            const d = new Date(getMs(groups[key][0].airingAt));

            return {
                key,
                date: d,
                header: getDayLabel(d),
                isToday: d.toDateString() === new Date().toDateString(),
                items: groups[key].sort((a, b) => a.airingAt - b.airingAt)
            };
        });
    });

    function formatTime(timestamp: number) {
        return new Date(getMs(timestamp)).toLocaleTimeString(i18n.locale, {
            hour: '2-digit',
            minute: '2-digit',
            hour12: false
        });
    }

    function formatUserStatus(status?: string | null) {
        if (!status) return null;
        if (status === 'CURRENT') return i18n.t('schedule.watching');
        if (status === 'PLANNING') return i18n.t('schedule.planning');

        return status.charAt(0).toUpperCase() + status.slice(1).toLowerCase();
    }
</script>

<svelte:head>
    <title>{i18n.t('schedule.title')}</title>
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
                <h1 class="text-2xl md:text-3xl font-black tracking-tight">{i18n.t('schedule.upcoming_episodes')}</h1>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-70 uppercase tracking-wider">
                    {i18n.t('schedule.release_calendar', { name: auth.user?.username || i18n.t('schedule.my') })}
                </p>
            </div>
        </div>

        <div class="overflow-hidden bg-muted/10 p-1 rounded-xl border border-border/40 backdrop-blur-sm shrink-0">
            <Tabs.Root bind:value={viewMode}>
                <Tabs.List class="flex bg-transparent h-9 p-0 gap-1">
                    <Tabs.Trigger value="week" class="rounded-lg px-4 text-xs font-bold data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                        {i18n.t('schedule.next_7')}
                    </Tabs.Trigger>
                    <Tabs.Trigger value="month" class="rounded-lg px-4 text-xs font-bold data-[state=active]:bg-primary data-[state=active]:text-primary-foreground">
                        {i18n.t('schedule.full_month')}
                    </Tabs.Trigger>
                </Tabs.List>
            </Tabs.Root>
        </div>
    </header>

    <section class="relative w-full">
        {#if isLoading}
            <div class="space-y-12">
                {#each Array(3) as _}
                    <div class="space-y-6">
                        <Skeleton class="h-8 w-48 rounded-lg" />
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-4">
                            {#each Array(4) as __}
                                <Skeleton class="h-28 sm:h-32 md:h-36 w-full rounded-xl" />
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {:else if groupedEntries.length === 0}
            <div class="flex flex-col items-center justify-center py-32 text-center space-y-5 border-2 border-dashed rounded-xl border-muted/20 bg-muted/5" in:fade>
                <div class="bg-background p-6 rounded-full shadow-sm border border-border/50">
                    <CalendarIcon class="h-12 w-12 text-muted-foreground/30" />
                </div>
                <div class="space-y-2 px-6">
                    <h3 class="text-2xl font-bold">{i18n.t('schedule.empty_title')}</h3>
                    <p class="text-sm text-muted-foreground max-w-[300px] mx-auto">{i18n.t('schedule.empty_desc')}</p>
                </div>
            </div>
        {:else}
            <div class="space-y-12 md:space-y-16 relative">
                <div class="hidden lg:block absolute left-[19px] top-4 bottom-0 w-[2px] bg-border/40 z-0 rounded-full"></div>

                {#each groupedEntries as group (group.key)}
                    <div class="relative z-10" in:fade={{ duration: 400 }}>
                        <div class="flex items-center gap-4 mb-6 sticky top-16 md:top-20 bg-background/95 backdrop-blur-md py-4 z-20 lg:-ml-[5px]">
                            <div class="hidden lg:flex h-12 w-12 rounded-full border-4 border-background bg-muted items-center justify-center shrink-0 shadow-sm z-10
                                {group.isToday ? 'bg-primary border-primary/20 text-primary-foreground' : 'text-muted-foreground'}">
                                <CalendarIcon class="h-5 w-5" />
                            </div>
                            <h2 class="text-2xl font-black tracking-tight {group.isToday ? 'text-primary' : 'text-foreground'}">
                                {group.header}
                            </h2>
                            {#if group.isToday}
                                <Badge variant="default" class="uppercase tracking-widest text-[10px] font-black">
                                    {i18n.t('schedule.airing_today')}
                                </Badge>
                            {/if}
                            <div class="h-[1px] flex-1 bg-border/40 ml-4 hidden sm:block"></div>
                        </div>

                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 3xl:grid-cols-5 gap-4 pl-0 lg:pl-16">
                            {#each group.items as entry (entry.id)}
                                <a
                                        href={`/content/${entry.cid}`}
                                        class="group flex h-28 sm:h-32 md:h-36 bg-card/40 hover:bg-card rounded-xl border border-border/50 hover:border-primary/50 transition-all duration-300 overflow-hidden shadow-sm hover:shadow-md"
                                >
                                    <div class="relative h-full w-20 sm:w-24 md:w-28 shrink-0 bg-muted overflow-hidden">
                                        {#if entry.coverImage}
                                            <img src={entry.coverImage} alt={entry.title} class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105" />
                                        {:else}
                                            <div class="h-full w-full flex items-center justify-center bg-muted/80">
                                                <PlayCircle class="h-8 w-8 text-muted-foreground/30" />
                                            </div>
                                        {/if}

                                        {#if entry.userStatus}
                                            <div class="absolute bottom-1 left-1 bg-background/90 text-foreground text-[9px] font-bold uppercase tracking-widest px-1.5 py-0.5 rounded shadow-sm backdrop-blur-md">
                                                {formatUserStatus(entry.userStatus)}
                                            </div>
                                        {/if}
                                    </div>

                                    <div class="flex flex-col flex-1 p-3 md:p-4 min-w-0 justify-between">
                                        <div class="flex items-start justify-between gap-2 mb-1">
                                            <div class="flex items-center gap-1.5 text-primary font-bold text-sm md:text-base tracking-tight bg-primary/10 px-2 py-0.5 rounded-md w-fit">
                                                <Clock class="h-3.5 w-3.5" />
                                                {formatTime(entry.airingAt)}
                                            </div>
                                            {#if entry.subtype}
                                                <span class="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/60 border border-border/50 rounded px-1.5 py-0.5 hidden sm:block">
                                                    {entry.subtype}
                                                </span>
                                            {/if}
                                        </div>

                                        <div class="mb-auto mt-1">
                                            <h3 class="font-bold text-sm leading-tight line-clamp-2 group-hover:text-primary transition-colors text-foreground/90" title={entry.title}>
                                                {entry.title}
                                            </h3>
                                        </div>

                                        <div class="flex items-center justify-between mt-2 pt-2 border-t border-border/40">
                                            <span class="text-xs font-black bg-foreground/5 text-foreground px-2 py-0.5 rounded-full">
                                                {i18n.t('schedule.episode_number', { num: entry.episode })}
                                            </span>
                                            <ChevronRight class="h-4 w-4 text-muted-foreground/50 group-hover:text-primary group-hover:translate-x-1 transition-all" />
                                        </div>
                                    </div>
                                </a>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </section>
</main>