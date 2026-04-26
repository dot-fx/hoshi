<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { MangaReaderState } from "@/app/manga.svelte";

    interface Props {
        imgEntry: { id: string; url: string };
        readerState: MangaReaderState;
    }

    let { imgEntry, readerState }: Props = $props();

    const status = $derived(readerState.imageStatus[imgEntry.id] || "loading");

    const isPaged = $derived(readerState.layout === "paged");
    const wrapperStyle = $derived.by(() => {
        if (isPaged) {
            switch (readerState.fitMode) {
                case 'width':  return 'flex: 1 1 0; min-width: 0; height: 100%; max-height: 100%;';
                case 'height': return 'height: 100%; flex-shrink: 0; max-width: 100%;';
                case 'fit':    return 'flex: 1 1 0; min-width: 0; height: 100%; max-height: 100%;';
                default:       return 'flex: 1 1 0; min-width: 0; height: 100%; max-height: 100%;';
            }
        } else {
            // scroll mode
            switch (readerState.fitMode) {
                case 'width':  return 'width: 100%; flex-shrink: 1; flex-grow: 1; min-width: 0;';
                case 'height': return 'height: 100%; flex-shrink: 0; max-width: 100%;';
                case 'fit':    return 'flex: 1 1 0; min-width: 0;';
                default:       return 'width: 100%; flex-shrink: 1; flex-grow: 1; min-width: 0;';
            }
        }
    });

    const imgStyle = $derived.by(() => {
        if (isPaged) {
            switch (readerState.fitMode) {
                case 'width':
                    return 'width: 100%; height: auto; max-height: 100%; object-fit: contain; display: block;';
                case 'height':
                    return 'height: 100%; width: auto; max-width: 100%; object-fit: contain; display: block;';
                case 'fit':
                    return 'max-width: 100%; max-height: 100%; width: auto; height: auto; object-fit: contain; display: block;';
                default:
                    return 'width: 100%; height: auto; max-height: 100%; object-fit: contain; display: block;';
            }
        } else {
            // scroll mode
            switch (readerState.fitMode) {
                case 'width':
                    return 'width: 100%; height: auto; display: block;';
                case 'height':
                    return 'height: 100%; width: auto; max-width: 100%; display: block;';
                case 'fit':
                    return 'max-width: 100%; max-height: 90vh; width: auto; height: auto; object-fit: contain; display: block;';
                default:
                    return 'width: 100%; height: auto; display: block;';
            }
        }
    });
</script>

<div class="relative" style={wrapperStyle}>
    {#if status === "loading" || status === "error"}
        <div class="absolute inset-0 flex flex-col items-center justify-center bg-muted/10 animate-pulse rounded-lg">
            {#if status === "loading"}
                <div class="size-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin"></div>
            {:else}
                <div class="flex flex-col items-center gap-2 text-muted-foreground/50">
                    <span class="text-[10px] font-black uppercase tracking-tighter">{i18n.t("reader.error_loading")}</span>
                </div>
            {/if}
        </div>
    {/if}

    <img
            src={imgEntry.url}
            alt={i18n.t("reader.page_alt")}
            draggable="false"
            loading="lazy"
            class="transition-opacity duration-500 {status === 'loaded' ? 'opacity-100' : 'opacity-0'}"
            style={imgStyle}
            onload={() => readerState.setImgStatus(imgEntry.id, "loaded")}
            onerror={() => readerState.setImgStatus(imgEntry.id, "error")}
            use:readerState.resolveBlobSrc={imgEntry}
            use:readerState.handleImgMount={imgEntry.id}
    />
</div>