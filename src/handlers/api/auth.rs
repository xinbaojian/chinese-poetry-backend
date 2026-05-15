use axum::{
    extract::{Extension, State},
    Json,
};

use crate::auth;
use crate::errors::{AppError, AppResult};
use crate::models::user::{ChangePasswordRequest, LoginRequest, LoginResponse, RefreshRequest, RegisterRequest, User, UserInfo};
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<LoginResponse>> {
    let username = req.username.trim().to_string();
    let password = req.password.trim().to_string();

    if username.len() < 3 || username.len() > 64 {
        return Err(AppError::Validation("用户名长度需要3-64个字符".to_string()));
    }
    if password.len() < 6 {
        return Err(AppError::Validation("密码长度至少6个字符".to_string()));
    }

    let existing: Option<(u64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::Duplicate("用户名已存在".to_string()));
    }

    let hash = tokio::task::spawn_blocking(move || auth::hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, 'user')")
        .bind(&username)
        .bind(&hash)
        .execute(&state.db)
        .await?;

    let user_id = result.last_insert_id();
    let user_info = UserInfo {
        id: user_id,
        username,
        role: "user".to_string(),
    };

    let token = auth::create_jwt(
        &user_info,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_exp_hours,
    )?;

    let refresh_token = auth::generate_refresh_token();
    auth::save_refresh_token(&state.db, user_id, &refresh_token, state.config.auth.refresh_exp_days).await?;

    Ok(Json(LoginResponse { token, refresh_token, user: user_info }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&req.username)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::InvalidCredentials)?;

    let password = req.password.clone();
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || auth::verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let user_info = UserInfo::from(&user);
    let token = auth::create_jwt(
        &user_info,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_exp_hours,
    )?;

    let refresh_token = auth::generate_refresh_token();
    auth::save_refresh_token(&state.db, user_info.id, &refresh_token, state.config.auth.refresh_exp_days).await?;

    Ok(Json(LoginResponse { token, refresh_token, user: user_info }))
}

pub async fn change_password(
    Extension(user_info): Extension<UserInfo>,
    State(state): State<AppState>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if req.new_password.len() < 6 {
        return Err(AppError::Validation("新密码长度至少6个字符".to_string()));
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user_info.id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::Internal("用户不存在".to_string()))?;

    let password = req.old_password.clone();
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || auth::verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let new_password = req.new_password.clone();
    let new_hash = tokio::task::spawn_blocking(move || auth::hash_password(&new_password))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(user_info.id)
        .execute(&state.db)
        .await?;

    auth::revoke_all_refresh_tokens(&state.db, user_info.id).await?;

    Ok(Json(serde_json::json!({ "message": "密码修改成功" })))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user_id = auth::consume_refresh_token(&state.db, &req.refresh_token).await?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::InvalidToken)?;

    let user_info = UserInfo::from(&user);
    let token = auth::create_jwt(
        &user_info,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_exp_hours,
    )?;

    let refresh_token = auth::generate_refresh_token();
    auth::save_refresh_token(&state.db, user_id, &refresh_token, state.config.auth.refresh_exp_days).await?;

    Ok(Json(LoginResponse { token, refresh_token, user: user_info }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<serde_json::Value>> {
    auth::revoke_refresh_token(&state.db, &req.refresh_token).await?;
    Ok(Json(serde_json::json!({ "message": "已退出登录" })))
}
