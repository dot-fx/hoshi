export type ExtensionType = "anime" | "manga" | "novel" | "booru" | "unknown";

export interface Extension {
    id: string;
    name: string;
    version: string;
    author: string;
    icon?: string | null;
    extType: ExtensionType;
}

export interface ExtensionSettingsResponse {
    episodeServers?: string[];
    supportsDub?: boolean;
}

export interface ExtensionFiltersResponse {
    filters: Record<string, unknown>;
}