export interface TrackerAuthConfig {
    oauthFlow: "implicit" | "code";
    authUrl: string;
    clientId?: string | null;
    scopes: string[];
}

export interface TrackerInfo {
    name: string;
    displayName: string;
    iconUrl: string;
    supportedTypes: string[];
    auth: TrackerAuthConfig;
    connected: boolean;
    trackerUserId?: string | null;
    syncEnabled?: boolean | null;
}

export interface TrackerIntegration {
    trackerName: string;
    accessToken?: string;
    username?: string;
    password?: string;
    codeVerifier?: string;
}