<script lang="ts">
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import type { ExtensionsConfig } from "@/api/config/types";

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
        <h2 class="text-2xl font-bold tracking-tight">Extensions</h2>
        <p class="text-sm text-muted-foreground mt-1">Manage content repositories and sources.</p>
    </div>

    <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-4 py-6 border-b border-border/40">
        <div class="space-y-1 pr-4 flex-1">
            <Label class="text-base font-bold" for="repoUrl">Extension Repository URL</Label>
            <p class="text-sm text-muted-foreground">URL where extensions and plugins are fetched from.</p>
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
                    Update Repo
                </Button>
            </div>
        </div>
    </div>
</section>