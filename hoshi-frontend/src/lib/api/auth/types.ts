export interface LoginRequest {
    userId: number;
    password?: string;
}

export interface RegisterRequest {
    username: string;
    password?: string;
}

export interface UserInfo {
    id: number;
    username: string;
    avatar?: string | null;
    hasPassword?: boolean;
}

export interface AuthResponse {
    user: UserInfo;
    sessionId?: string;
}