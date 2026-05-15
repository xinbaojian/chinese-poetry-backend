use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::auth;
use crate::errors::{AppError, AppResult};
use crate::models::user::{LoginResponse, RefreshRequest, User, UserInfo};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(form): Json<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE username = ?",
    )
    .bind(&form.username)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or(AppError::InvalidCredentials)?;

    // Verify password
    let password = form.password.clone();
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || auth::verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    if user.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    let user_info = UserInfo::from(&user);
    let token = auth::create_jwt(&user_info, &state.config.auth.jwt_secret, state.config.auth.jwt_exp_hours)?;

    let refresh_token = auth::generate_refresh_token();
    auth::save_refresh_token(&state.db, user_info.id, &refresh_token, state.config.auth.refresh_exp_days).await?;

    Ok(Json(LoginResponse {
        token,
        refresh_token,
        user: user_info,
    }))
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

    if user.role != "admin" {
        return Err(AppError::Unauthorized);
    }

    let user_info = UserInfo::from(&user);
    let token = auth::create_jwt(&user_info, &state.config.auth.jwt_secret, state.config.auth.jwt_exp_hours)?;

    let refresh_token = auth::generate_refresh_token();
    auth::save_refresh_token(&state.db, user_id, &refresh_token, state.config.auth.refresh_exp_days).await?;

    Ok(Json(LoginResponse { token, refresh_token, user: user_info }))
}
