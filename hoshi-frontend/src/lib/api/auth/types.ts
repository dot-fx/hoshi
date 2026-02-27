export interface LoginRequest {
    userId: number;
    password?: string;
}

export interface RegisterRequest {
    username: string;
    profilePictureUrl?: string;
    password?: string;
}

export interface UserInfo {
    id: number;
    username: string;
    avatar?: string | null;
}

export interface AuthResponse {
    success: boolean;
    user: UserInfo;
}

export interface SuccessResponse {
    success: boolean;
}