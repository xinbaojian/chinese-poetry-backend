use axum::{
    extract::{Extension, State},
    Json,
};

use crate::auth;
use crate::errors::{AppError, AppResult};
use crate::models::user::{ChangePasswordRequest, LoginRequest, LoginResponse, RegisterRequest, User, UserInfo};
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<LoginResponse>> {
    let username = req.username.trim();
    let password = req.password.trim();

    if username.len() < 3 || username.len() > 64 {
        return Err(AppError::Validation("用户名长度需要3-64个字符".to_string()));
    }
    if password.len() < 6 {
        return Err(AppError::Validation("密码长度至少6个字符".to_string()));
    }

    let existing: Option<(u64,)> = sqlx::query_as("SELECT id FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::Duplicate("用户名已存在".to_string()));
    }

    let hash = auth::hash_password(password)?;

    let result = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, 'user')")
        .bind(username)
        .bind(&hash)
        .execute(&state.db)
        .await?;

    let user_id = result.last_insert_id();
    let user_info = UserInfo {
        id: user_id,
        username: username.to_string(),
        role: "user".to_string(),
    };

    let token = auth::create_jwt(
        &user_info,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_exp_hours,
    )?;

    Ok(Json(LoginResponse { token, user: user_info }))
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

    let valid = auth::verify_password(&req.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let user_info = UserInfo::from(&user);
    let token = auth::create_jwt(
        &user_info,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_exp_hours,
    )?;

    Ok(Json(LoginResponse { token, user: user_info }))
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

    let valid = auth::verify_password(&req.old_password, &user.password_hash)?;
    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let new_hash = auth::hash_password(&req.new_password)?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(user_info.id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "密码修改成功" })))
}
