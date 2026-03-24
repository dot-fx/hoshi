<script lang="ts">
    import { browser } from '$app/environment';
    import { Minus, Square, X } from 'lucide-svelte';
    import { layoutState } from '@/layout.svelte.js';

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
    <div data-tauri-drag-region class="h-9 flex justify-between items-center bg-background border-b border-border/40 select-none z-50 shrink-0 w-full">

        <div data-tauri-drag-region class="flex items-center gap-2.5 pl-3 h-full flex-1 pointer-events-auto">
            <div class="h-4 w-4 rounded-[4px] bg-primary flex items-center justify-center text-primary-foreground text-[9px] font-black shadow-sm pointer-events-none">
                H
            </div>
            <span class="text-xs font-medium text-muted-foreground/80 tracking-wide line-clamp-1 pointer-events-none">
                {layoutState.title || 'hoshi'}
            </span>
        </div>

        <div class="flex h-full shrink-0">
            <button
                    onclick={minimize}
                    class="h-full w-[46px] hover:bg-muted/60 text-muted-foreground/80 hover:text-foreground transition-none inline-flex items-center justify-center"
                    tabindex="-1"
            >
                <Minus class="size-[15px] stroke-[1.5]" />
            </button>
            <button
                    onclick={maximize}
                    class="h-full w-[46px] hover:bg-muted/60 text-muted-foreground/80 hover:text-foreground transition-none inline-flex items-center justify-center"
                    tabindex="-1"
            >
                <Square class="size-[13px] stroke-[1.5]" />
            </button>
            <button
                    onclick={close}
                    class="h-full w-[46px] hover:bg-[#e81123] text-muted-foreground/80 hover:text-white transition-none inline-flex items-center justify-center"
                    tabindex="-1"
            >
                <X class="size-[15px] stroke-[1.5]" />
            </button>
        </div>
    </div>
{/if}