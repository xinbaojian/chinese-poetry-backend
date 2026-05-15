use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::rand_core::OsRng;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::user::{JwtClaims, UserInfo};
use crate::state::AppState;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn create_jwt(user: &UserInfo, secret: &str, exp_hours: u64) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = JwtClaims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        iat: now.timestamp() as usize,
        exp: (now.timestamp() + (exp_hours as i64 * 3600)) as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<JwtClaims, AppError> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::InvalidToken)?;
    Ok(token_data.claims)
}

/// Admin API auth middleware — Bearer JWT + role=admin check
pub async fn admin_api_auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::InvalidToken)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InvalidToken);
    }

    let token = &auth_header[7..];
    let claims = decode_jwt(token, &state.config.auth.jwt_secret)?;

    if claims.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    let user_id: u64 = claims.sub.parse().map_err(|_| AppError::InvalidToken)?;
    let user_info = UserInfo {
        id: user_id,
        username: claims.username.clone(),
        role: claims.role,
    };

    request.extensions_mut().insert(user_info);
    Ok(next.run(request).await)
}

/// App API JWT auth middleware (existing)
pub async fn api_auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::InvalidToken)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InvalidToken);
    }

    let token = &auth_header[7..];
    let claims = decode_jwt(token, &state.config.auth.jwt_secret)?;

    let user_id: u64 = claims.sub.parse().map_err(|_| AppError::InvalidToken)?;
    let user_info = UserInfo {
        id: user_id,
        username: claims.username.clone(),
        role: claims.role.clone(),
    };

    request.extensions_mut().insert(user_info);
    Ok(next.run(request).await)
}

pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

pub async fn save_refresh_token(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    token: &str,
    exp_days: u64,
) -> Result<(), AppError> {
    let expires_at = Utc::now() + chrono::Duration::days(exp_days as i64);
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)"
    )
    .bind(user_id)
    .bind(token)
    .bind(expires_at.naive_utc())
    .execute(pool)
    .await?;
    Ok(())
}

/// 原子操作：验证并消费 refresh_token（事务 + FOR UPDATE 行锁防并发）
pub async fn consume_refresh_token(
    pool: &sqlx::MySqlPool,
    token: &str,
) -> Result<u64, AppError> {
    let mut tx = pool.begin().await?;

    let row: Option<(u64,)> = sqlx::query_as(
        "SELECT user_id FROM refresh_tokens WHERE token = ? AND expires_at > NOW() FOR UPDATE"
    )
    .bind(token)
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = row.map(|(id,)| id).ok_or(AppError::InvalidToken)?;

    sqlx::query("DELETE FROM refresh_tokens WHERE token = ?")
        .bind(token)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(user_id)
}

/// 吊销单个 refresh_token（用于 logout）
pub async fn revoke_refresh_token(
    pool: &sqlx::MySqlPool,
    token: &str,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_tokens WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_all_refresh_tokens(
    pool: &sqlx::MySqlPool,
    user_id: u64,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_expired_refresh_tokens(pool: &sqlx::MySqlPool) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
        .execute(pool)
        .await?;
    if result.rows_affected() > 0 {
        tracing::info!("Cleaned up {} expired refresh tokens", result.rows_affected());
    }
    Ok(())
}
