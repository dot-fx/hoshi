export interface TrackerIntegration {
    userId: number;
    trackerName: string;
    trackerUserId: string;
    accessToken: string;
    refreshToken?: string | null;
    tokenType: string;
    expiresAt: string;
    syncEnabled: boolean;
    createdAt: string;
    updatedAt: string;
}

export interface IntegrationsResponse {
    integrations: TrackerIntegration[];
}

export interface SuccessResponse {
    success: boolean;
}

export interface SyncResponse {
    success: boolean;
    synced: number;
    errors: string[];
}