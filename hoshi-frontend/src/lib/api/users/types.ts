export interface UserPublic {
    id: number;
    username: string;
    avatar?: string | null;
}

export interface UserPrivate {
    id: number;
    username: string;
    avatar?: string | null;
    hasPassword?: boolean;
}

export interface UserResponse {
    id: number;
    username: string;
    avatar?: string | null;
    hasPassword: boolean;
}

export interface UsersResponse {
    users: UserResponse[];
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