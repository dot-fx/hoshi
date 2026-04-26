<script lang="ts">
    import type { Component } from 'svelte';
    import { ChevronRight } from 'lucide-svelte';
    import { Button } from "@/components/ui/button";

    interface Props {
        icon:       Component;
        label:      string;
        value?:     string;
        onclick:    () => void;
        children?:  import('svelte').Snippet;
    }

    let { icon: Icon, label, value, onclick, children }: Props = $props();
</script>

<Button
        variant="ghost"
        class="group flex items-center justify-between w-full px-3 py-2.5 h-auto
           rounded-sm text-foreground hover:bg-accent hover:text-accent-foreground"
        {onclick}
>
    <div class="flex items-center gap-3">
        <div class="flex items-center justify-center w-8 h-8 rounded-sm
                    bg-muted group-hover:bg-accent transition-colors duration-100">
            <Icon class="w-4 h-4 text-muted-foreground group-hover:text-accent-foreground" />
        </div>
        <span class="text-sm font-medium">{label}</span>
    </div>

    {#if children}
        {@render children()}
    {:else}
        <div class="flex items-center gap-2">
            {#if value}
                <span class="text-xs text-muted-foreground font-normal">{value}</span>
            {/if}
            <ChevronRight class="w-4 h-4 text-muted-foreground/50 group-hover:text-muted-foreground transition-colors" />
        </div>
    {/if}
</Button>