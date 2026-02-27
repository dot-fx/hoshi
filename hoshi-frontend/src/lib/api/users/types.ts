export interface UserPublic {
    id: number;
    username: string;
    avatar?: string | null;
}

export interface UserPrivate {
    id: number;
    username: string;
    avatar?: string | null;
    hasPassword: boolean;
}

export interface UserResponse {
    id: number;
    username: string;
    avatar?: string | null;
    hasPassword: boolean;
}

export interface UpdateUserBody {
    username?: string;
    password?: string;
}

export interface ChangePasswordBody {
    currentPassword?: string;
    newPassword?: string;
}

export interface DeleteUserBody {
    password?: string;
}

export interface UsersListResponse {
    users: UserResponse[];
}

export interface SingleUserResponse {
    user: UserPublic;
}

export interface SuccessResponse {
    success: boolean;
}

export interface MessageResponse {
    success: boolean;
    message: string;
}