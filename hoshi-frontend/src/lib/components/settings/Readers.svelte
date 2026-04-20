<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { i18n } from "@/stores/i18n.svelte.js";
    import MangaReader from "./MangaReader.svelte";
    import NovelReader from "./NovelReader.svelte";
    import type { MangaConfig, NovelConfig } from "@/api/config/types";
    import { BookOpen, BookOpenText } from "lucide-svelte";

    let {
        mangaConfig = $bindable(),
        novelConfig = $bindable(),
        onSave
    }: {
        mangaConfig: MangaConfig,
        novelConfig: NovelConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<div class="space-y-6">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.readers')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.readers_desc')}</p>
    </div>

    <Tabs.Root value="manga_sub" class="w-full">
        <Tabs.List class="grid w-full max-w-[400px] grid-cols-2 rounded-xl h-11 p-1 bg-muted/50 mb-8">
            <Tabs.Trigger value="manga_sub" class="rounded-lg font-bold flex items-center gap-2">
                <BookOpen class="size-4" /> {i18n.t('settings.manga')}
            </Tabs.Trigger>
            <Tabs.Trigger value="novel_sub" class="rounded-lg font-bold flex items-center gap-2">
                <BookOpenText class="size-4" /> {i18n.t('settings.novel')}
            </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="manga_sub" class="focus-visible:outline-none mt-0">
            <MangaReader bind:config={mangaConfig} {onSave} />
        </Tabs.Content>

        <Tabs.Content value="novel_sub" class="focus-visible:outline-none mt-0">
            <NovelReader bind:config={novelConfig} {onSave} />
        </Tabs.Content>
    </Tabs.Root>
</div>