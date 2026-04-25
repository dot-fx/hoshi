<script lang="ts">
    import type { Character, StaffMember } from "$lib/api/content/types";
    import { Mic2, User, Users } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";

    const formatRole = (role: string | undefined | null) => {
        if (!role) return '';
        const normalized = role.toUpperCase().replace(/\s+/g, '_');
        const key = `roles.${normalized}` as any;
        const translated = i18n.t(key);

        return translated === key ? role : translated;
    };

    let { characters, staff }: { characters: Character[], staff: StaffMember[] } = $props();
</script>

<div class="space-y-6">
    {#if staff && staff.length > 0}
        <section class="space-y-3">
            <h3 class="text-sm font-bold uppercase tracking-wider text-muted-foreground/70 px-1">
                {i18n.t('content.staff')}
            </h3>

            <div class="flex overflow-x-auto pb-2 -mx-4 px-4 snap-x snap-mandatory hide-scrollbar gap-2">
                {#each staff as person}
                    <div class="flex items-center gap-2 bg-muted/5 pl-1.5 pr-3 py-1.5 rounded-full border border-border/20 shrink-0 snap-start hover:bg-muted/10 transition-colors">
                        {#if person.image && !person.image.includes('default.jpg')}
                            <img src={person.image} alt={person.name} class="w-7 h-7 object-cover rounded-full bg-card shrink-0" />
                        {:else}
                            <div class="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0">
                                <User class="h-3 w-3 text-muted-foreground" />
                            </div>
                        {/if}
                        <div class="flex flex-col overflow-hidden max-w-[120px]">
                            <span class="font-medium text-[11px] text-foreground truncate leading-tight">{person.name}</span>
                            <span class="text-[9px] text-muted-foreground truncate uppercase opacity-70">{formatRole(person.role)}</span>
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}

    {#if characters && characters.length > 0}
        <section class="space-y-3">
            <h3 class="text-sm font-bold uppercase tracking-wider text-muted-foreground/70 flex items-center gap-2 px-1">
                <Users class="h-4 w-4" /> {i18n.t('content.characters')}
            </h3>

            <div class="flex overflow-x-auto pb-2 -mx-4 px-4 snap-x snap-mandatory hide-scrollbar gap-3">
                {#each characters as char}
                    <div class="flex gap-2.5 bg-muted/5 p-2 rounded-lg border border-border/20 shrink-0 w-[200px] snap-start hover:bg-muted/10 transition-colors">
                        <img src={char.image} alt={char.name} class="w-10 h-14 object-cover rounded bg-card border border-border/40 shrink-0" />

                        <div class="flex flex-col justify-center overflow-hidden w-full">
                            <span class="font-medium text-xs text-foreground truncate">{char.name}</span>
                            <span class="text-[10px] text-muted-foreground capitalize truncate">{formatRole(char.role)}</span>
                            {#if char.actor}
                                <div class="mt-1 flex items-center gap-1 text-[9px] font-medium text-primary/80">
                                    <Mic2 class="h-2.5 w-2.5 shrink-0" />
                                    <span class="truncate">{char.actor}</span>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>

<style>
    .hide-scrollbar::-webkit-scrollbar { display: none; }
    .hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>