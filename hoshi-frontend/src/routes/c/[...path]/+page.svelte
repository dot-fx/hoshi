<script lang="ts">
    import { fade, fly } from "svelte/transition";

    import { i18n } from "@/stores/i18n.svelte.js";
    import { primaryMetadata } from "@/api/content/types";
    import Episodes from "@/components/content/Episodes.svelte";
    import Chapters from "@/components/content/Chapters.svelte";
    import CastAndStaff from "@/components/content/CastAndStaff.svelte";
    import Relations from "@/components/content/Relations.svelte";
    import TrackerManager from "@/components/modals/TrackerManager.svelte";
    import ExtensionManager from '@/components/modals/ExtensionManager.svelte';
    import { appConfig } from "@/stores/config.svelte";

    import { Button } from "@/components/ui/button";
    import { Spinner } from "@/components/ui/spinner";
    import {
        AlertCircle, ChevronDown, ChevronUp
    } from "lucide-svelte";

    import { ContentDetailState } from "@/app/content.svelte";
    import { layoutState } from "@/stores/layout.svelte";
    import ContentHero from "@/components/hero/ContentHero.svelte";
    import ListEditorButton from "@/components/ListEditorButton.svelte";

    const detail = new ContentDetailState();

    let showTrackerModal = $state(false);
    let showExtensionModal = $state(false);
    let synopsisExpanded = $state(false);

    $effect(() => {
        if (detail.synopsisElement && !synopsisExpanded) {
            detail.canTruncate = detail.synopsisElement.scrollHeight > detail.synopsisElement.clientHeight;
        }
    });

    $effect(() => {
        layoutState.showBack = true;
        layoutState.backUrl = "/";
        layoutState.headerAction = headerAction;
    });
</script>

{#snippet headerAction()}
    {#if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}

        <div class="sm:hidden">
            <ListEditorButton
                    cid={detail.fullContent.content.cid}
                    title={displayTitle}
                    contentType={detail.fullContent.content.contentType}
                    coverImage={meta?.coverImage}
            />
        </div>
    {/if}
{/snippet}

<svelte:head>
    {#if detail.isLoading}
        <title>{i18n.t('content.loading')}</title>
    {:else if detail.error}
        <title>Error</title>
    {:else if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const title = meta?.titleI18n?.[pref] || meta?.title || i18n.t('content.details')}
        <title>{title}</title>
    {/if}
</svelte:head>

<div class="min-h-screen bg-background">
    {#if detail.isLoading}
        <div class="flex h-[85vh] w-full items-center justify-center">
            <Spinner class="w-10 h-10 text-muted-foreground/20" />
        </div>

    {:else if detail.error}
        <div class="flex h-[85vh] flex-col items-center justify-center gap-4">
            <AlertCircle class="w-12 h-12 text-destructive opacity-20" />
            <p class="text-lg text-muted-foreground font-medium">{i18n.t(detail.error?.key || 'errors.error')}</p>
            <Button variant="outline" class="rounded-full" onclick={() => detail.retry()}>
                {i18n.t('content.retry')}
            </Button>
        </div>

    {:else if detail.fullContent}
        {@const meta = primaryMetadata(detail.fullContent, appConfig.data?.content?.preferredMetadataProvider)}
        {@const isMovie = meta?.subtype === 'MOVIE'}
        {@const pref = appConfig.data?.ui?.titleLanguage || 'romaji'}
        {@const displayTitle = meta?.titleI18n?.[pref] || meta?.title || ''}
        {@const isAdultContent = detail.fullContent.content.nsfw || meta?.genres?.some(g => ['hentai', 'adult'].includes(g.toLowerCase()))}
        {@const shouldBlur = isAdultContent && (appConfig.data?.general?.blurAdultContent ?? true)}
        {@const isAnime = detail.fullContent.content.contentType === 'anime'}
        {@const hasCastOrStaff = (meta?.characters?.length ?? 0) > 0 || (meta?.staff?.length ?? 0) > 0}
        {@const hasRelations = detail.relations.length > 0 || detail.relationsLoading}

        <div class="absolute top-0 inset-x-0 h-[62vh] md:h-[72vh] overflow-hidden pointer-events-none" in:fade={{ duration: 1000 }}>
            <img
                    src={meta?.bannerImage || meta?.coverImage}
                    alt=""
                    class="w-full h-full object-cover {shouldBlur ? 'blur-3xl scale-125 opacity-5' : 'opacity-45'}"
            />
            <div class="absolute inset-0 bg-gradient-to-b from-transparent via-background/30 to-background"></div>
            <div class="absolute inset-0 bg-gradient-to-b from-transparent via-background/30 to-background"></div>
            <div class="absolute top-0 inset-x-0 h-24 bg-gradient-to-b from-background/60 to-transparent"></div>
        </div>

        <ContentHero
                fullContent={detail.fullContent}
                {meta}
                {displayTitle}
                {isAnime}
                bind:showTrackerModal
                bind:showExtensionModal
                onWatchNow={() => detail.watchNow()}
        />

        <div class="relative z-10 w-full max-w-[2000px] mx-auto px-4 md:px-8 lg:pl-32 lg:pr-12 mt-10 pb-24" in:fade={{ delay: 250, duration: 400 }}>
            <div class="flex flex-col xl:flex-row gap-8 xl:gap-12 items-start">

                <div class="w-full xl:flex-1 min-w-0 space-y-10">

                    {#if meta?.synopsis}
                        <div class="space-y-1.5">
                            <p
                                    bind:this={detail.synopsisElement}
                                    class="text-muted-foreground leading-relaxed {synopsisExpanded ? '' : 'line-clamp-3'}"
                            >
                                {@html meta.synopsis.replace(/<[^>]*>?/gm, '')}
                            </p>

                            {#if detail.canTruncate || synopsisExpanded}
                                <button
                                        onclick={() => synopsisExpanded = !synopsisExpanded}
                                        class="flex items-center gap-1 text-xs font-bold text-primary/80 hover:text-primary transition-colors"
                                >
                                    {#if synopsisExpanded}
                                        {i18n.t('general.show_less')} <ChevronUp class="w-3 h-3" />
                                    {:else}
                                        {i18n.t('general.show_more')} <ChevronDown class="w-3 h-3" />
                                    {/if}
                                </button>
                            {/if}
                        </div>
                    {/if}

                    {#if hasCastOrStaff}
                        <CastAndStaff characters={meta?.characters || []} staff={meta?.staff || []} />
                    {/if}

                    {#if hasRelations}
                        <div class="pt-2 border-t border-border/20">
                            <Relations relations={detail.relations} loading={detail.relationsLoading} />
                        </div>
                    {/if}

                </div>

                {#if !isMovie}
                    <div class="w-full xl:w-[540px] 2xl:w-[540px] shrink-0 xl:-mt-64 relative z-20">
                        <div class="xl:sticky xl:top-8 xl:h-[calc(100vh-2rem)]">
                            {#if isAnime}
                                <Episodes
                                        cid={detail.fullContent.content.cid}
                                        epsOrChapters={meta?.epsOrChapters}
                                        contentUnits={detail.fullContent.contentUnits}
                                        duration={meta?.episodeDuration}
                                />
                            {:else}
                                <Chapters
                                        cid={detail.fullContent.content.cid}
                                        contentType={detail.fullContent.content.contentType}
                                />
                            {/if}
                        </div>
                    </div>
                {/if}
            </div>
        </div>

        <TrackerManager bind:open={showTrackerModal} cid={detail.fullContent.content.cid} trackers={detail.fullContent.trackerMappings} metadata={meta} />
        <ExtensionManager bind:open={showExtensionModal} cid={detail.fullContent.content.cid} metadata={meta} isNsfw={isAdultContent} extensions={detail.fullContent.extensionSources} contentType={detail.fullContent.content.contentType} />
    {/if}
</div>