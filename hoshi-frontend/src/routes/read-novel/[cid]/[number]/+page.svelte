<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { contentApi } from "$lib/api/content/content";
    import type { ContentUnit } from "$lib/api/content/types";

    import { Button } from "$lib/components/ui/button";
    import { Slider } from "$lib/components/ui/slider";
    import { Type, AlignLeft, AlignJustify, Palette, Expand } from "lucide-svelte";

    // IMPORTAR EL LAYOUT COMPARTIDO
    import ReaderLayout from "$lib/components/ReaderLayout.svelte";

    const params = $derived(page.params as Record<string, string>);
    const cid = $derived(params.cid);
    const extension = $derived(params.extension);
    const chapterNumber = $derived(Number(params.number));

    let title = $state("");
    let chapterTitle = $state("");
    let contentHtml = $state<string>("");
    let allChapters = $state<ContentUnit[]>([]);

    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let showSettings = $state(false);

    // --- NOVEL CONFIG ---
    let theme = $state<"light" | "dark" | "sepia" | "oled">("dark");
    let fontFamily = $state<"sans" | "serif" | "mono">("sans");
    let textAlign = $state<"left" | "justify">("left");

    let fontSizeArr = $state([18]);
    let fontSize = $derived(fontSizeArr[0]);
    let lineHeightArr = $state([1.6]);
    let lineHeight = $derived(lineHeightArr[0]);
    let maxWidthArr = $state([800]);
    let maxWidth = $derived(maxWidthArr[0]);

    let hasNextChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber + 1));
    let hasPrevChapter = $derived(allChapters.some(c => c.unitNumber === chapterNumber - 1));

    const themes = {
        light: { bg: "#fdfdfd", text: "#1a1a1a" },
        dark: { bg: "#1a1a1a", text: "#e0e0e0" },
        sepia: { bg: "#f4ecd8", text: "#5b4636" },
        oled: { bg: "#000000", text: "#d1d5db" }
    };

    $effect(() => {
        if (!isLoading) {
            localStorage.setItem("hoshi-novel-config", JSON.stringify({
                theme, fontFamily, textAlign, fontSize: fontSizeArr[0], lineHeight: lineHeightArr[0], maxWidth: maxWidthArr[0]
            }));
        }
    });

    onMount(async () => {
        const savedConfig = localStorage.getItem("hoshi-novel-config");
        if (savedConfig) {
            try {
                const parsed = JSON.parse(savedConfig);
                if (parsed.theme) theme = parsed.theme;
                if (parsed.fontFamily) fontFamily = parsed.fontFamily;
                if (parsed.textAlign) textAlign = parsed.textAlign;
                if (parsed.fontSize) fontSizeArr = [parsed.fontSize];
                if (parsed.lineHeight) lineHeightArr = [parsed.lineHeight];
                if (parsed.maxWidth) maxWidthArr = [parsed.maxWidth];
            } catch (e) {}
        }
        await loadChapter();
    });

    async function loadChapter() {
        isLoading = true;
        error = null;

        const mainContainer = document.getElementById("novel-main-container");
        if (mainContainer) mainContainer.scrollTop = 0;

        try {
            const [contentRes, playRes] = await Promise.all([
                contentApi.get(cid || ""),
                contentApi.play(cid || "", extension || "", chapterNumber)
            ]);

            // CORRECCIÓN: contentRes ya es directamente ContentWithMappings (sin .data)
            title = contentRes.title ?? "";
            allChapters = (contentRes.contentUnits ?? []).filter(u => u.contentType === "chapter");

            const currentUnit = allChapters.find(u => u.unitNumber === chapterNumber);
            chapterTitle = currentUnit?.title ? `Ch. ${chapterNumber} - ${currentUnit.title}` : `Chapter ${chapterNumber}`;

            // CORRECCIÓN: Usar playType y verificar la existencia de data según PlayResponse
            if (playRes.type !== "reader" || !playRes.data) {
                throw new Error("No novel data available");
            }

            // CORRECCIÓN: playRes.data contiene la información cruda devuelta por la extensión
            const data: any = playRes.data;
            contentHtml = data.html || data.text || data.content || data;

            if (!contentHtml) throw new Error("Content is empty");
        } catch (e: any) {
            error = e?.message ?? "Failed to load chapter";
        } finally {
            isLoading = false;
        }
    }
</script>

<svelte:head>
    <title>{chapterTitle} — {title}</title>
</svelte:head>

<!-- El resto del HTML queda intacto porque usa las variables de estado reactivas que ya corregimos -->
{#snippet NovelSettings()}
    <div class="space-y-6">
        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2"><Palette class="size-4"/> Theme</span>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={theme === 'light' ? 'secondary' : 'outline'} class="text-sm h-9 bg-[#fdfdfd] text-black hover:bg-[#fdfdfd]/90 hover:text-black" onclick={() => theme = 'light'}>Light</Button>
                <Button variant={theme === 'dark' ? 'secondary' : 'outline'} class="text-sm h-9 bg-[#1a1a1a] text-white hover:bg-[#1a1a1a]/90 hover:text-white" onclick={() => theme = 'dark'}>Dark</Button>
                <Button variant={theme === 'sepia' ? 'secondary' : 'outline'} class="text-sm h-9 bg-[#f4ecd8] text-[#5b4636] hover:bg-[#f4ecd8]/90 hover:text-[#5b4636]" onclick={() => theme = 'sepia'}>Sepia</Button>
                <Button variant={theme === 'oled' ? 'secondary' : 'outline'} class="text-sm h-9 bg-black text-gray-300 hover:bg-black/90 hover:text-gray-300" onclick={() => theme = 'oled'}>OLED</Button>
            </div>
        </div>

        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2"><Type class="size-4"/> Font Family</span>
            <div class="grid grid-cols-3 gap-2">
                <Button variant={fontFamily === 'sans' ? 'secondary' : 'outline'} class="text-sm h-9 font-sans" onclick={() => fontFamily = 'sans'}>Sans</Button>
                <Button variant={fontFamily === 'serif' ? 'secondary' : 'outline'} class="text-sm h-9 font-serif" onclick={() => fontFamily = 'serif'}>Serif</Button>
                <Button variant={fontFamily === 'mono' ? 'secondary' : 'outline'} class="text-sm h-9 font-mono" onclick={() => fontFamily = 'mono'}>Mono</Button>
            </div>
        </div>

        <div class="space-y-3">
            <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-2">Alignment</span>
            <div class="grid grid-cols-2 gap-2">
                <Button variant={textAlign === 'left' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => textAlign = 'left'}><AlignLeft class="size-4 mr-2"/> Left</Button>
                <Button variant={textAlign === 'justify' ? 'secondary' : 'outline'} class="text-sm h-9" onclick={() => textAlign = 'justify'}><AlignJustify class="size-4 mr-2"/> Justify</Button>
            </div>
        </div>

        <div class="space-y-5 pt-2 border-t border-border/40">
            <div>
                <div class="flex items-center justify-between mb-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Font Size</span>
                    <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{fontSize}px</span>
                </div>
                <Slider bind:value={fontSizeArr} min={12} max={32} step={1} class="w-full" />
            </div>

            <div>
                <div class="flex items-center justify-between mb-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground">Line Height</span>
                    <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{lineHeight}</span>
                </div>
                <Slider bind:value={lineHeightArr} min={1} max={3} step={0.1} class="w-full" />
            </div>

            <div>
                <div class="flex items-center justify-between mb-3">
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground flex items-center gap-1"><Expand class="size-3"/> Content Width</span>
                    <span class="text-xs font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/50">{maxWidth}px</span>
                </div>
                <Slider bind:value={maxWidthArr} min={400} max={1200} step={50} class="w-full" />
            </div>
        </div>
    </div>
{/snippet}

<ReaderLayout
        {isLoading}
        {error}
        {title}
        {chapterTitle}
        {cid}
        bind:showSettings
        onRetry={loadChapter}
        settings={NovelSettings}
>
    <main
            id="novel-main-container"
            class="flex-1 overflow-y-auto overflow-x-hidden relative transition-colors duration-300"
            style="background-color: {themes[theme].bg}; color: {themes[theme].text};"
    >
        <article
                class="mx-auto px-4 py-8 md:py-12 transition-all duration-300 {fontFamily === 'sans' ? 'font-sans' : fontFamily === 'serif' ? 'font-serif' : 'font-mono'}"
                style="max-width: {maxWidth}px; font-size: {fontSize}px; line-height: {lineHeight}; text-align: {textAlign};"
        >
            <div class="prose max-w-none novel-content" style="color: inherit; text-align: inherit; line-height: inherit;">
                {@html contentHtml}
            </div>

            <div class="w-full mt-24 mb-12 flex justify-between gap-4 border-t border-opacity-20 pt-8" style="border-color: {themes[theme].text}">
                <Button variant="outline" disabled={!hasPrevChapter} href={`/read/${cid}/${extension}/${chapterNumber - 1}`} class="flex-1 bg-transparent border-current hover:bg-black/5 hover:text-current">Previous Chapter</Button>
                <Button variant="outline" disabled={!hasNextChapter} href={`/read/${cid}/${extension}/${chapterNumber + 1}`} class="flex-1 bg-transparent border-current hover:bg-black/5 hover:text-current">Next Chapter</Button>
            </div>
        </article>
    </main>
</ReaderLayout>

<style>
    :global(.novel-content *) {
        text-align: inherit;
        line-height: inherit;
        color: inherit;
    }
    :global(.novel-content p) {
        margin-bottom: 1.5em;
    }
</style>