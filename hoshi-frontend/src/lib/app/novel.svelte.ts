import { appConfig } from "@/stores/config.svelte";
import { progressApi } from "@/api/progress/progress";
import { contentApi } from "@/api/content/content";
import type { NovelConfig } from "@/api/config/types";
import type { CoreError } from "@/api/client";
import { untrack } from "svelte";

import { BaseReaderState } from "./reader.svelte";

export const NOVEL_THEMES = {
    light: { bg: "#fdfdfd", text: "#1a1a1a", border: "#e5e7eb" },
    dark:  { bg: "#1a1a1a", text: "#e0e0e0", border: "#262626" },
    sepia: { bg: "#f4ecd8", text: "#5b4636", border: "#e2d7bf" },
    oled:  { bg: "#000000", text: "#d1d5db", border: "#171717" },
} as const;

export class NovelReaderState extends BaseReaderState {
    contentHtml = $state("");

    private novelConfig = $derived(appConfig.data?.novel);
    theme        = $derived(this.novelConfig?.theme      ?? "dark");
    fontFamily   = $derived(this.novelConfig?.fontFamily ?? "sans");
    textAlign    = $derived(this.novelConfig?.textAlign  ?? "left");

    // Initialize immediately from config so there's no flash of defaults
    fontSize         = $state(appConfig.data?.novel?.fontSize         ?? 18);
    lineHeight       = $state(appConfig.data?.novel?.lineHeight       ?? 1.6);
    maxWidth         = $state(appConfig.data?.novel?.maxWidth         ?? 800);
    paragraphSpacing = $state(appConfig.data?.novel?.paragraphSpacing ?? 1.5);

    private debounceTimer: ReturnType<typeof setTimeout> | undefined;

    constructor() {
        super();

        // Config → sliders: runs when novelConfig loads/changes externally.
        // Uses untrack so assigning sliders here doesn't trigger the save effect.
        $effect(() => {
            const cfg = this.novelConfig;
            if (!cfg) return;
            untrack(() => {
                this.fontSize         = cfg.fontSize         ?? 18;
                this.lineHeight       = cfg.lineHeight       ?? 1.6;
                this.maxWidth         = cfg.maxWidth         ?? 800;
                this.paragraphSpacing = cfg.paragraphSpacing ?? 1.5;
            });
        });

        // Sliders → config: reads slider $state values as reactive dependencies,
        // but reads novelConfig via untrack so saving doesn't re-trigger this effect.
        $effect(() => {
            const size    = this.fontSize;
            const line    = this.lineHeight;
            const width   = this.maxWidth;
            const spacing = this.paragraphSpacing;

            clearTimeout(this.debounceTimer);
            this.debounceTimer = setTimeout(() => {
                untrack(() => this.updateNovelConfig({
                    fontSize:         size,
                    lineHeight:       line,
                    maxWidth:         width,
                    paragraphSpacing: spacing,
                }));
            }, 500);
        });
    }

    protected async loadChapter(currentCid: string, currentExt: string, currentChapterNum: number) {
        this.isLoading = true;
        this.error = null;

        const container = document.getElementById("novel-main-container");
        if (container) container.scrollTop = 0;

        try {
            const [, playRes] = await Promise.all([
                this.fetchShared(currentCid, currentExt, currentChapterNum),
                contentApi.play(currentCid, currentExt, currentChapterNum),
            ]);

            if (playRes.type.toLowerCase() !== "novel" || !playRes.data) {
                throw { key: "reader.no_data" } as CoreError;
            }

            const data: any = playRes.data;
            this.contentHtml = data.html || data.text || data.content || data;

            if (!this.contentHtml) {
                throw { key: "reader.no_content" } as CoreError;
            }

            progressApi
                .updateChapterProgress({ cid: currentCid, chapter: currentChapterNum, completed: false })
                .catch(e => console.error("History sync failed", e));
        } catch (e: any) {
            this.error = e.key ? e : { key: "errors.unknown_error" };
        } finally {
            this.isLoading = false;
        }
    }

    async updateNovelConfig(patch: Partial<NovelConfig>) {
        if (!appConfig.data?.novel) return;
        try {
            await appConfig.update({ novel: { ...appConfig.data.novel, ...patch } });
        } catch (err) {
            console.error("Error updating novel config:", err);
        }
    }

    onScroll(e: Event) {
        const target = e.currentTarget as HTMLElement;
        if (target.scrollHeight > 0) {
            const ratio = (target.scrollTop + target.clientHeight) / target.scrollHeight;
            this.handleProgress(ratio);
        }
    }

    get themeColors() {
        return NOVEL_THEMES[this.theme as keyof typeof NOVEL_THEMES] ?? NOVEL_THEMES.dark;
    }
}