use sqlx::SqlitePool;

use crate::db::repo::settings_repo;
use crate::error::AppError;

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, AppError> {
    settings_repo::get(pool, key)
        .await
        .map_err(AppError::from)
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    settings_repo::set(pool, key, value)
        .await
        .map_err(AppError::from)
}
