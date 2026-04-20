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

<div class="space-y-10">
    {#if characters && characters.length > 0}
        <section class="space-y-4">
            <h3 class="text-xl font-semibold tracking-tight flex items-center gap-2 px-1">
                <Users class="h-5 w-5 text-primary" /> {i18n.t('content.characters')}
            </h3>

            <div class="flex overflow-x-auto pb-4 -mx-4 px-4 sm:hidden snap-x snap-mandatory hide-scrollbar gap-3">
                {#each characters as char}
                    <div class="flex gap-2.5 bg-muted/10 p-2 rounded-xl border border-border/40 shrink-0 w-[240px] snap-start">
                        <img src={char.image} alt={char.name} class="w-12 h-16 object-cover rounded-md bg-card border border-border/50 shrink-0" />
                        <div class="flex flex-col justify-center overflow-hidden w-full">
                            <span class="font-medium text-sm text-foreground truncate">{char.name}</span>
                            <span class="text-[10px] text-muted-foreground capitalize truncate">{formatRole(char.role)}</span>
                            {#if char.actor}
                                <div class="mt-auto pt-0.5 flex items-center gap-1 text-[10px] font-medium text-primary/90">
                                    <Mic2 class="h-2.5 w-2.5 shrink-0" />
                                    <span class="truncate">{char.actor}</span>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>

            <div class="hidden sm:grid grid-cols-2 lg:grid-cols-3 gap-4">
                {#each characters as char}
                    <div class="flex gap-3 bg-muted/10 p-2.5 rounded-xl border border-border/40 hover:bg-muted/30 hover:border-primary/30 transition-all shadow-sm">
                        <img src={char.image} alt={char.name} class="w-14 h-20 object-cover rounded-lg bg-card border border-border/50 shrink-0" />
                        <div class="flex flex-col justify-center overflow-hidden">
                            <span class="font-medium text-foreground truncate">{char.name}</span>
                            <span class="text-xs text-muted-foreground capitalize">{formatRole(char.role)}</span>
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
            <h3 class="text-xl font-semibold tracking-tight px-1">{i18n.t('content.staff')}</h3>

            <div class="flex overflow-x-auto pb-4 -mx-4 px-4 sm:hidden snap-x snap-mandatory hide-scrollbar gap-3">
                {#each staff as person}
                    <div class="flex gap-2 bg-muted/5 p-2 rounded-xl border border-border/30 shrink-0 w-[200px] snap-start">
                        {#if person.image && !person.image.includes('default.jpg')}
                            <img src={person.image} alt={person.name} class="w-10 h-14 object-cover rounded-md bg-card shrink-0" />
                        {:else}
                            <div class="w-10 h-14 rounded-md bg-muted flex items-center justify-center shrink-0">
                                <User class="h-4 w-4 text-muted-foreground" />
                            </div>
                        {/if}
                        <div class="flex flex-col justify-center overflow-hidden w-full">
                            <span class="font-medium text-xs text-foreground truncate">{person.name}</span>
                            <span class="text-[10px] text-muted-foreground line-clamp-2 leading-tight mt-0.5">{formatRole(person.role)}</span>
                        </div>
                    </div>
                {/each}
            </div>

            <div class="hidden sm:grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {#each staff as person}
                    <div class="flex gap-3 bg-muted/5 p-2 rounded-xl border border-border/30 hover:bg-muted/20 transition-all">
                        {#if person.image && !person.image.includes('default.jpg')}
                            <img src={person.image} alt={person.name} class="w-12 h-16 object-cover rounded-lg bg-card shrink-0" />
                        {:else}
                            <div class="w-12 h-16 rounded-lg bg-muted flex items-center justify-center shrink-0">
                                <User class="h-5 w-5 text-muted-foreground" />
                            </div>
                        {/if}
                        <div class="flex flex-col justify-center overflow-hidden">
                            <span class="font-medium text-sm text-foreground truncate">{person.name}</span>
                            <span class="text-xs text-muted-foreground line-clamp-2 leading-tight mt-0.5">{formatRole(person.role)}</span>
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>