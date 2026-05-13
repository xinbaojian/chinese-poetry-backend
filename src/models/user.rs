use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserInfo {
    pub id: u64,
    pub username: String,
    pub role: String,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}
