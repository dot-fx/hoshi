import { browser } from '$app/environment';
import en from './locales/en';
import es from './locales/es';

const dictionaries = { en, es };

type Language = keyof typeof dictionaries;
type TranslationKey = keyof typeof en;

class I18n {
    locale = $state<Language>('en');

    constructor() {

    }

    setLocale(lang: Language) {
        this.locale = lang;
        if (browser) localStorage.setItem('app_lang', lang);
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