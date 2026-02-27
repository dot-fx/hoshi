<script lang="ts">
    import type { Character, StaffMember } from "$lib/api/content/types";
    import { Mic2, User } from "lucide-svelte";

    let { characters, staff }: { characters: Character[], staff: StaffMember[] } = $props();
</script>

<div class="space-y-10">
    {#if characters && characters.length > 0}
        <section class="space-y-4">
            <h3 class="text-xl font-semibold tracking-tight flex items-center gap-2">
                <User class="h-5 w-5 text-primary" /> Characters
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {#each characters as char}
                    <div class="flex gap-3 bg-muted/10 p-2.5 rounded-xl border border-border/40 hover:bg-muted/30 hover:border-primary/30 transition-all shadow-sm">
                        <img src={char.image} alt={char.name} class="w-14 h-20 object-cover rounded-lg bg-card border border-border/50 flex-shrink-0" />
                        <div class="flex flex-col justify-center overflow-hidden">
                            <span class="font-medium text-foreground truncate">{char.name}</span>
                            <span class="text-xs text-muted-foreground capitalize">{char.role}</span>
                            {#if char.actor}
                                <div class="mt-auto pt-1 flex items-center gap-1.5 text-xs font-medium text-primary/90">
                                    <Mic2 class="h-3 w-3" />
                                    <span class="truncate">{char.actor}</span>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}

    {#if staff && staff.length > 0}
        <section class="space-y-4">
            <h3 class="text-xl font-semibold tracking-tight">Staff</h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {#each staff as person}
                    <div class="flex gap-3 bg-muted/5 p-2 rounded-xl border border-border/30 hover:bg-muted/20 transition-all">
                        {#if person.image && !person.image.includes('default.jpg')}
                            <img src={person.image} alt={person.name} class="w-12 h-16 object-cover rounded-lg bg-card" />
                        {:else}
                            <div class="w-12 h-16 rounded-lg bg-muted flex items-center justify-center">
                                <User class="h-5 w-5 text-muted-foreground" />
                            </div>
                        {/if}
                        <div class="flex flex-col justify-center overflow-hidden">
                            <span class="font-medium text-sm text-foreground truncate">{person.name}</span>
                            <span class="text-xs text-muted-foreground line-clamp-2 leading-tight mt-0.5">{person.role}</span>
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>