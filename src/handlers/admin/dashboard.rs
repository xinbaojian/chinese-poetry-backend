use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub total_poems: i64,
    pub total_poets: i64,
    pub total_users: i64,
    pub total_records: i64,
}

pub async fn dashboard(State(state): State<AppState>) -> Result<impl IntoResponse, crate::errors::AppError> {
    let total_poems = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM poems")
        .fetch_one(&state.db)
        .await?;

    let total_poets = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM poets")
        .fetch_one(&state.db)
        .await?;

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    let total_records = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM learning_records")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(DashboardResponse {
        total_poems,
        total_poets,
        total_users,
        total_records,
    }))
}
