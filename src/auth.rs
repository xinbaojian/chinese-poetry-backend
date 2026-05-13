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
