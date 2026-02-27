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
    userId: number;
    trackerName: string;
    trackerUserId: string;
    accessToken: string;
    refreshToken?: string | null;
    tokenType: string;
    expiresAt: number;
    syncEnabled: boolean;
    createdAt: number;
    updatedAt: number;
}

export interface SuccessResponse {
    success: boolean;
}

export interface SyncResponse {
    success: boolean;
    synced: number;
    errors: string[];
}