<script lang="ts">
    import { onMount } from 'svelte';
    import { X, Minus, Square } from 'lucide-svelte';

    let { title }: { title: string } = $props();
    let isTauri = $state(false);

    onMount(() => {
        // Detectar si estamos corriendo en Tauri
        if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
            isTauri = true;
        }
    });

    async function minimize() {
        const { Window } = await import('@tauri-apps/api/window');
        Window.getCurrent().minimize();
    }

    async function maximize() {
        const { Window } = await import('@tauri-apps/api/window');
        const win = Window.getCurrent();
        await win.isMaximized() ? win.unmaximize() : win.maximize();
    }

    async function close() {
        const { Window } = await import('@tauri-apps/api/window');
        Window.getCurrent().close();
    }
</script>

{#if isTauri}
    <div data-tauri-drag-region class="h-8 flex justify-between items-center bg-background border-b border-border select-none z-[100] shrink-0 w-full">

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