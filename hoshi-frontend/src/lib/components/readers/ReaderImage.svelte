<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { MangaReaderState } from "@/app/manga.svelte";

    interface Props {
        imgEntry: { id: string; url: string };
        readerState: MangaReaderState;
        customClass?: string;
        customStyle?: string;
    }

    let { imgEntry, readerState, customClass = '', customStyle = '' }: Props = $props();

    const status = $derived(readerState.imageStatus[imgEntry.id] || "loading");
</script>

<div class="relative flex items-center justify-center {customClass}" style={customStyle}>
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
            class="transition-all duration-500 {status === 'loaded' ? 'opacity-100 scale-100' : 'opacity-0 scale-95'} {customClass}"
            style={customStyle}
            onload={() => readerState.setImgStatus(imgEntry.id, "loaded")}
            onerror={() => readerState.setImgStatus(imgEntry.id, "error")}
            use:readerState.resolveBlobSrc={imgEntry}
            use:readerState.handleImgMount={imgEntry.id}
    />
</div>