use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub user_id: i32,
    pub password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user: UserInfo,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
}