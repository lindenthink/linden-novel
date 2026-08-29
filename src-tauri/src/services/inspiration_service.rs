use sqlx::SqlitePool;

use crate::db::repo::inspiration_repo;
use crate::error::AppError;
use crate::models::inspiration::{CreateInspiration, Inspiration, UpdateInspiration};

pub async fn list(pool: &SqlitePool, project_id: &str) -> Result<Vec<Inspiration>, AppError> {
    inspiration_repo::list_by_project(pool, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Inspiration, AppError> {
    inspiration_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Inspiration '{id}' not found")))
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateInspiration,
) -> Result<Inspiration, AppError> {
    if input.content.trim().is_empty() {
        return Err(AppError::Validation("Content must not be empty".into()));
    }
    inspiration_repo::create(pool, input)
        .await
        .map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateInspiration,
) -> Result<Inspiration, AppError> {
    get(pool, id).await?;
    inspiration_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    inspiration_repo::delete(pool, id)
        .await
        .map_err(AppError::from)
}
