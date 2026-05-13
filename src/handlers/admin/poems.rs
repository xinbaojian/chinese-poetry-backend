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
pub struct PoemRow {
    pub id: u64,
    pub title: String,
    pub poet_id: u64,
    pub dynasty: String,
    pub category: String,
    pub grade: u8,
    pub content: String,
    pub translation: Option<String>,
    pub source_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// A poem joined with its poet name for the list view.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PoemWithPoet {
    pub id: u64,
    pub title: String,
    pub poet_id: u64,
    pub poet_name: String,
    pub dynasty: String,
    pub category: String,
    pub grade: u8,
}

/// A lightweight poet row for dropdowns.
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PoetOption {
    pub id: u64,
    pub name: String,
    pub dynasty: String,
}

// ---------------------------------------------------------------------------
// Query / Request structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PoemQuery {
    pub keyword: Option<String>,
    pub dynasty: Option<String>,
    pub category: Option<String>,
    pub grade: Option<String>,
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePoemRequest {
    pub title: String,
    pub poet_id: Option<u64>,
    pub poet_name: Option<String>,
    pub dynasty: String,
    pub category: Option<String>,
    pub grade: Option<u8>,
    pub content: String,
    pub translation: Option<String>,
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PoemListResponse {
    pub poems: Vec<PoemWithPoet>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct PoemDetailResponse {
    pub id: u64,
    pub title: String,
    pub poet_id: u64,
    pub poet_name: String,
    pub dynasty: String,
    pub category: String,
    pub grade: u8,
    pub content: Vec<String>,
    pub translation: Option<String>,
    pub source_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct FilterOptionsResponse {
    pub dynasties: Vec<String>,
    pub categories: Vec<String>,
    pub grades: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<PoemQuery>,
) -> Result<impl IntoResponse, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page: u32 = 20;
    let offset = (page - 1) * per_page;

    let keyword = params.keyword.unwrap_or_default();
    let dynasty = params.dynasty.unwrap_or_default();
    let category = params.category.unwrap_or_default();
    let grade_val = params.grade.as_deref().and_then(|g| g.parse::<u8>().ok()).filter(|&g| g > 0);

    let keyword_filter = if keyword.is_empty() { None } else { Some(format!("%{keyword}%")) };
    let dynasty_filter = if dynasty.is_empty() { None } else { Some(dynasty.clone()) };
    let category_filter = if category.is_empty() { None } else { Some(category.clone()) };
    let grade_filter = grade_val;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM poems p \
         LEFT JOIN poets pt ON p.poet_id = pt.id \
         WHERE (? IS NULL OR p.title LIKE ?) \
           AND (? IS NULL OR p.dynasty = ?) \
           AND (? IS NULL OR p.category = ?) \
           AND (? IS NULL OR p.grade = ?)",
    )
    .bind(&keyword_filter)
    .bind(&keyword_filter)
    .bind(&dynasty_filter)
    .bind(&dynasty_filter)
    .bind(&category_filter)
    .bind(&category_filter)
    .bind(grade_filter)
    .bind(grade_filter)
    .fetch_one(&state.db)
    .await?;

    let poems: Vec<PoemWithPoet> = sqlx::query_as(
        "SELECT p.id, p.title, p.poet_id, pt.name AS poet_name, p.dynasty, p.category, p.grade \
         FROM poems p \
         LEFT JOIN poets pt ON p.poet_id = pt.id \
         WHERE (? IS NULL OR p.title LIKE ?) \
           AND (? IS NULL OR p.dynasty = ?) \
           AND (? IS NULL OR p.category = ?) \
           AND (? IS NULL OR p.grade = ?) \
         ORDER BY p.id DESC \
         LIMIT ? OFFSET ?",
    )
    .bind(&keyword_filter)
    .bind(&keyword_filter)
    .bind(&dynasty_filter)
    .bind(&dynasty_filter)
    .bind(&category_filter)
    .bind(&category_filter)
    .bind(grade_filter)
    .bind(grade_filter)
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
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let poem: PoemRow = sqlx::query_as(
        "SELECT id, title, poet_id, dynasty, category, grade, CAST(content AS CHAR) AS content, translation, source_id, created_at, updated_at \
         FROM poems WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("诗词 #{id} 不存在")))?;

    let poet_name: String = sqlx::query_scalar("SELECT name FROM poets WHERE id = ?")
        .bind(poem.poet_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    // Parse JSON content back to paragraphs
    let paragraphs: Vec<String> = serde_json::from_str(&poem.content).unwrap_or_else(|_| {
        vec![poem.content.clone()]
    });

    Ok(Json(PoemDetailResponse {
        id: poem.id,
        title: poem.title,
        poet_id: poem.poet_id,
        poet_name,
        dynasty: poem.dynasty,
        category: poem.category,
        grade: poem.grade,
        content: paragraphs,
        translation: poem.translation,
        source_id: poem.source_id,
        created_at: poem.created_at,
        updated_at: poem.updated_at,
    }))
}

pub async fn get_filter_options(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let dynasties: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT dynasty FROM poems WHERE dynasty != '' ORDER BY dynasty")
            .fetch_all(&state.db)
            .await?;

    let categories: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT category FROM poems WHERE category != '' ORDER BY category")
            .fetch_all(&state.db)
            .await?;

    let grades: Vec<u8> =
        sqlx::query_scalar("SELECT DISTINCT grade FROM poems WHERE grade > 0 ORDER BY grade")
            .fetch_all(&state.db)
            .await?;

    Ok(Json(FilterOptionsResponse { dynasties, categories, grades }))
}

pub async fn get_poets(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let poets: Vec<PoetOption> = sqlx::query_as(
        "SELECT id, name, dynasty FROM poets ORDER BY dynasty, name",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "poets": poets })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(form): Json<CreatePoemRequest>,
) -> Result<impl IntoResponse, AppError> {
    let title = form.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation("标题不能为空".to_string()));
    }

    let paragraphs: Vec<String> = form
        .content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if paragraphs.is_empty() {
        return Err(AppError::Validation("内容不能为空".to_string()));
    }

    let content_json = serde_json::to_string(&paragraphs)
        .map_err(|e| AppError::Internal(format!("JSON序列化失败: {e}")))?;

    let poet_id = resolve_poet_id(&state, form.poet_id, form.poet_name.as_deref(), form.dynasty.trim()).await?;

    let dynasty = if form.dynasty.trim().is_empty() {
        let poet_dynasty: Option<String> = sqlx::query_scalar(
            "SELECT dynasty FROM poets WHERE id = ?",
        )
        .bind(poet_id)
        .fetch_optional(&state.db)
        .await?;
        poet_dynasty.unwrap_or_default()
    } else {
        form.dynasty.trim().to_string()
    };

    let category = form.category.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
    let translation = form.translation.map(|t| t.trim().to_string()).filter(|t| !t.is_empty());

    sqlx::query(
        "INSERT INTO poems (title, poet_id, dynasty, category, grade, content, translation) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&title)
    .bind(poet_id)
    .bind(&dynasty)
    .bind(&category)
    .bind(form.grade.unwrap_or(0))
    .bind(&content_json)
    .bind(&translation)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "创建成功" })))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(form): Json<CreatePoemRequest>,
) -> Result<impl IntoResponse, AppError> {
    let title = form.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Validation("标题不能为空".to_string()));
    }

    let paragraphs: Vec<String> = form
        .content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if paragraphs.is_empty() {
        return Err(AppError::Validation("内容不能为空".to_string()));
    }

    let content_json = serde_json::to_string(&paragraphs)
        .map_err(|e| AppError::Internal(format!("JSON序列化失败: {e}")))?;

    let poet_id = resolve_poet_id(&state, form.poet_id, form.poet_name.as_deref(), form.dynasty.trim()).await?;

    let dynasty = if form.dynasty.trim().is_empty() {
        let poet_dynasty: Option<String> = sqlx::query_scalar(
            "SELECT dynasty FROM poets WHERE id = ?",
        )
        .bind(poet_id)
        .fetch_optional(&state.db)
        .await?;
        poet_dynasty.unwrap_or_default()
    } else {
        form.dynasty.trim().to_string()
    };

    let category = form.category.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
    let translation = form.translation.map(|t| t.trim().to_string()).filter(|t| !t.is_empty());

    let result = sqlx::query(
        "UPDATE poems SET title = ?, poet_id = ?, dynasty = ?, category = ?, grade = ?, content = ?, translation = ? \
         WHERE id = ?",
    )
    .bind(&title)
    .bind(poet_id)
    .bind(&dynasty)
    .bind(&category)
    .bind(form.grade.unwrap_or(0))
    .bind(&content_json)
    .bind(&translation)
    .bind(id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("诗词 #{id} 不存在")));
    }

    Ok(Json(serde_json::json!({ "message": "更新成功" })))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM poems WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("诗词 #{id} 不存在")));
    }

    Ok(Json(serde_json::json!({ "message": "删除成功" })))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn resolve_poet_id(
    state: &AppState,
    poet_id: Option<u64>,
    poet_name: Option<&str>,
    dynasty: &str,
) -> Result<u64, AppError> {
    if let Some(id) = poet_id {
        if id > 0 {
            return Ok(id);
        }
    }

    let name = poet_name
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("请选择或输入诗人".to_string()))?;

    let poet_dynasty = if dynasty.is_empty() { "唐" } else { dynasty };

    let existing: Option<(u64,)> = sqlx::query_as(
        "SELECT id FROM poets WHERE name = ? AND dynasty = ?"
    )
    .bind(name)
    .bind(poet_dynasty)
    .fetch_optional(&state.db)
    .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let result = sqlx::query("INSERT INTO poets (name, dynasty) VALUES (?, ?)")
        .bind(name)
        .bind(poet_dynasty)
        .execute(&state.db)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("Duplicate entry") {
                AppError::Duplicate(format!("诗人「{name}（{poet_dynasty}）」刚被创建"))
            } else {
                AppError::Database(e)
            }
        })?;

    Ok(result.last_insert_id())
}
