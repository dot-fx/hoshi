function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
}

async function loadSavedValue(key: string): Promise<string | null> {
    try {
        if (isTauri()) {
            const { load } = await import('@tauri-apps/plugin-store');
            const store = await load('settings.json', { autoSave: true, defaults: {} });
            const value = await store.get<string>(key);
            return value ?? null;
        } else {
            return localStorage.getItem(key);
        }
    } catch {
        return null;
    }
}

async function persistValue(key: string, value: string | null): Promise<void> {
    try {
        if (isTauri()) {
            const { load } = await import('@tauri-apps/plugin-store');
            const store = await load('settings.json', { autoSave: true, defaults: {} });
            if (value) {
                await store.set(key, value);
            } else {
                await store.delete(key);
            }
        } else {
            if (value) {
                localStorage.setItem(key, value);
            } else {
                localStorage.removeItem(key);
            }
        }
    } catch { /* ignore */ }
}

class ThemeManager {
    theme = $state<string>('dark'); // Valor por defecto
    accentColor = $state<string | null>(null);

    constructor() {
        this.init();
    }

    async init() {
        const savedTheme = await loadSavedValue('app_theme');
        const savedAccent = await loadSavedValue('app_accent');

        if (savedTheme) this.theme = savedTheme;
        if (savedAccent) this.accentColor = savedAccent;

        this.applyToDOM();
    }

    async setTheme(newTheme: string) {
        this.theme = newTheme;
        this.applyToDOM();
        await persistValue('app_theme', newTheme);
    }

    async setAccentColor(newColor: string | null) {
        this.accentColor = newColor;
        this.applyToDOM();
        await persistValue('app_accent', newColor);
    }

    applyToDOM() {
        if (typeof document === 'undefined') return;

        const html = document.documentElement;
        const allThemes = ['light', 'dark', 'oled'];

        allThemes.forEach(t => html.classList.remove(t));
        if (this.theme && this.theme !== 'light') {
            html.classList.add(this.theme);
        }

        if (this.accentColor) {
            html.style.setProperty('--primary', this.accentColor);
            html.style.setProperty('--ring', this.accentColor);
        } else {
            html.style.removeProperty('--primary');
            html.style.removeProperty('--ring');
        }
    }
}

export const themeManager = new ThemeManager();