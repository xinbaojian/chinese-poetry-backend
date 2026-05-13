use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "认证已过期，请重新登录".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "请先登录".to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Duplicate(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!(error = %msg, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误".to_string())
            }
            AppError::Io(e) => {
                tracing::error!(error = %e, "IO error");
                (StatusCode::INTERNAL_SERVER_ERROR, "IO 错误".to_string())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
