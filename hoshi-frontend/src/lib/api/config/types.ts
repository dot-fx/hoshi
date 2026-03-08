export type AppLanguage = 'en' | 'es';
export type AppTheme = 'system' | 'light' | 'dark';
export type AccentColor = 'default' | 'blue' | 'purple' | 'green';
export type FontSize = 'small' | 'medium' | 'large';
export type TrackerService = 'anilist' | 'myanimelist' | 'kitsu' | 'none';
export type AudioLanguage = 'ja' | 'en' | 'es';
export type SubLanguage = 'en' | 'es' | 'none';
export type SeekStepOption = '5' | '10' | '15' | '30';

export interface AppConfig {
    // General
    appLanguage: AppLanguage;
    showAdultContent: boolean;
    blurAdultContent: boolean;

    // Interface
    theme: AppTheme;
    accentColor: AccentColor;
    reduceAnimations: boolean;
    uiFontSize: FontSize;
    sidebarCollapsed: boolean;

    // Player
    playerAutoNext: boolean;
    prefSubLang: SubLanguage;
    prefAudioLang: AudioLanguage;
    skipIntroAuto: boolean;
    skipOutroAuto: boolean;
    seekStep: SeekStepOption;
    resumeLastPos: boolean;

    // Tracking
    preferredTracker: TrackerService;
    autoUpdateProgress: boolean;
    syncTrackerStartup: boolean;

    // Extensions
    autoUpdateExt: boolean;
    extensionRepoUrls: string;
}