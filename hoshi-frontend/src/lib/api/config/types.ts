// ── Shared ────────────────────────────────────────────────────────────────────

export type AppTheme = 'system' | 'light' | 'dark' | 'oled';
export type HomeSection = 'anime' | 'manga' | 'novel';
export type MetadataProvider = 'anilist' | 'myanimelist' | 'kitsu';
export type EpisodeLayout = 'grid' | 'list';

// ── Manga ─────────────────────────────────────────────────────────────────────

export type MangaLayout = 'scroll' | 'paged';
export type ReadingDirection = 'ltr' | 'rtl';
export type FitMode = 'width' | 'height';

// ── Novel ─────────────────────────────────────────────────────────────────────

export type NovelTheme = 'light' | 'dark' | 'sepia' | 'oled';
export type FontFamily = 'sans' | 'serif' | 'mono';
export type TextAlign = 'left' | 'justify';

// ── Config sections ───────────────────────────────────────────────────────────

// Appearance and content safety — applies globally
export interface GeneralConfig {
    theme: AppTheme;
    accentColor: string;
    language: string;
    showAdultContent: boolean;
    blurAdultContent: boolean;
}

// Interface layout and behaviour
export interface UiConfig {
    sidebarCollapsed: boolean;
    disableCardTrailers: boolean;
    defaultHomeSection: HomeSection;
}

// Metadata and progress behaviour across all content types
export interface ContentConfig {
    preferredMetadataProvider: MetadataProvider;
    autoUpdateProgress: boolean;
}

// Notification preferences
export interface NotificationsConfig {
    enabled: boolean;
    notifyNewEpisodes: boolean;
    notifyStatusChanges: boolean;
}

// Extension infrastructure
export interface ExtensionsConfig {
    repoUrl: string;
}

// Video playback preferences (anime)
export interface PlayerConfig {
    autoplayNextEpisode: boolean;
    preferredSubLang: string;
    preferredDubLang: string;
    autoSkipIntro: boolean;
    autoSkipOutro: boolean;
    seekStep: number;
    resumeFromLastPos: boolean;
    defaultEpisodeLayout: EpisodeLayout;
}

export interface MangaConfig {
    layout: MangaLayout;
    direction: ReadingDirection;
    pagesPerView: number;
    fitMode: FitMode;
    gapX: number;
    gapY: number;
    preloadPages: number;
}

export interface NovelConfig {
    theme: NovelTheme;
    fontFamily: FontFamily;
    fontSize: number;
    lineHeight: number;
    maxWidth: number;
    textAlign: TextAlign;
    paragraphSpacing: number;
}

export interface AppConfig {
    general: GeneralConfig;
    ui: UiConfig;
    content: ContentConfig;
    notifications: NotificationsConfig;
    extensions: ExtensionsConfig;
    player: PlayerConfig;
    manga: MangaConfig;
    novel: NovelConfig;
}

export const DEFAULT_CONFIG: AppConfig = {
    general: {
        theme: 'system',
        accentColor: '#6366f1',
        language: 'en',
        showAdultContent: false,
        blurAdultContent: true,
    },
    ui: {
        sidebarCollapsed: false,
        disableCardTrailers: false,
        defaultHomeSection: 'anime',
    },
    content: {
        preferredMetadataProvider: 'anilist',
        autoUpdateProgress: true,
    },
    notifications: {
        enabled: true,
        notifyNewEpisodes: true,
        notifyStatusChanges: true,
    },
    extensions: {
        repoUrl: '',
    },
    player: {
        autoplayNextEpisode: true,
        preferredSubLang: 'en',
        preferredDubLang: 'en',
        autoSkipIntro: false,
        autoSkipOutro: false,
        seekStep: 10,
        resumeFromLastPos: true,
        defaultEpisodeLayout: 'grid',
    },
    manga: {
        layout: 'scroll',
        direction: 'rtl',
        pagesPerView: 1,
        fitMode: 'width',
        gapX: 0,
        gapY: 8,
        preloadPages: 3,
    },
    novel: {
        theme: 'light',
        fontFamily: 'sans',
        fontSize: 16,
        lineHeight: 1.6,
        maxWidth: 700,
        textAlign: 'left',
        paragraphSpacing: 2.0,
    },
};