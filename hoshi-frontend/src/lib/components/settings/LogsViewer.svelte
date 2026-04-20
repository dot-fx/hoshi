<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "svelte-sonner";
    import { Button } from "$lib/components/ui/button";
    import { RefreshCw, Copy, Trash2, Search, Terminal } from "lucide-svelte";
    import { onMount } from "svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    interface LogEntry {
        timestamp: number;
        level: 'ERROR' | 'WARN' | 'INFO' | 'DEBUG' | 'TRACE';
        target: string;
        message: string;
    }

    let logs = $state<LogEntry[]>([]);
    let isLoading = $state(false);
    let selectedLevel = $state('ALL');
    let searchQuery = $state('');

    const levels = ['ALL', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'];
    const logLevels = levels.map(level => ({
        value: level,
        label: level === 'ALL' ? i18n.t('settings.logs.level_all') : level
    }));

    const MAX_LOGS_DISPLAY = 500;

    const levelColors: Record<string, string> = {
        'ERROR': 'text-destructive font-bold',
        'WARN': 'text-yellow-500 font-semibold',
        'INFO': 'text-foreground',
        'DEBUG': 'text-blue-400/80',
        'TRACE': 'text-fuchsia-400/80'
    };

    let filteredLogs = $derived.by(() => {
        let result = logs;

        if (selectedLevel !== 'ALL') {
            result = result.filter(l => l.level === selectedLevel);
        }

        if (searchQuery.trim() !== '') {
            const query = searchQuery.toLowerCase();
            result = result.filter(l =>
                l.message.toLowerCase().includes(query) ||
                l.target.toLowerCase().includes(query)
            );
        }

        return [...result].reverse().slice(0, MAX_LOGS_DISPLAY);
    });

    async function fetchLogs() {
        if (isLoading) return;
        isLoading = true;
        try {
            logs = await invoke<LogEntry[]>("get_system_logs");
        } catch (e: any) {
            console.error("Failed to fetch logs:", e);
            toast.error(i18n.t(e.key));
        } finally {
            isLoading = false;
        }
    }

    function formatTimestamp(timestamp: number): string {
        const date = new Date(timestamp);
        const timeStr = date.toLocaleTimeString('en-GB', { hour12: false });
        const millis = date.getMilliseconds().toString().padStart(3, '0');
        return `${timeStr}.${millis}`;
    }

    async function copyToClipboard() {
        if (filteredLogs.length === 0) return;
        const text = filteredLogs.map(l =>
            `[${formatTimestamp(l.timestamp)}] [${l.level.padEnd(5)}] [${l.target}] ${l.message}`
        ).join('\n');
        try {
            await navigator.clipboard.writeText(text);
            toast.success(i18n.t('settings.logs.copied_success'));
        } catch (err) {
            console.error(err);
            toast.error(i18n.t('errors.unknown_error'));
        }
    }

    function clearUi() {
        logs = [];
        searchQuery = '';
    }

    onMount(() => {
        fetchLogs();

        const intervalId = setInterval(() => {
            fetchLogs();
        }, 3000);

        return () => {
            clearInterval(intervalId);
        };
    });
</script>

<section class="flex flex-col w-full h-[85vh] max-h-225 bg-transparent font-mono -mt-4 sm:mt-0">

    <div class="flex flex-col lg:flex-row lg:items-center justify-between pb-6 gap-4 shrink-0 border-b border-border/20 mb-4">
        <div class="flex items-center gap-3">
            <div class="p-2.5 bg-primary/5 rounded-xl border border-primary/10 text-primary">
                <Terminal class="size-5" />
            </div>
            <div>
                <h2 class="text-xl md:text-2xl font-black tracking-tight">{i18n.t('settings.logs.title')}</h2>
                <p class="text-xs md:text-sm text-muted-foreground font-medium opacity-80 mt-0.5">{i18n.t('settings.logs.description')}</p>
            </div>
        </div>

        <div class="flex flex-wrap items-center gap-2 lg:gap-3">
            <div class="relative flex items-center w-full sm:w-auto flex-1 sm:flex-none">
                <Search class="absolute left-3 size-4 text-muted-foreground/60" />
                <input
                        type="text"
                        bind:value={searchQuery}
                        placeholder="Search logs..."
                        class="h-10 w-full sm:w-48 lg:w-64 rounded-lg border-none bg-muted/30 hover:bg-muted/50 focus:bg-background pl-10 pr-3 py-1 text-sm shadow-none ring-1 ring-border/50 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                />
            </div>

            <div class="w-full sm:w-32">
                <ResponsiveSelect
                    bind:value={selectedLevel}
                    items={logLevels}
                    class="h-10 rounded-lg border-none bg-muted/30 hover:bg-muted/50 px-4 py-1 text-sm shadow-none ring-1 ring-border/50 transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                />
            </div>

            <div class="h-6 w-px bg-border/40 mx-1 hidden sm:block"></div>

            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-lg hover:bg-muted/50" onclick={fetchLogs} disabled={isLoading} title={i18n.t('settings.logs.btn_refresh') || 'Refresh'}>
                <RefreshCw class="size-4 text-muted-foreground {isLoading ? 'animate-spin' : ''}" />
            </Button>

            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-lg hover:bg-muted/50" onclick={copyToClipboard} disabled={filteredLogs.length === 0} title={i18n.t('settings.logs.btn_copy') || 'Copy'}>
                <Copy class="size-4 text-muted-foreground" />
            </Button>

            <Button variant="ghost" size="icon" class="h-10 w-10 rounded-lg text-destructive/70 hover:text-destructive hover:bg-destructive/10" onclick={clearUi} disabled={logs.length === 0} title={i18n.t('settings.logs.btn_clear_ui') || 'Clear'}>
                <Trash2 class="size-4" />
            </Button>
        </div>
    </div>

    <div class="hidden sm:grid sm:grid-cols-[100px_70px_1fr_3fr] gap-4 px-2 py-2 text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50 shrink-0">
        <div>{i18n.t('settings.logs.col_time')}</div>
        <div>{i18n.t('settings.logs.col_level')}</div>
        <div>{i18n.t('settings.logs.col_target')}</div>
        <div>{i18n.t('settings.logs.col_message')}</div>
    </div>

    <div class="flex-1 overflow-y-auto sm:space-y-0.5 custom-scrollbar bg-transparent -mx-2 px-2">
        {#if isLoading && logs.length === 0}
            <div class="flex flex-col items-center justify-center h-full text-muted-foreground gap-4">
                <RefreshCw class="size-8 animate-spin text-primary/30" />
                <p class="text-sm animate-pulse">{i18n.t('settings.logs.loading')}</p>
            </div>
        {:else if filteredLogs.length === 0}
            <div class="flex flex-col items-center justify-center h-full text-muted-foreground/40 gap-4">
                <Terminal class="size-12 opacity-20" />
                <p class="text-sm">{i18n.t('settings.logs.empty')}</p>
            </div>
        {:else}
            {#each filteredLogs as log, i (log.timestamp.toString() + '_' + i)}
                <div class="flex flex-col sm:grid sm:grid-cols-[100px_70px_1fr_3fr] gap-1 sm:gap-4 items-start hover:bg-muted/30 px-3 py-2.5 sm:px-2 sm:py-2 rounded-lg transition-colors text-[11px] lg:text-xs border-b border-border/10 sm:border-none last:border-none group">

                    <div class="flex sm:contents items-center gap-2 mb-1 sm:mb-0">
                        <div class="text-muted-foreground/60 whitespace-nowrap pt-0.5 group-hover:text-muted-foreground transition-colors">
                            {formatTimestamp(log.timestamp)}
                        </div>
                        <div class={levelColors[log.level] }>
                            {log.level}
                        </div>
                    </div>

                    <div class="text-primary/70 whitespace-nowrap overflow-hidden text-ellipsis pt-0.5 font-medium" title={log.target}>
                        {log.target}
                    </div>
                    <div class="wrap-break-word whitespace-pre-wrap text-foreground/80 leading-relaxed font-mono">
                        {log.message}
                    </div>
                </div>
            {/each}
        {/if}
    </div>
</section>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: rgba(150, 150, 150, 0.1);
        border-radius: 10px;
    }
    .custom-scrollbar:hover::-webkit-scrollbar-thumb {
        background: rgba(150, 150, 150, 0.3);
    }
</style>