<script lang="ts">
    import { ChevronLeft, Check } from 'lucide-svelte';
    import { fly } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { Button } from "@/components/ui/button";

    interface Option {
        id:    string | number;
        label: string;
    }

    interface Props {
        label:    string;
        options:  Option[];
        current:  string | number;
        onSelect: (id: string) => void;
        onBack:   () => void;
    }

    let { label, options, current, onSelect, onBack }: Props = $props();
</script>

<div
        class="flex flex-col py-1"
        in:fly={{ x: 12, duration: 180, easing: cubicOut }}
>
    <Button
            variant="ghost"
            class="flex items-center justify-start gap-2.5 w-full px-3 py-2.5 h-auto mb-1
               rounded-sm border-b border-border text-foreground hover:bg-accent"
            onclick={onBack}
    >
        <ChevronLeft class="w-4 h-4 text-muted-foreground" />
        <span class="text-sm font-semibold">{label}</span>
    </Button>

    <div class="flex flex-col max-h-64 overflow-y-auto">
        {#each options as opt (opt.id)}
            {@const isActive = current === opt.id}
            <Button
                    variant="ghost"
                    class="flex items-center justify-start gap-3 w-full px-3 py-2.5 h-auto rounded-sm
                       {isActive
                           ? 'text-primary hover:text-primary hover:bg-accent'
                           : 'text-foreground hover:bg-accent'}"
                    onclick={() => { onSelect(String(opt.id)); onBack(); }}
            >
                <div class="flex items-center justify-center w-5 h-5 shrink-0">
                    {#if isActive}
                        <Check class="w-4 h-4 text-primary" />
                    {/if}
                </div>
                <span class="flex-1 text-left text-sm {isActive ? 'font-medium' : 'text-muted-foreground'}">
                    {opt.label}
                </span>
            </Button>
        {/each}
    </div>
</div>