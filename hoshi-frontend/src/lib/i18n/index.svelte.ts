import en from './locales/en';
import es from './locales/es';

const dictionaries = { en, es };

type Language = keyof typeof dictionaries;

type NestedKeyOf<ObjectType extends object> = {
    [Key in keyof ObjectType & (string | number)]: ObjectType[Key] extends object
        ? `${Key}.${NestedKeyOf<ObjectType[Key]>}`
        : `${Key}`;
}[keyof ObjectType & (string | number)];

type TranslationKey = NestedKeyOf<typeof en>;

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

    t(key: TranslationKey | (string & {}), params?: Record<string, string | number>): string {
        const keys = (key as string).split('.');

        let value: any = dictionaries[this.locale];
        let fallback: any = dictionaries.en;

        for (const k of keys) {
            value = value?.[k];
            fallback = fallback?.[k];
        }

        let result = (typeof value === 'string' && value)
            ? value
            : (typeof fallback === 'string' && fallback)
                ? fallback
                : (key as string);

        if (params) {
            for (const [paramKey, paramValue] of Object.entries(params)) {
                result = result.replace(new RegExp(`{{${paramKey}}}`, 'g'), String(paramValue));
            }
        }

        return result;
    }
}

export const i18n = new I18n();