use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Database row models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PoetRow {
    pub id: u64,
    pub name: String,
    pub dynasty: String,
    pub created_at: NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Query / Request structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PoetQuery {
    pub keyword: Option<String>,
    pub dynasty: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePoetRequest {
    pub name: String,
    pub dynasty: String,
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PoetListResponse {
    pub poets: Vec<PoetRow>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct DynastiesResponse {
    pub dynasties: Vec<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PoetQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let keyword = params.keyword.unwrap_or_default();
    let dynasty = params.dynasty.unwrap_or_default();

    let keyword_filter = if keyword.is_empty() {
        None
    } else {
        Some(format!("%{keyword}%"))
    };
    let dynasty_filter = if dynasty.is_empty() {
        None
    } else {
        Some(dynasty.clone())
    };

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM poets WHERE (? IS NULL OR name LIKE ?) AND (? IS NULL OR dynasty = ?)",
    )
    .bind(&keyword_filter)
    .bind(&keyword_filter)
    .bind(&dynasty_filter)
    .bind(&dynasty_filter)
    .fetch_one(&state.db)
    .await?;

    let poets: Vec<PoetRow> = sqlx::query_as(
        "SELECT id, name, dynasty, created_at FROM poets WHERE (? IS NULL OR name LIKE ?) AND (? IS NULL OR dynasty = ?) ORDER BY id DESC LIMIT ? OFFSET ?",
    )
    .bind(&keyword_filter)
    .bind(&keyword_filter)
    .bind(&dynasty_filter)
    .bind(&dynasty_filter)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(PoetListResponse {
        poets,
        total,
        page,
        per_page,
    }))
}

pub async fn get_dynasties(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let dynasties: Vec<String> = sqlx::query_scalar("SELECT DISTINCT dynasty FROM poets ORDER BY dynasty")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(DynastiesResponse { dynasties }))
}

pub async fn create(
    State(state): State<AppState>,
    Json(form): Json<CreatePoetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let name = form.name.trim().to_string();
    let dynasty = form.dynasty.trim().to_string();

    if name.is_empty() || dynasty.is_empty() {
        return Err(AppError::Validation("姓名和朝代不能为空".to_string()));
    }

    let result = sqlx::query("INSERT INTO poets (name, dynasty) VALUES (?, ?)")
        .bind(&name)
        .bind(&dynasty)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) => {
            let poet: PoetRow = sqlx::query_as(
                "SELECT id, name, dynasty, created_at FROM poets WHERE id = ?",
            )
            .bind(r.last_insert_id())
            .fetch_one(&state.db)
            .await?;

            Ok(Json(poet))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Duplicate entry") || msg.contains("uk_name_dynasty") {
                Err(AppError::Duplicate(format!(
                    "诗人 {name}（{dynasty}）已存在"
                )))
            } else {
                Err(AppError::Database(e))
            }
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(form): Json<CreatePoetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let name = form.name.trim().to_string();
    let dynasty = form.dynasty.trim().to_string();

    if name.is_empty() || dynasty.is_empty() {
        return Err(AppError::Validation("姓名和朝代不能为空".to_string()));
    }

    let result = sqlx::query("UPDATE poets SET name = ?, dynasty = ? WHERE id = ?")
        .bind(&name)
        .bind(&dynasty)
        .bind(id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                return Err(AppError::NotFound(format!("诗人 #{id} 不存在")));
            }
            let poet: PoetRow = sqlx::query_as(
                "SELECT id, name, dynasty, created_at FROM poets WHERE id = ?",
            )
            .bind(id)
            .fetch_one(&state.db)
            .await?;

            Ok(Json(poet))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Duplicate entry") || msg.contains("uk_name_dynasty") {
                Err(AppError::Duplicate(format!(
                    "诗人 {name}（{dynasty}）已存在"
                )))
            } else {
                Err(AppError::Database(e))
            }
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM poets WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                return Err(AppError::NotFound(format!("诗人 #{id} 不存在")));
            }
            Ok(Json(serde_json::json!({ "message": "删除成功" })))
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("a foreign key constraint fails") {
                Err(AppError::Validation(
                    "该诗人下还有诗词，无法删除。请先删除或移动相关诗词。".to_string(),
                ))
            } else {
                Err(AppError::Database(e))
            }
        }
    }
}
