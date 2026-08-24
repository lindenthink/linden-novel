use sqlx::SqlitePool;

use crate::db::repo::foreshadow_repo;
use crate::error::AppError;
use crate::models::foreshadow::{CreateForeshadow, Foreshadow, UpdateForeshadow};

pub async fn list(pool: &SqlitePool, project_id: &str) -> Result<Vec<Foreshadow>, AppError> {
    foreshadow_repo::list_by_project(pool, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Foreshadow, AppError> {
    foreshadow_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Foreshadow '{id}' not found")))
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateForeshadow,
) -> Result<Foreshadow, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title must not be empty".into()));
    }
    foreshadow_repo::create(pool, input)
        .await
        .map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateForeshadow,
) -> Result<Foreshadow, AppError> {
    get(pool, id).await?;
    foreshadow_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    foreshadow_repo::delete(pool, id)
        .await
        .map_err(AppError::from)
}
