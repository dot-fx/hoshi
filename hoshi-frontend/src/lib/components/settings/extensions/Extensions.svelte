<script lang="ts">
    import { onMount } from "svelte";
    import { extensions } from "@/stores/extensions.svelte.js";

    import * as Tabs from "$lib/components/ui/tabs";
    import { Puzzle, Store } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";

    import InstalledExtensions from "$lib/components/settings/extensions/Installed.svelte";
    import ExtensionMarketplace from "$lib/components/settings/extensions/Marketplace.svelte";
    import type { ExtensionsConfig } from "@/api/config/types";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    onMount(() => {
        extensions.load();
    });
</script>

<section class="space-y-6 w-full">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.extensions')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.extension_section.extensions_desc')}</p>
    </div>

    <Tabs.Root value="installed" class="w-full">
        <Tabs.List class="grid w-full grid-cols-2 max-w-[400px] mb-6 rounded-xl">
            <Tabs.Trigger value="installed" class="font-bold gap-2 rounded-lg">
                <Puzzle class="w-4 h-4" />
                {i18n.t('settings.extension_section.installed_extensions')}
            </Tabs.Trigger>
            <Tabs.Trigger value="explore" class="font-bold gap-2 rounded-lg">
                <Store class="w-4 h-4" />
                {i18n.t('marketplace.title')}
            </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="installed" class="space-y-6 outline-none">
            <InstalledExtensions bind:config={config} {onSave} />
        </Tabs.Content>

        <Tabs.Content value="explore" class="space-y-6 outline-none">
            <ExtensionMarketplace bind:config={config} {onSave} />
        </Tabs.Content>
    </Tabs.Root>
</section>