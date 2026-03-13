export type HomeSection = 'anime' | 'manga' | 'novel';
export type MetadataProvider = 'anilist' | 'myanimelist' | 'kitsu';
export type EpisodeLayout = 'grid' | 'list';

export type MangaLayout = 'scroll' | 'paged';
export type ReadingDirection = 'ltr' | 'rtl';
export type FitMode = 'width' | 'height';


export type NovelTheme = 'light' | 'dark' | 'sepia' | 'oled';
export type FontFamily = 'sans' | 'serif' | 'mono';
export type TextAlign = 'left' | 'justify';


export interface GeneralConfig {
    language: string;
    showAdultContent: boolean;
    blurAdultContent: boolean;
}

export interface UiConfig {
    sidebarCollapsed: boolean;
    disableCardTrailers: boolean;
    defaultHomeSection: HomeSection;
}

export interface ContentConfig {
    preferredMetadataProvider: MetadataProvider;
    autoUpdateProgress: boolean;
}

export interface NotificationsConfig {
    enabled: boolean;
    notifyNewEpisodes: boolean;
    notifyStatusChanges: boolean;
}

export interface ExtensionsConfig {
    repoUrl: string;
}

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