use serde::Serialize;

/// 统一应用错误类型，序列化为 JSON 返回前端
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "variant", content = "message")]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".into()),
            _ => AppError::Db(e.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}
