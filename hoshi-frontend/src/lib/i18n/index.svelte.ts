import { browser } from '$app/environment';
import en from './locales/en';
import es from './locales/es';

const dictionaries = { en, es };

type Language = keyof typeof dictionaries;
type TranslationKey = keyof typeof en;

class I18n {
    locale = $state<Language>('en');

    constructor() {
        if (browser) {
            const saved = localStorage.getItem('app_lang') as Language;
            if (saved && dictionaries[saved]) {
                this.locale = saved;
            } else {
                this.locale = navigator.language.startsWith('es') ? 'es' : 'en';
            }
        }
    }

    setLocale(lang: Language) {
        this.locale = lang;
        if (browser) localStorage.setItem('app_lang', lang);
    }

    t(key: TranslationKey): string {
        const translation = dictionaries[this.locale][key];
        if (translation) return translation;
        if (this.locale !== 'en') {
            const fallback = dictionaries['en'][key];
            if (fallback) return fallback;
        }
        return key;
    }
}

export const i18n = new I18n();