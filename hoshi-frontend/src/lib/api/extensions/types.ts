export type ExtensionType = "anime" | "manga" | "novel" | "booru" | "unknown";

export interface Extension {
    id: string;
    name: string;
    version: string;
    author: string;
    icon?: string | null;
    ext_type: ExtensionType;
}

export interface ExtensionsListResponse {
    extensions: string[];
}

export interface ExtensionSettingsResponse {
    episodeServers?: string[];
    supportsDub?: boolean;
}

export interface ExtensionFiltersResponse {
    filters: Record<string, unknown>;
}