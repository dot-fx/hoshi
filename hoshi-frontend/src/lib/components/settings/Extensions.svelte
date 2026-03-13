<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import type { ExtensionsConfig } from "@/api/config/types";
    import {i18n} from "@/i18n/index.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();
</script>

<section>
    <div class="mb-6">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.extensions')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.extensions_desc')}</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold" for="repoUrl">{i18n.t('settings.repo_url')}</Label>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.repo_url_desc')}</p>
        </div>
        <div class="w-full sm:max-w-md space-y-3">
            <Input
                    id="repoUrl"
                    bind:value={config.repoUrl}
                    placeholder="https://raw.githubusercontent.com/..."
                    class="rounded-xl h-11"
            />
            <div class="flex justify-end">
                <Button variant="secondary" size="sm" class="rounded-lg font-bold" onclick={onSave}>
                    {i18n.t('settings.update_repo')}
                </Button>
            </div>
        </div>
    </div>
</section>