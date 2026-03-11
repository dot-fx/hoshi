// ── General ───────────────────────────────────────────────────────────────────

export type AppTheme = 'system' | 'light' | 'dark' | 'sepia' | 'oled';
export type FontSize = 'small' | 'medium' | 'large';
export type TrackerService = 'anilist' | 'myanimelist' | 'kitsu' | 'none';

// ── Anime / Player ────────────────────────────────────────────────────────────

export type AudioLanguage = 'ja' | 'en' | 'es';
export type SubLanguage = 'en' | 'es' | 'none';
export type SeekStep = 5 | 10 | 15 | 30;
export type MetadataProvider = 'anilist' | 'myanimelist' | 'kitsu';
export type EpisodeLayout = 'grid' | 'list';

// ── Manga ─────────────────────────────────────────────────────────────────────

export type MangaLayout = 'scroll' | 'paged';
export type ReadingDirection = 'ltr' | 'rtl';
export type FitMode = 'width' | 'height';
export type ChapterLayout = 'grid' | 'list';

// ── Novel ─────────────────────────────────────────────────────────────────────

export type NovelTheme = 'light' | 'dark' | 'sepia' | 'oled';
export type FontFamily = 'sans' | 'serif' | 'mono';
export type TextAlign = 'left' | 'justify';

// ── Config sections ───────────────────────────────────────────────────────────

export interface GeneralConfig {
    showAdultContent: boolean;
    blurAdultContent: boolean;
    theme: AppTheme;
    accentColor: string;           // hex, e.g. "#6366f1"
    sidebarCollapsed: boolean;
    disableCardTrailers: boolean;
    autoUpdateProgress: boolean;
    defaultTrackingService: TrackerService;
    notificationsEnabled: boolean;
}

export interface AnimeConfig {
    autoplayNextEpisode: boolean;
    preferredMetadataProvider: MetadataProvider;
    preferredSubLang: SubLanguage;
    preferredDubLang: AudioLanguage;
    autoSkipIntro: boolean;
    autoSkipOutro: boolean;
    seekStep: SeekStep;
    resumeFromLastPos: boolean;
    extensionRepoUrl: string;
    defaultEpisodeLayout: EpisodeLayout;
    notifyNewEpisodes: boolean;
}

export interface MangaConfig {
    layout: MangaLayout;
    direction: ReadingDirection;
    pagesPerView: 1 | 2;
    fitMode: FitMode;
    gapX: number;
    gapY: number;
    preloadPages: number;
    defaultChapterLayout: ChapterLayout;
    notifyNewChapters: boolean;
}

export interface NovelConfig {
    theme: NovelTheme;
    fontFamily: FontFamily;
    fontSize: number;
    lineHeight: number;
    maxWidth: number;
    textAlign: TextAlign;
}

export interface AppConfig {
    general: GeneralConfig;
    anime: AnimeConfig;
    manga: MangaConfig;
    novel: NovelConfig;
}


export const DEFAULT_CONFIG: AppConfig = {
    general: {
        showAdultContent: false,
        blurAdultContent: true,
        theme: 'system',
        accentColor: '#6366f1',
        sidebarCollapsed: false,
        disableCardTrailers: false,
        autoUpdateProgress: true,
        defaultTrackingService: 'anilist',
        notificationsEnabled: true,
    },
    anime: {
        autoplayNextEpisode: true,
        preferredMetadataProvider: 'anilist',
        preferredSubLang: 'en',
        preferredDubLang: 'ja',
        autoSkipIntro: false,
        autoSkipOutro: false,
        seekStep: 10,
        resumeFromLastPos: true,
        extensionRepoUrl: '',
        defaultEpisodeLayout: 'grid',
        notifyNewEpisodes: true,
    },
    manga: {
        layout: 'scroll',
        direction: 'rtl',
        pagesPerView: 1,
        fitMode: 'width',
        gapX: 0,
        gapY: 8,
        preloadPages: 3,
        defaultChapterLayout: 'grid',
        notifyNewChapters: true,
    },
    novel: {
        theme: 'light',
        fontFamily: 'sans',
        fontSize: 16,
        lineHeight: 1.6,
        maxWidth: 700,
        textAlign: 'left',
    },
};