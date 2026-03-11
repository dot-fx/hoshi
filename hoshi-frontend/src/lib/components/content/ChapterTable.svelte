<script lang="ts">
    import { contentApi } from "$lib/api/content/content";
    import { extensions } from "$lib/extensions.svelte";
    import type { ExtensionSource } from "$lib/api/content/types";

    import * as Table from "$lib/components/ui/table";
    import * as Select from "$lib/components/ui/select";
    import * as Empty from "$lib/components/ui/empty";
    import * as Pagination from "$lib/components/ui/pagination";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { Button } from "$lib/components/ui/button";
    import { BookOpen, SearchX } from "lucide-svelte";
    import { i18n } from "$lib/i18n/index.svelte";

    let {
        cid,
        contentType
    }: {
        cid: string,
        contentType: string
    } = $props();

    let availableExtensions = $derived(
        contentType === "manga" ? extensions.manga.map(e => e.id) :
            contentType === "novel" ? extensions.novel.map(e => e.id) : []
    );

    let selectedExtensionName = $state("");
    let chapters = $state<any[]>([]);
    let loading = $state(false);

    let currentPage = $state(1);
    const perPage = 10;

    const basePath = $derived(contentType === "novel" ? "/read-novel" : "/read");

    let paginatedChapters = $derived(
        chapters.slice((currentPage - 1) * perPage, currentPage * perPage)
    );

    const formatDate = (dateString: string | null) => {
        if (!dateString) return i18n.t('unknown_date');
        return new Intl.DateTimeFormat(i18n.locale, {
            year: 'numeric', month: 'short', day: 'numeric'
        }).format(new Date(dateString));
    };

    $effect(() => {
        if (!selectedExtensionName && availableExtensions.length > 0) {
            selectedExtensionName = availableExtensions[0];
        }
    });

    $effect(() => {
        if (selectedExtensionName) {
            loadChapters(selectedExtensionName);
        }
    });

    async function loadChapters(extName: string) {
        loading = true;
        currentPage = 1;
        try {
            const res = await contentApi.getItems(cid, extName);
            chapters = Array.isArray(res) ? res : [];
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
        <h3 class="text-xl font-semibold">{i18n.t('chapters_title')}</h3>

        {#if availableExtensions.length > 0}
            <div class="w-full sm:w-[250px]">
                <Select.Root type="single" bind:value={selectedExtensionName}>
                    <Select.Trigger class="capitalize">
                        {selectedExtensionName || i18n.t('select_extension')}
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
                <Empty.Title>{i18n.t('no_sources_available')}</Empty.Title>
                <Empty.Description class="max-w-md mx-auto">
                    {i18n.t('install_extension_prompt').replace('{contentType}', contentType)}
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
                <Empty.Title>{i18n.t('no_chapters')}</Empty.Title>
                <Empty.Description>
                    {i18n.t('no_chapters_found').replace('{extension}', selectedExtensionName)}
                </Empty.Description>
            </Empty.Header>
        </Empty.Root>

    {:else}
        <div class="rounded-md border">
            <Table.Root>
                <Table.Header>
                    <Table.Row>
                        <Table.Head class="w-[80px]">{i18n.t('table_number')}</Table.Head>
                        <Table.Head>{i18n.t('table_title')}</Table.Head>
                        <Table.Head class="hidden md:table-cell">{i18n.t('table_date')}</Table.Head>
                        <Table.Head class="text-right">{i18n.t('table_action')}</Table.Head>
                    </Table.Row>
                </Table.Header>
                <Table.Body>
                    {#each paginatedChapters as chapter (chapter.id || chapter.number)}
                        <Table.Row>
                            <Table.Cell class="font-medium text-muted-foreground">{chapter.number ?? chapter.unitNumber}</Table.Cell>

                            <Table.Cell>
                                <span class="line-clamp-1 font-medium">
                                    {chapter.title?.trim() ? chapter.title : `${i18n.t('chapter')} ${chapter.number ?? chapter.unitNumber}`}
                                </span>
                            </Table.Cell>

                            <Table.Cell class="hidden md:table-cell text-muted-foreground">
                                {formatDate(chapter.releaseDate)}
                            </Table.Cell>

                            <Table.Cell class="text-right">
                                <Button size="sm" variant="secondary" href={`${basePath}/${cid}/${selectedExtensionName}/${chapter.number ?? chapter.unitNumber}`}>
                                    {i18n.t('read')}
                                </Button>
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </div>

        {#if chapters.length > perPage}
            <div class="mt-4 flex justify-center">
                <Pagination.Root count={chapters.length} {perPage} bind:page={currentPage}>
                    {#snippet children({ pages, currentPage })}
                        <Pagination.Content>
                            <Pagination.Item>
                                <Pagination.PrevButton />
                            </Pagination.Item>
                            {#each pages as page (page.key)}
                                {#if page.type === "ellipsis"}
                                    <Pagination.Item>
                                        <Pagination.Ellipsis />
                                    </Pagination.Item>
                                {:else}
                                    <Pagination.Item>
                                        <Pagination.Link {page} isActive={currentPage === page.value}>
                                            {page.value}
                                        </Pagination.Link>
                                    </Pagination.Item>
                                {/if}
                            {/each}
                            <Pagination.Item>
                                <Pagination.NextButton />
                            </Pagination.Item>
                        </Pagination.Content>
                    {/snippet}
                </Pagination.Root>
            </div>
        {/if}
    {/if}
</div>