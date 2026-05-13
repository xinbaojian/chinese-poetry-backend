use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::{AppError, AppResult};
use crate::models::user::UserInfo;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PoemQuery {
    pub keyword: Option<String>,
    pub dynasty: Option<String>,
    pub category: Option<String>,
    pub grade: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PoemItem {
    pub id: u64,
    pub title: String,
    pub poet_name: String,
    pub dynasty: String,
    pub category: String,
    pub grade: u8,
    pub content: String,
    pub translation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PoemListResponse {
    pub poems: Vec<PoemItem>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

pub async fn list_poems(
    Extension(_user): Extension<UserInfo>,
    State(state): State<AppState>,
    Query(query): Query<PoemQuery>,
) -> AppResult<Json<PoemListResponse>> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let keyword = query.keyword.as_deref();
    let dynasty = query.dynasty.as_deref();
    let category = query.category.as_deref();
    let grade = query.grade.as_deref().and_then(|g| g.parse::<u8>().ok()).filter(|&g| g > 0);

    // Count total
    let count_sql = concat!(
        "SELECT COUNT(*) as total FROM poems p ",
        "JOIN poets pt ON p.poet_id = pt.id ",
        "WHERE (? IS NULL OR p.title LIKE CONCAT('%', ?, '%')) ",
        "AND (? IS NULL OR p.dynasty = ?) ",
        "AND (? IS NULL OR p.category = ?) ",
        "AND (? IS NULL OR p.grade = ?)"
    );

    let (total,): (i64,) = sqlx::query_as(count_sql)
        .bind(keyword)
        .bind(keyword)
        .bind(dynasty)
        .bind(dynasty)
        .bind(category)
        .bind(category)
        .bind(grade)
        .bind(grade)
        .fetch_one(&state.db)
        .await?;

    // Fetch page
    let data_sql = concat!(
        "SELECT p.id, p.title, pt.name as poet_name, p.dynasty, p.category, p.grade, ",
        "p.content, p.translation ",
        "FROM poems p JOIN poets pt ON p.poet_id = pt.id ",
        "WHERE (? IS NULL OR p.title LIKE CONCAT('%', ?, '%')) ",
        "AND (? IS NULL OR p.dynasty = ?) ",
        "AND (? IS NULL OR p.category = ?) ",
        "AND (? IS NULL OR p.grade = ?) ",
        "ORDER BY p.id DESC LIMIT ? OFFSET ?"
    );

    let poems: Vec<PoemItem> = sqlx::query_as(data_sql)
        .bind(keyword)
        .bind(keyword)
        .bind(dynasty)
        .bind(dynasty)
        .bind(category)
        .bind(category)
        .bind(grade)
        .bind(grade)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(PoemListResponse {
        poems,
        total,
        page,
        per_page,
    }))
}

pub async fn get_poem(
    Extension(_user): Extension<UserInfo>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> AppResult<Json<PoemItem>> {
    let sql = concat!(
        "SELECT p.id, p.title, pt.name as poet_name, p.dynasty, p.category, p.grade, ",
        "p.content, p.translation ",
        "FROM poems p JOIN poets pt ON p.poet_id = pt.id ",
        "WHERE p.id = ?"
    );

    let poem: PoemItem = sqlx::query_as(sql)
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::NotFound(format!("诗词 id={} 不存在", id)))?;

    Ok(Json(poem))
}
