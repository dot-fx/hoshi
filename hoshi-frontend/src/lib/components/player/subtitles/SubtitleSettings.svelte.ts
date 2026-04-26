const STORE_KEY = 'player_subtitle_settings';

export interface SubtitleSettingsData {
    fontFamily:   string;
    fontSize:     number;
    fontWeight:   'normal' | 'bold';
    italic:       boolean;

    color:        string;

    outlineStyle: 'none' | 'outline' | 'drop-shadow' | 'raised' | 'depressed';
    outlineColor: string;
    outlineWidth: number;

    bgColor:      string;
    bgPadding:    number;
    bgRadius:     number;

    positionY:    number;
    maxWidth:     number;
    textAlign:    'left' | 'center' | 'right';

    lineHeight:   number;
    letterSpacing: number;
    opacity:      number;
}

export const SUBTITLE_FONTS = [
    { label: 'System default',     value: 'system-ui, sans-serif' },
    { label: 'Arial',              value: 'Arial, sans-serif' },
    { label: 'Trebuchet MS',       value: '"Trebuchet MS", sans-serif' },
    { label: 'Georgia',            value: 'Georgia, serif' },
    { label: 'Courier New',        value: '"Courier New", monospace' },
    { label: 'Impact',             value: 'Impact, fantasy' },
    { label: 'Netflix Sans-style', value: '"Netflix Sans", "Helvetica Neue", sans-serif' },
] as const;

const DEFAULTS: SubtitleSettingsData = {
    fontFamily:    'system-ui, sans-serif',
    fontSize:      2,
    fontWeight:    'normal',
    italic:        false,

    color:         '#ffffff',

    outlineStyle:  'outline',
    outlineColor:  '#000000',
    outlineWidth:  3,

    bgColor:       'rgba(0,0,0,0)',
    bgPadding:     6,
    bgRadius:      4,

    positionY:     90,
    maxWidth:      80,
    textAlign:     'center',

    lineHeight:    1.4,
    letterSpacing: 0,
    opacity:       1,
};

async function loadSaved(): Promise<Partial<SubtitleSettingsData> | null> {
    try {
        const { load } = await import('@tauri-apps/plugin-store');
        const store = await load('settings.json', { autoSave: true, defaults: {} });
        const value = await store.get<Partial<SubtitleSettingsData>>(STORE_KEY);
        return value ?? null;
    } catch {
        return null;
    }
}

async function persist(data: SubtitleSettingsData): Promise<void> {
    try {
        const { load } = await import('@tauri-apps/plugin-store');
        const store = await load('settings.json', { autoSave: true, defaults: {} });
        await store.set(STORE_KEY, data);
    } catch { /* ignore */ }
}

export class SubtitleSettings {
    fontFamily    = $state(DEFAULTS.fontFamily);
    fontSize      = $state(DEFAULTS.fontSize);
    fontWeight    = $state(DEFAULTS.fontWeight);
    italic        = $state(DEFAULTS.italic);

    color         = $state(DEFAULTS.color);

    outlineStyle  = $state(DEFAULTS.outlineStyle);
    outlineColor  = $state(DEFAULTS.outlineColor);
    outlineWidth  = $state(DEFAULTS.outlineWidth);

    bgColor       = $state(DEFAULTS.bgColor);
    bgPadding     = $state(DEFAULTS.bgPadding);
    bgRadius      = $state(DEFAULTS.bgRadius);

    positionY     = $state(DEFAULTS.positionY);
    maxWidth      = $state(DEFAULTS.maxWidth);
    textAlign     = $state(DEFAULTS.textAlign);

    lineHeight    = $state(DEFAULTS.lineHeight);
    letterSpacing = $state(DEFAULTS.letterSpacing);
    opacity       = $state(DEFAULTS.opacity);

    constructor() {
        this.init();
    }

    async init() {
        const saved = await loadSaved();
        if (!saved) return;
        for (const [k, v] of Object.entries(saved)) {
            if (k in DEFAULTS) (this as any)[k] = v;
        }
    }

    async save() {
        await persist(this.#snapshot());
    }

    async reset() {
        for (const [k, v] of Object.entries(DEFAULTS)) {
            (this as any)[k] = v;
        }
        await this.save();
    }

    #snapshot(): SubtitleSettingsData {
        return {
            fontFamily:    this.fontFamily,
            fontSize:      this.fontSize,
            fontWeight:    this.fontWeight,
            italic:        this.italic,
            color:         this.color,
            outlineStyle:  this.outlineStyle,
            outlineColor:  this.outlineColor,
            outlineWidth:  this.outlineWidth,
            bgColor:       this.bgColor,
            bgPadding:     this.bgPadding,
            bgRadius:      this.bgRadius,
            positionY:     this.positionY,
            maxWidth:      this.maxWidth,
            textAlign:     this.textAlign,
            lineHeight:    this.lineHeight,
            letterSpacing: this.letterSpacing,
            opacity:       this.opacity,
        };
    }

    get textShadow(): string {
        const c = this.outlineColor;
        const w = this.outlineWidth;
        switch (this.outlineStyle) {
            case 'none':        return 'none';
            case 'drop-shadow': return `${w}px ${w}px ${w * 1.5}px ${c}`;
            case 'raised':      return `0 -${w}px 0 ${c}, 0 -${w}px ${w}px rgba(0,0,0,.5)`;
            case 'depressed':   return `0 ${w}px 0 ${c}, 0 ${w}px ${w}px rgba(0,0,0,.5)`;
            case 'outline': {
                const d = Math.round(w * 0.707);
                return [
                    `${w}px 0 0 ${c}`,
                    `-${w}px 0 0 ${c}`,
                    `0 ${w}px 0 ${c}`,
                    `0 -${w}px 0 ${c}`,
                    `${d}px ${d}px 0 ${c}`,
                    `-${d}px ${d}px 0 ${c}`,
                    `${d}px -${d}px 0 ${c}`,
                    `-${d}px -${d}px 0 ${c}`,
                ].join(', ');
            }
            default: return 'none';
        }
    }

    get wrapperStyle(): string {
        const fromBottom = 100 - this.positionY;
        return [
            `bottom: ${fromBottom}%`,
            `max-width: ${this.maxWidth}%`,
            `opacity: ${this.opacity}`,
            `text-align: ${this.textAlign}`,
        ].join('; ');
    }

    get containerStyle(): string {
        return [
            `font-family: ${this.fontFamily}`,
            `font-size: ${this.fontSize}em`,
            `font-weight: ${this.fontWeight}`,
            `font-style: ${this.italic ? 'italic' : 'normal'}`,
            `color: ${this.color}`,
            `text-shadow: ${this.textShadow}`,
            `background-color: ${this.bgColor}`,
            `padding: ${this.bgPadding / 2}px ${this.bgPadding}px`,
            `border-radius: ${this.bgRadius}px`,
            `line-height: ${this.lineHeight}`,
            `letter-spacing: ${this.letterSpacing}em`,
            `display: inline`,
            `-webkit-box-decoration-break: clone`,
            `box-decoration-break: clone`,
        ].join('; ');
    }
}