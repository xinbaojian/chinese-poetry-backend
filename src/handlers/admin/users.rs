use axum::{
    extract::{Extension, Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::auth;
use crate::errors::AppError;
use crate::models::user::UserInfo;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Database row models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserRow {
    pub id: u64,
    pub username: String,
    pub role: String,
    pub created_at: String,
    pub record_count: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserProgressItem {
    pub id: u64,
    pub poem_id: u64,
    pub poem_title: String,
    pub poet_name: String,
    pub mastery_level: String,
    pub review_count: u32,
    pub next_review_date: Option<String>,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Query struct
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserRow>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<UserQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    let users: Vec<UserRow> = sqlx::query_as(
        "SELECT u.id, u.username, u.role, DATE_FORMAT(u.created_at, '%Y-%m-%d %H:%i') as created_at, \
         (SELECT COUNT(*) FROM learning_records WHERE user_id = u.id) as record_count \
         FROM users u ORDER BY u.id DESC LIMIT ? OFFSET ?",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(UserListResponse {
        users,
        total,
        page,
        per_page,
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    if id == current_user.id {
        return Err(AppError::Validation("不能删除当前登录的管理员账号".to_string()));
    }

    // Delete review history via learning_records
    sqlx::query(
        "DELETE rh FROM review_history rh \
         JOIN learning_records lr ON rh.learning_record_id = lr.id \
         WHERE lr.user_id = ?",
    )
    .bind(id)
    .execute(&state.db)
    .await?;

    // Delete learning records
    sqlx::query("DELETE FROM learning_records WHERE user_id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    // Delete user
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "删除成功" })))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<UserInfo>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    if id == current_user.id {
        return Err(AppError::Validation("不能重置当前登录管理员账号的密码".to_string()));
    }

    let exists: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(AppError::NotFound(format!("用户 #{id} 不存在")));
    }

    let default_password = "123456";
    let password = default_password.to_string();
    let hash = tokio::task::spawn_blocking(move || auth::hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&hash)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "密码已重置为默认密码" })))
}

pub async fn get_progress(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let exists: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Err(AppError::NotFound(format!("用户 #{id} 不存在")));
    }

    let records: Vec<UserProgressItem> = sqlx::query_as(
        "SELECT lr.id, lr.poem_id, p.title as poem_title, pt.name as poet_name, \
         lr.mastery_level, lr.review_count, \
         DATE_FORMAT(lr.next_review_date, '%Y-%m-%d %H:%i') as next_review_date, \
         DATE_FORMAT(lr.updated_at, '%Y-%m-%d %H:%i') as updated_at \
         FROM learning_records lr \
         JOIN poems p ON lr.poem_id = p.id \
         JOIN poets pt ON p.poet_id = pt.id \
         WHERE lr.user_id = ? \
         ORDER BY lr.updated_at DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(records))
}
