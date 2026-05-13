use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Database row models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ExportRow {
    pub id: u64,
    pub username: String,
    pub user_id: u64,
    pub poem_title: String,
    pub poet_name: String,
    pub mastery_level: String,
    pub review_count: i64,
    pub next_review_date: Option<String>,
    pub lr_created_at: String,
    pub lr_updated_at: String,
}

// ---------------------------------------------------------------------------
// Query struct
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
    pub user_id: Option<u64>,
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct UserListForExport {
    pub users: Vec<UserItem>,
}

#[derive(Debug, Serialize)]
pub struct UserItem {
    pub id: u64,
    pub username: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users: Vec<UserItem> = sqlx::query_as("SELECT id, username FROM users ORDER BY id")
        .fetch_all(&state.db)
        .await
        .map(|rows: Vec<IdUsernameRow>| {
            rows.into_iter()
                .map(|r| UserItem { id: r.id, username: r.username })
                .collect()
        })?;

    Ok(Json(UserListForExport { users }))
}

pub async fn download(
    State(state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let format = params.format.unwrap_or_else(|| "csv".to_string());

    let rows: Vec<ExportRow> = sqlx::query_as(
        "SELECT lr.id, u.username, lr.user_id, p.title as poem_title, pt.name as poet_name, \
         lr.mastery_level, lr.review_count, \
         DATE_FORMAT(lr.next_review_date, '%Y-%m-%d %H:%i:%s') as next_review_date, \
         DATE_FORMAT(lr.created_at, '%Y-%m-%d %H:%i:%s') as lr_created_at, \
         DATE_FORMAT(lr.updated_at, '%Y-%m-%d %H:%i:%s') as lr_updated_at \
         FROM learning_records lr \
         JOIN users u ON lr.user_id = u.id \
         JOIN poems p ON lr.poem_id = p.id \
         JOIN poets pt ON p.poet_id = pt.id \
         WHERE (? IS NULL OR lr.user_id = ?) \
         ORDER BY lr.id",
    )
    .bind(params.user_id)
    .bind(params.user_id)
    .fetch_all(&state.db)
    .await?;

    let (body, content_type, extension) = if format == "json" {
        let json = serde_json::to_string_pretty(&rows)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        (json, "application/json", "json")
    } else {
        let mut csv = String::from(
            "ID,用户名,用户ID,诗词标题,诗人,掌握程度,复习次数,下次复习时间,创建时间,更新时间\n",
        );
        for row in &rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                row.id,
                row.username,
                row.user_id,
                row.poem_title,
                row.poet_name,
                row.mastery_level,
                row.review_count,
                row.next_review_date.as_deref().unwrap_or(""),
                row.lr_created_at,
                row.lr_updated_at,
            ));
        }
        (csv, "text/csv", "csv")
    };

    let filename = format!("learning_data.{}", extension);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
        .unwrap()
        .into_response())
}

// Helper struct for fetching id + username only
#[derive(Debug, FromRow)]
struct IdUsernameRow {
    id: u64,
    username: String,
}
