import { scheduleApi } from "@/api/schedule/schedule";
import type { AiringEntry } from "@/api/schedule/types";
import type { CoreError } from "@/api/client";
import { i18n } from "@/stores/i18n.svelte.js";
import type { NormalizedCard } from "@/utils/normalize";

export type ScheduleGroup = {
    key: string;
    header: string;
    isToday: boolean;
    items: Array<{
        card: NormalizedCard;
        episode: number;
        airingAt: number;
        trackerId: string;
    }>;
};

function getMs(ts: number) {
    return ts > 1e11 ? ts : ts * 1000;
}

function toCard(entry: AiringEntry): NormalizedCard {
    return {
        cid:              entry.trackerId,
        titleI18n:        entry.titleI18n ?? {},
        titleDefault:     entry.title ?? "",
        cover:            entry.coverImage ?? "",
        bannerImage:      entry.bannerImage ?? null,
        synopsis:         entry.synopsis?.replace(/<[^>]*>?/gm, "") ?? null,
        score:            entry.rating ? Math.round(entry.rating * (entry.rating <= 10 ? 10 : 1)) : null,
        year:             entry.releaseDate ? entry.releaseDate.split("-")[0] : null,
        nsfw:             entry.nsfw,
        hasAdultGenre:    entry.genres?.some(g => g.toLowerCase() === "hentai" || g.toLowerCase() === "adult") ?? false,
        contentTypeLabel: entry.subtype ?? "TV",
        status:           entry.status ?? null,
        trailerUrlRaw:    entry.trailerUrl ?? null,
        episodeCount:     null,
        contentType:      "anime",
        href:             `/c/anilist/${entry.trackerId}`,
    };
}

function buildGroups(entries: AiringEntry[]): ScheduleGroup[] {
    const todayStr    = new Date().toDateString();
    const tomorrowStr = new Date(Date.now() + 86_400_000).toDateString();

    function getDayLabel(d: Date) {
        const s = d.toDateString();
        if (s === todayStr)    return i18n.t("schedule.today");
        if (s === tomorrowStr) return i18n.t("schedule.tomorrow");
        return d.toLocaleDateString(i18n.locale, { weekday: "long", month: "long", day: "numeric" });
    }

    const groups: Record<string, AiringEntry[]> = {};
    for (const e of entries) {
        const d   = new Date(getMs(e.airingAt));
        const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
        (groups[key] ??= []).push(e);
    }

    return Object.keys(groups).sort().map(key => {
        const sorted = groups[key].sort((a, b) => a.airingAt - b.airingAt);
        const d      = new Date(getMs(sorted[0].airingAt));
        return {
            key,
            header:  getDayLabel(d),
            isToday: d.toDateString() === todayStr,
            items:   sorted.map(e => ({
                card:      toCard(e),
                episode:   e.episode,
                airingAt:  e.airingAt,
                trackerId: e.trackerId,
            })),
        };
    });
}

class ScheduleStore {
    weekEntries  = $state<AiringEntry[]>([]);
    monthEntries = $state<AiringEntry[]>([]);
    isLoading    = $state(false);
    error        = $state<CoreError | null>(null);
    viewMode     = $state<"week" | "month">("week");
    myListOnly   = $state(false);

    get rawEntries(): AiringEntry[] {
        return this.viewMode === "week" ? this.weekEntries : this.monthEntries;
    }

    get entries(): AiringEntry[] {
        if (!this.myListOnly) return this.rawEntries;
        return this.rawEntries.filter(
            e => e.userStatus === "CURRENT" || e.userStatus === "PLANNING"
        );
    }

    get groups(): ScheduleGroup[] {
        return buildGroups(this.entries);
    }

    async load(force = false) {
        if (!force && this.rawEntries.length > 0) return;

        this.isLoading = true;
        this.error = null;

        try {
            const daysAhead = this.viewMode === "week" ? 7 : 30;
            const res = await scheduleApi.get({ daysBack: 0, daysAhead });

            if (this.viewMode === "week") this.weekEntries = res;
            else this.monthEntries = res;
        } catch (err) {
            console.error("Failed to load schedule:", err);
            this.error = err as CoreError;
            if (this.viewMode === "week") this.weekEntries = [];
            else this.monthEntries = [];
        } finally {
            this.isLoading = false;
        }
    }

    toggleMyList() { this.myListOnly = !this.myListOnly; }

    switchView(mode: "week" | "month") {
        this.viewMode = mode;
        this.load();
    }
}

export const scheduleStore = new ScheduleStore();