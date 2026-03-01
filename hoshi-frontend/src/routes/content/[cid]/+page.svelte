<script lang="ts">
    import { page } from "$app/state";
    import ContentLayout from "$lib/components/content/ContentLayout.svelte";
</script>

<ContentLayout cid={page.params.cid || ""} />