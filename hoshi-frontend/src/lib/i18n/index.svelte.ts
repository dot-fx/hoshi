import en from './locales/en';
import es from './locales/es';

const dictionaries = { en, es };

type Language = keyof typeof dictionaries;
type TranslationKey = keyof typeof en;

function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
}

async function loadSavedLanguage(): Promise<Language> {
    try {
        if (isTauri()) {
            const { load } = await import('@tauri-apps/plugin-store');
            const store = await load('settings.json', { autoSave: true, defaults: {} });
            const saved = await store.get<Language>('app_lang');
            if (saved && saved in dictionaries) return saved;
        } else {
            const saved = localStorage.getItem('app_lang') as Language;
            if (saved && saved in dictionaries) return saved;
        }
    } catch { /* ignore */ }
    return 'en';
}

async function persistLanguage(lang: Language): Promise<void> {
    try {
        if (isTauri()) {
            const { load } = await import('@tauri-apps/plugin-store');
            const store = await load('settings.json', { autoSave: true, defaults: {} });
            await store.set('app_lang', lang);
        } else {
            localStorage.setItem('app_lang', lang);
        }
    } catch { /* ignore */ }
}

class I18n {
    locale = $state<Language>('en');

    constructor() {
        loadSavedLanguage().then(lang => { this.locale = lang; });
    }

    async setLocale(lang: Language) {
        this.locale = lang;
        await persistLanguage(lang);
    }

    t(key: TranslationKey): string {
        return (
            dictionaries[this.locale][key] ??
            dictionaries.en[key] ??
            key
        );
    }
}

export const i18n = new I18n();