<script lang="ts">
    import { Button } from '@/components/ui/button';
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { i18n } from "@/i18n/index.svelte.js";

    let { onTerminate } = $props<{ onTerminate: () => void }>();

    let showCloseConfirm = $state(false);

    function handleConfirm() {
        showCloseConfirm = false;
        onTerminate();
    }
</script>

<Button
        onclick={() => showCloseConfirm = true}
        variant="destructive"
        size="sm"
        class="font-bold rounded-lg shadow-lg"
>
    {i18n.t('watchparty.terminate_btn')}
</Button>

<AlertDialog.Root bind:open={showCloseConfirm}>
    <AlertDialog.Content class="border-border/40 bg-background/95 backdrop-blur-xl">
        <AlertDialog.Header>
            <AlertDialog.Title class="text-xl font-black">{i18n.t('watchparty.terminate_title')}</AlertDialog.Title>
            <AlertDialog.Description class="text-muted-foreground font-medium">
                {i18n.t('watchparty.terminate_desc')}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel class="rounded-xl font-bold">{i18n.t('watchparty.cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action
                    onclick={handleConfirm}
                    class="bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded-xl font-bold"
            >
                {i18n.t('watchparty.terminate_confirm')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>