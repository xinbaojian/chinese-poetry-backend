use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::AppResult;
use crate::models::user::UserInfo;
use crate::state::AppState;

#[derive(Debug, Serialize, FromRow)]
pub struct LearningRecordItem {
    pub id: u64,
    pub poem_id: u64,
    pub poem_title: String,
    pub poet_name: String,
    pub mastery_level: String,
    pub review_count: u32,
    pub next_review_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub records: Vec<SyncRecord>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRecord {
    pub poem_id: u64,
    pub mastery_level: String,
    pub review_count: u32,
    pub next_review_date: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub synced: u32,
    pub skipped: u32,
}

const PROGRESS_SELECT: &str = concat!(
    "SELECT lr.id, lr.poem_id, p.title as poem_title, pt.name as poet_name, ",
    "lr.mastery_level, lr.review_count, ",
    "DATE_FORMAT(lr.next_review_date, '%Y-%m-%dT%H:%i:%s') as next_review_date, ",
    "DATE_FORMAT(lr.created_at, '%Y-%m-%dT%H:%i:%s') as created_at, ",
    "DATE_FORMAT(lr.updated_at, '%Y-%m-%dT%H:%i:%s') as updated_at ",
    "FROM learning_records lr ",
    "JOIN poems p ON lr.poem_id = p.id ",
    "JOIN poets pt ON p.poet_id = pt.id ",
    "WHERE lr.user_id = ?"
);

pub async fn get_progress(
    Extension(user_info): Extension<UserInfo>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<LearningRecordItem>>> {
    let records: Vec<LearningRecordItem> = sqlx::query_as(PROGRESS_SELECT)
        .bind(user_info.id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(records))
}

pub async fn sync_progress(
    Extension(user_info): Extension<UserInfo>,
    State(state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> AppResult<Json<SyncResponse>> {
    let mut synced: u32 = 0;
    let mut skipped: u32 = 0;
    let user_id = user_info.id;

    for record in req.records {
        // Check if record exists
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s') FROM learning_records WHERE user_id = ? AND poem_id = ?"
        )
        .bind(user_id)
        .bind(record.poem_id)
        .fetch_optional(&state.db)
        .await?;

        match existing {
            Some((existing_updated_at,)) => {
                // Compare timestamps — keep newer
                if record.updated_at > existing_updated_at {
                    // Client data is newer, update
                    update_learning_record(
                        &state.db,
                        user_id,
                        &record,
                    )
                    .await?;
                    synced += 1;
                } else {
                    skipped += 1;
                }
            }
            None => {
                // New record, insert
                insert_learning_record(
                    &state.db,
                    user_id,
                    &record,
                )
                .await?;
                synced += 1;
            }
        }
    }

    Ok(Json(SyncResponse { synced, skipped }))
}

async fn insert_learning_record(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    record: &SyncRecord,
) -> AppResult<()> {
    let next_review_str = record.next_review_date.as_deref().unwrap_or("NULL");

    if record.next_review_date.is_some() {
        sqlx::query(
            "INSERT INTO learning_records (user_id, poem_id, mastery_level, review_count, next_review_date, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(record.poem_id)
        .bind(&record.mastery_level)
        .bind(record.review_count)
        .bind(next_review_str)
        .bind(&record.updated_at)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO learning_records (user_id, poem_id, mastery_level, review_count, updated_at) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(record.poem_id)
        .bind(&record.mastery_level)
        .bind(record.review_count)
        .bind(&record.updated_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn update_learning_record(
    pool: &sqlx::MySqlPool,
    user_id: u64,
    record: &SyncRecord,
) -> AppResult<()> {
    if record.next_review_date.is_some() {
        let next_review_str = record.next_review_date.as_deref().unwrap_or("NULL");
        sqlx::query(
            "UPDATE learning_records SET mastery_level = ?, review_count = ?, next_review_date = ?, updated_at = ? \
             WHERE user_id = ? AND poem_id = ?"
        )
        .bind(&record.mastery_level)
        .bind(record.review_count)
        .bind(next_review_str)
        .bind(&record.updated_at)
        .bind(user_id)
        .bind(record.poem_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE learning_records SET mastery_level = ?, review_count = ?, next_review_date = NULL, updated_at = ? \
             WHERE user_id = ? AND poem_id = ?"
        )
        .bind(&record.mastery_level)
        .bind(record.review_count)
        .bind(&record.updated_at)
        .bind(user_id)
        .bind(record.poem_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn delete_progress(
    Extension(user_info): Extension<UserInfo>,
    State(state): State<AppState>,
    Path(poem_id): Path<u64>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query(
        "DELETE FROM learning_records WHERE user_id = ? AND poem_id = ?"
    )
    .bind(user_info.id)
    .bind(poem_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        Ok(Json(serde_json::json!({
            "message": "记录不存在，无需删除"
        })))
    } else {
        Ok(Json(serde_json::json!({
            "message": "删除成功"
        })))
    }
}

pub async fn get_due_reviews(
    Extension(user_info): Extension<UserInfo>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<LearningRecordItem>>> {
    let sql = format!(
        "{} AND lr.next_review_date <= NOW()",
        PROGRESS_SELECT
    );

    let records: Vec<LearningRecordItem> = sqlx::query_as(&sql)
        .bind(user_info.id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(records))
}
