use sqlx::SqlitePool;

use crate::db::repo::project_repo;
use crate::error::AppError;
use crate::models::project::{CreateProject, Project, UpdateProject};

pub async fn list(pool: &SqlitePool) -> Result<Vec<Project>, AppError> {
    project_repo::list(pool).await.map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Project, AppError> {
    project_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Project '{id}' not found")))
}

pub async fn create(pool: &SqlitePool, input: &CreateProject) -> Result<Project, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title must not be empty".into()));
    }
    project_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateProject,
) -> Result<Project, AppError> {
    // 确保存在
    get(pool, id).await?;
    project_repo::update(pool, id, input).await.map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    project_repo::delete(pool, id).await.map_err(AppError::from)
}
