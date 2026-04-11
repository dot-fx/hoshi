<script lang="ts">
    import { browser } from '$app/environment';
    import { Minus, Square, X } from 'lucide-svelte';
    import { layoutState } from '@/layout.svelte.js';

    const isTauri = browser && '__TAURI__' in window;
    const isMobile = browser && /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);
    const showTitlebar = isTauri && !isMobile;

    let winPromise: Promise<any> | null = null;

    function getWin() {
        if (!showTitlebar) return null;

        if (!winPromise) {
            winPromise = import('@tauri-apps/api/window')
                .then(m => m.getCurrentWindow());
        }

        return winPromise;
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
    <div class="absolute top-0 left-0 h-7 flex justify-between items-center bg-transparent select-none z-[60] w-full">

        <div data-tauri-drag-region class="flex-1 h-full pointer-events-auto flex items-center justify-center pl-24">
            <span class="text-[11px] font-medium text-muted-foreground/80 tracking-wide line-clamp-1 pointer-events-none">
                {layoutState.title || ''}
            </span>
        </div>

        <div class="flex h-full shrink-0">
            <button onclick={minimize} class="h-full w-[40px] hover:bg-muted/40 text-muted-foreground/80 hover:text-foreground transition-none inline-flex items-center justify-center" tabindex="-1">
                <Minus class="size-[13px] stroke-[1.5]" />
            </button>
            <button onclick={maximize} class="h-full w-[40px] hover:bg-muted/40 text-muted-foreground/80 hover:text-foreground transition-none inline-flex items-center justify-center" tabindex="-1">
                <Square class="size-[11px] stroke-[1.5]" />
            </button>
            <button onclick={close} class="h-full w-[40px] hover:bg-[#e81123] text-muted-foreground/80 hover:text-white transition-none inline-flex items-center justify-center" tabindex="-1">
                <X class="size-[13px] stroke-[1.5]" />
            </button>
        </div>
    </div>
{/if}