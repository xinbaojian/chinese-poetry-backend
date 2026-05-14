use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Data structures for JSON import
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PoemEntry {
    title: String,
    author: Option<String>,
    dynasty: Option<String>,
    #[serde(alias = "paragraphs")]
    paragraphes: Option<Vec<String>>,
    id: Option<serde_json::Value>,
    category: Option<String>,
    grade: Option<u8>,
    translation: Option<String>,
    #[serde(default)]
    strain: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Request / Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub json_data: String,
    pub dynasty: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportResults {
    pub imported: u32,
    pub skipped: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn import_poems(
    State(state): State<AppState>,
    Json(form): Json<ImportRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.db;

    let default_dynasty = form
        .dynasty
        .as_deref()
        .map(|d| d.trim())
        .filter(|d| !d.is_empty())
        .unwrap_or("唐")
        .to_string();

    let entries: Vec<PoemEntry> = match serde_json::from_str(form.json_data.trim()) {
        Ok(v) => v,
        Err(e) => {
            return Ok(Json(ImportResults {
                imported: 0,
                skipped: 0,
                failed: 0,
                errors: vec![format!("JSON 解析失败: {e}")],
            }));
        }
    };

    // Pre-load all poets into memory to avoid repeated queries
    let all_poets: Vec<(u64, String, String)> = sqlx::query_as(
        "SELECT id, name, dynasty FROM poets"
    )
    .fetch_all(pool)
    .await?;

    let mut poet_cache: std::collections::HashMap<(String, String), u64> = all_poets
        .into_iter()
        .map(|(id, name, dynasty)| ((name, dynasty), id))
        .collect();

    let mut imported: u32 = 0;
    let mut skipped: u32 = 0;
    let mut failed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for (index, entry) in entries.into_iter().enumerate() {
        let title = entry.title.trim().to_string();
        if title.is_empty() {
            failed += 1;
            errors.push(format!("第 {} 条: 标题为空", index + 1));
            continue;
        }

        let paragraphs = match entry.paragraphes {
            Some(p) if !p.is_empty() => p,
            _ => {
                failed += 1;
                errors.push(format!("「{title}」: 缺少段落内容 (paragraphes/paragraphs)"));
                continue;
            }
        };

        let author_name = match entry.author.as_deref().map(|a| a.trim()) {
            Some(a) if !a.is_empty() => a.to_string(),
            _ => {
                failed += 1;
                errors.push(format!("「{title}」: 缺少作者 (author)"));
                continue;
            }
        };

        let dynasty = entry
            .dynasty
            .as_deref()
            .map(|d| d.trim())
            .filter(|d| !d.is_empty())
            .unwrap_or(&default_dynasty)
            .to_string();

        // Resolve poet from cache or create
        let poet_id = match poet_cache.get(&(author_name.clone(), dynasty.clone())) {
            Some(&id) => id,
            None => {
                match find_or_create_poet(pool, &author_name, &dynasty).await {
                    Ok(id) => {
                        poet_cache.insert((author_name.clone(), dynasty.clone()), id);
                        id
                    }
                    Err(e) => {
                        failed += 1;
                        errors.push(format!("「{title}」: 创建诗人失败 - {e}"));
                        continue;
                    }
                }
            }
        };

        let content_json = serde_json::to_string(&paragraphs)
            .map_err(|e| AppError::Internal(format!("JSON 序列化失败: {e}")))?;

        let source_id = entry.id.map(|v| match v {
            serde_json::Value::String(s) => s,
            other => other.to_string(),
        });

        // Resolve fields — category/grade are NOT NULL in DB
        let category: String = entry.category
            .as_deref()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .unwrap_or("")
            .to_string();
        let grade: u8 = entry.grade.unwrap_or(0);
        let translation = entry.translation.as_deref().map(|t| t.trim()).filter(|t| !t.is_empty());

        // Check duplicate
        let existing: Option<(u64,)> = if let Some(ref sid) = source_id {
            sqlx::query_as("SELECT id FROM poems WHERE source_id = ?")
                .bind(sid)
                .fetch_optional(pool)
                .await
                .map_err(|e| AppError::Database(e))?
        } else {
            let first_line = paragraphs.first().map(|s| s.as_str()).unwrap_or("");
            sqlx::query_as(
                "SELECT id FROM poems WHERE title = ? AND poet_id = ? AND JSON_UNQUOTE(JSON_EXTRACT(content, '$[0]')) = ?"
            )
            .bind(&title)
            .bind(poet_id)
            .bind(first_line)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Database(e))?
        };

        if let Some((existing_id,)) = existing {
            // Backfill category/grade/translation for previously imported records
            if !category.is_empty() || grade > 0 || translation.is_some() {
                let _ = sqlx::query(
                    "UPDATE poems SET category = ?, grade = ?, translation = COALESCE(?, translation) WHERE id = ? \
                     AND (category = '' OR category IS NULL OR grade = 0 OR grade IS NULL)"
                )
                .bind(&category)
                .bind(grade)
                .bind(translation)
                .bind(existing_id)
                .execute(pool)
                .await;
            }
            skipped += 1;
            continue;
        }

        let insert_result = sqlx::query(
            "INSERT INTO poems (title, poet_id, dynasty, category, grade, content, translation, source_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&title)
        .bind(poet_id)
        .bind(&dynasty)
        .bind(&category)
        .bind(grade)
        .bind(&content_json)
        .bind(translation)
        .bind(&source_id)
        .execute(pool)
        .await;

        match insert_result {
            Ok(_) => imported += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("「{title}」: 插入失败 - {e}"));
            }
        }
    }

    Ok(Json(ImportResults {
        imported,
        skipped,
        failed,
        errors,
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn find_or_create_poet(
    pool: &sqlx::MySqlPool,
    name: &str,
    dynasty: &str,
) -> Result<u64, AppError> {
    let existing: Option<(u64,)> =
        sqlx::query_as("SELECT id FROM poets WHERE name = ? AND dynasty = ?")
            .bind(name)
            .bind(dynasty)
            .fetch_optional(pool)
            .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let result = sqlx::query("INSERT INTO poets (name, dynasty) VALUES (?, ?)")
        .bind(name)
        .bind(dynasty)
        .execute(pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("Duplicate entry") {
                AppError::Duplicate(format!("诗人 {name}（{dynasty}）已存在"))
            } else {
                AppError::Database(e)
            }
        })?;

    Ok(result.last_insert_id())
}
