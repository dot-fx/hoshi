import {
    Us, Es, Jp, Fr, De, It, Ru, Sa, Id, Tr, Cn, Kr, Th, Vn, Pl, Pt, Tw, Ph, In, My
} from "svelte-flag-icons";

const loaders = {
    en: () => import('./locales/en'),
    es: () => import('./locales/es'),
    ja: () => import('./locales/ja'),
    fr: () => import('./locales/fr'),
    de: () => import('./locales/de'),
    it: () => import('./locales/it'),
    ru: () => import('./locales/ru'),
    ar: () => import('./locales/ar'),
    id: () => import('./locales/id'),
    tr: () => import('./locales/tr'),
    'zh-CN': () => import('./locales/zh-CN'),
    'ko-KR': () => import('./locales/ko-KR'),
    th: () => import('./locales/th'),
    vi: () => import('./locales/vi'),
    pl: () => import('./locales/pl'),
    'pt-PT': () => import('./locales/pt-PT'),
    'zh-TW': () => import('./locales/zh-TW'),
    tl: () => import('./locales/tl'),
    ms: () => import('./locales/ms'),
    hi: () => import('./locales/hi')
};

export type Language = keyof typeof loaders;

import type enData from './locales/en';
export type TranslationKey = NestedKeyOf<typeof enData>;

type NestedKeyOf<ObjectType extends object> = {
    [Key in keyof ObjectType & (string | number)]: ObjectType[Key] extends object
        ? `${Key}.${NestedKeyOf<ObjectType[Key]>}`
        : `${Key}`;
}[keyof ObjectType & (string | number)];

function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
}

async function loadSavedLanguage(): Promise<Language> {
    try {
        if (isTauri()) {
            const { load } = await import('@tauri-apps/plugin-store');
            const store = await load('settings.json', { autoSave: true, defaults: {} });
            const saved = await store.get<Language>('app_lang');
            if (saved && saved in loaders) return saved;
        } else {
            const saved = localStorage.getItem('app_lang') as Language;
            if (saved && saved in loaders) return saved;
        }
    } catch { /* ignore */ }
    return 'en';
}

class I18n {
    locale = $state<Language>('en');
    currentData = $state<any>(null);
    fallbackData = $state<any>(null);

    constructor() {
        this.init();
    }

    private async init() {
        const lang = await loadSavedLanguage();
        const [enMod, currentMod] = await Promise.all([
            loaders.en(),
            loaders[lang]()
        ]);

        this.fallbackData = enMod.default;
        this.currentData = currentMod.default;
        this.locale = lang;
    }

    async setLocale(lang: Language) {
        if (!(lang in loaders)) return;

        const mod = await loaders[lang]();
        this.currentData = mod.default;
        this.locale = lang;

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

    t(key: TranslationKey | (string & {}), params?: Record<string, string | number>): string {
        if (!this.currentData) return (key as string);

        const keys = (key as string).split('.');
        let value: any = this.currentData;
        let fallback: any = this.fallbackData;

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

    getAvailableLanguages() {
        return [
            { code: 'en', name: 'English', icon: Us },
            { code: 'es', name: 'Español', icon: Es },
            { code: 'zh-CN', name: '简体中文', icon: Cn },
            { code: 'hi', name: 'हिन्दी', icon: In },
            { code: 'ar', name: 'العربية', icon: Sa },
            { code: 'pt-PT', name: 'Português (Portugal)', icon: Pt },
            { code: 'ru', name: 'Русский', icon: Ru },
            { code: 'ja', name: '日本語', icon: Jp },
            { code: 'de', name: 'Deutsch', icon: De },
            { code: 'fr', name: 'Français', icon: Fr },
            { code: 'it', name: 'Italiano', icon: It },
            { code: 'ko-KR', name: '한국어', icon: Kr },
            { code: 'tr', name: 'Türkçe', icon: Tr },
            { code: 'vi', name: 'Tiếng Việt', icon: Vn },
            { code: 'id', name: 'Bahasa Indonesia', icon: Id },
            { code: 'th', name: 'ไทย', icon: Th },
            { code: 'pl', name: 'Polski', icon: Pl },
            { code: 'ms', name: 'Bahasa Melayu', icon: My },
            { code: 'tl', name: 'Tagalog', icon: Ph },
            { code: 'zh-TW', name: '繁體中文', icon: Tw }
        ];
    }
}

export const i18n = new I18n();