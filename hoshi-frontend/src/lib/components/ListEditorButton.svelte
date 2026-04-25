<script lang="ts">
    import { listStore } from "@/app/list.svelte";
    import { openListEditor } from "@/stores/layout.svelte";
    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import { Check, BookmarkPlus } from "lucide-svelte";

    let { cid, title, contentType, coverImage } = $props<{
        cid: string;
        title: string;
        contentType: string;
        coverImage?: string;
    }>();

    const isInList = $derived(listStore.hasCid(cid));

    function handleClick() {
        openListEditor({
            cid,
            title,
            contentType,
            coverImage
        });
    }
</script>

<Button
        size="icon"
        variant="secondary"
        class="rounded-sm w-12 h-12"
        onclick={handleClick}
>
    {#if listStore.isLoading}
        <Spinner class="w-4 h-4" />
    {:else if isInList}
        <Check class="w-4 h-4 text-green-500" />
    {:else}
        <BookmarkPlus class="w-4 h-4" />
    {/if}
</Button>