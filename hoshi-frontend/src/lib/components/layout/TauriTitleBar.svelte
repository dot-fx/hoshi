<script lang="ts">
    import { browser } from '$app/environment';
    import { Minus, Square, X } from 'lucide-svelte';

    let { title }: { title: string } = $props();

    const isTauri = browser && '__TAURI__' in window;

    async function getWin() {
        if (!isTauri) return null;
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        return getCurrentWindow();
    }

    async function minimize() {
        const win = await getWin();
        win?.minimize();
    }

    async function maximize() {
        const win = await getWin();
        if (!win) return;
        const maximized = await win.isMaximized();
        maximized ? await win.unmaximize() : await win.maximize();
    }

    async function close() {
        const win = await getWin();
        win?.close();
    }
</script>

{#if isTauri}
    <div data-tauri-drag-region class="h-8 flex justify-between items-center bg-background border-b border-border select-none z-100 shrink-0 w-full">

        <div class="flex items-center gap-2 pl-3 pointer-events-none">
            <div class="h-4 w-4 rounded bg-primary/20 flex items-center justify-center text-primary text-[10px] font-bold">H</div>
            <span class="text-xs font-semibold text-muted-foreground">{title}</span>
        </div>

        <div class="flex h-full">
            <button onclick={minimize} class="h-full px-3 hover:bg-muted text-muted-foreground transition-colors inline-flex items-center justify-center">
                <Minus class="size-3.5" />
            </button>

            <button onclick={maximize} class="h-full px-3 hover:bg-muted text-muted-foreground transition-colors inline-flex items-center justify-center">
                <Square class="size-3" />
            </button>

            <button onclick={close} class="h-full px-3 hover:bg-destructive hover:text-destructive-foreground text-muted-foreground transition-colors inline-flex items-center justify-center">
                <X class="size-3.5" />
            </button>
        </div>

    </div>
{/if}