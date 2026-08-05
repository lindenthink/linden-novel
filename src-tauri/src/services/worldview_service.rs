use sqlx::SqlitePool;

use crate::db::repo::worldview_repo;
use crate::error::AppError;
use crate::models::worldview::{WorldviewEntry, CreateWorldviewEntry, UpdateWorldviewEntry};

pub async fn list(pool: &SqlitePool, project_id: &str) -> Result<Vec<WorldviewEntry>, AppError> {
    worldview_repo::list_by_project(pool, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<WorldviewEntry, AppError> {
    worldview_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Worldview entry '{id}' not found")))
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateWorldviewEntry,
) -> Result<WorldviewEntry, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name must not be empty".into()));
    }
    worldview_repo::create(pool, input)
        .await
        .map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateWorldviewEntry,
) -> Result<WorldviewEntry, AppError> {
    get(pool, id).await?;
    worldview_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    worldview_repo::delete(pool, id).await.map_err(AppError::from)
}
