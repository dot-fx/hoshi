<script lang="ts">
    import * as Select from "$lib/components/ui/select";
    import { cn } from "$lib/utils";

    interface Item {
        value: string;
        label: string;
    }

    let {
        value = $bindable(),
        items = [] as Item[],
        placeholder = "select...",
        class: className = "",
        label = ""
    } = $props();

    let isDesktop = $state(true);

    $effect(() => {
        const mediaQuery = window.matchMedia("(min-width: 768px)");
        // Seteo inicial
        isDesktop = mediaQuery.matches;

        const handler = (e: MediaQueryListEvent) => {
            isDesktop = e.matches;
        };

        mediaQuery.addEventListener("change", handler);
        return () => mediaQuery.removeEventListener("change", handler);
    });

    let selectedLabel = $derived(
        items.find(i => i.value === value)?.label || placeholder
    );
</script>

{#if isDesktop}
    <Select.Root type="single" bind:value>
        <Select.Trigger class={cn("w-full", className)}>
            {selectedLabel}
        </Select.Trigger>
        <Select.Content class="z-[999] pointer-events-auto">
            {#each items as item}
                <Select.Item value={item.value} label={item.label}>
                    {item.label}
                </Select.Item>
            {/each}
        </Select.Content>
    </Select.Root>
{:else}
    <div class="relative w-full">
        {#if label}
            <label class="sr-only" for="native-select">{label}</label>
        {/if}
        <select
                id="native-select"
                bind:value
                class={cn(
                "flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 appearance-none",
                className
            )}
        >
            {#if !value}
                <option value="" disabled selected>{placeholder}</option>
            {/if}
            {#each items as item}
                <option value={item.value}>
                    {item.label}
                </option>
            {/each}
        </select>

        <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-muted-foreground">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-50">
                <path d="m6 9 6 6 6-6"/>
            </svg>
        </div>
    </div>
{/if}