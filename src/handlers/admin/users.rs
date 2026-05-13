use axum::{
    extract::{Extension, Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

// ---------------------------------------------------------------------------
// Query struct
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<u32>,
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
    let per_page: u32 = 20;
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
