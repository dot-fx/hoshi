<script lang="ts">
    import { contentApi } from "$lib/api/content/content";
    import type { ExtensionSource } from "$lib/api/content/types";
    import * as Table from "$lib/components/ui/table";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import { BookOpen, SearchX } from "lucide-svelte";

    let { cid, contentType, extensions, availableExtensions }: {
        cid: string,
        contentType: string,
        extensions: ExtensionSource[],
        availableExtensions: string[] // This is the array of installed extension names (e.g., ["mangapill"])
    } = $props();

    // Initialize with the first available installed extension, if any
    let selectedExtensionName = $state(availableExtensions.length > 0 ? availableExtensions[0] : "");
    let chapters = $state<any[]>([]);
    let loading = $state(false);

    const basePath = $derived(contentType === "novel" ? "/read-novel" : "/read");

    // Date formatter
    const formatDate = (dateString: string | null) => {
        if (!dateString) return "Unknown";
        return new Intl.DateTimeFormat('en-US', {
            year: 'numeric', month: 'short', day: 'numeric'
        }).format(new Date(dateString));
    };

    // Reactively fetch chapters when the selected extension changes
    $effect(() => {
        if (selectedExtensionName) {
            loadChapters(selectedExtensionName);
        }
    });

    async function loadChapters(extName: string) {
        loading = true;
        try {
            // Make the API call using the extension name to fetch chapters on the fly
            const res = await contentApi.getItems(cid, extName);
            // Ensure we handle the array response correctly based on your Rust backend structure
            chapters = Array.isArray(res.data) ? res.data : [];
        } catch (error) {
            console.error("Failed to load chapters:", error);
            chapters = [];
        } finally {
            loading = false;
        }
    }
</script>

<div class="space-y-6">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <h3 class="text-xl font-semibold">Chapters</h3>

        {#if availableExtensions.length > 0}
            <div class="w-full sm:w-[250px]">
                <Select.Root type="single" bind:value={selectedExtensionName}>
                    <Select.Trigger class="capitalize">
                        {selectedExtensionName || "Select extension"}
                    </Select.Trigger>
                    <Select.Content>
                        {#each availableExtensions as extName}
                            <Select.Item value={extName} class="capitalize">{extName}</Select.Item>
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>
        {/if}
    </div>

    {#if availableExtensions.length === 0}
        <Empty.Root class="border border-dashed py-16">
            <Empty.Header>
                <Empty.Media variant="icon">
                    <BookOpen />
                </Empty.Media>
                <Empty.Title>No sources available</Empty.Title>
                <Empty.Description class="max-w-md mx-auto">
                    Please install a {contentType} extension in the server settings to start reading.
                </Empty.Description>
            </Empty.Header>
        </Empty.Root>

    {:else if loading}
        <div class="space-y-3">
            <Skeleton class="h-10 w-full rounded-md" />
            <Skeleton class="h-10 w-full rounded-md" />
            <Skeleton class="h-10 w-full rounded-md" />
            <Skeleton class="h-10 w-full rounded-md" />
        </div>

    {:else if chapters.length === 0}
        <Empty.Root class="border py-12 bg-muted/10">
            <Empty.Header>
                <Empty.Media variant="icon">
                    <SearchX />
                </Empty.Media>
                <Empty.Title>No Chapters</Empty.Title>
                <Empty.Description>
                    No chapters were found in {selectedExtensionName} for this entry.
                </Empty.Description>
            </Empty.Header>
        </Empty.Root>

    {:else}
        <div class="rounded-md border">
            <Table.Root>
                <Table.Header>
                    <Table.Row>
                        <Table.Head class="w-[80px]">#</Table.Head>
                        <Table.Head>Title</Table.Head>
                        <Table.Head class="hidden md:table-cell">Date</Table.Head>
                        <Table.Head class="text-right">Action</Table.Head>
                    </Table.Row>
                </Table.Header>
                <Table.Body>
                    {#each chapters as chapter}
                        <Table.Row>
                            <Table.Cell class="font-medium">{chapter.number}</Table.Cell>
                            <Table.Cell>
                                <span class="line-clamp-1">{chapter.title || `Chapter ${chapter.number}`}</span>
                            </Table.Cell>
                            <Table.Cell class="hidden md:table-cell text-muted-foreground">
                                {formatDate(chapter.releaseDate)} </Table.Cell>
                            <Table.Cell class="text-right">
                                <Button size="sm" variant="secondary" href={`${basePath}/${cid}/${selectedExtensionName}/${chapter.number}`}>
                                    Read
                                </Button>
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </div>
    {/if}
</div>