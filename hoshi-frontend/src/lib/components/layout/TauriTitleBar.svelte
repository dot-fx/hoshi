<script lang="ts">
    import { browser } from '$app/environment';
    import { Minus, Square, X } from 'lucide-svelte';
    import { layoutState } from '$lib/layoutState.svelte';

    const isTauri = browser && '__TAURI__' in window;
    const isMobile = browser && /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
    const showTitlebar = isTauri && !isMobile;

    async function getWin() {
        if (!showTitlebar) return null;
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

{#if showTitlebar}
    <div data-tauri-drag-region class="h-11 flex justify-between items-center bg-background/95 backdrop-blur-sm border-b border-border select-none z-50 shrink-0 w-full transition-colors">
        <div class="flex items-center gap-3 pl-4 pointer-events-none">
            <div class="h-5 w-5 rounded-md bg-primary/20 flex items-center justify-center text-primary text-[11px] font-black shadow-sm">
                H
            </div>
            <span class="text-sm font-semibold text-muted-foreground tracking-tight line-clamp-1">{layoutState.title}</span>
        </div>
        <div class="flex h-full">
            <button onclick={minimize} class="h-full w-12 hover:bg-muted/80 text-muted-foreground transition-colors inline-flex items-center justify-center"><Minus class="size-4" /></button>
            <button onclick={maximize} class="h-full w-12 hover:bg-muted/80 text-muted-foreground transition-colors inline-flex items-center justify-center"><Square class="size-3.5" /></button>
            <button onclick={close} class="h-full w-12 hover:bg-destructive hover:text-destructive-foreground text-muted-foreground transition-colors inline-flex items-center justify-center"><X class="size-4" /></button>
        </div>
    </div>
{/if}