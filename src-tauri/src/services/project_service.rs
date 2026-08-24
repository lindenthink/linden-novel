use sqlx::SqlitePool;

use crate::db::repo::{embedding_chunk_repo, embedding_repo, foreshadow_repo, project_repo};
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
    // 清理项目级向量（摘要级 + 切片级），失败仅告警不阻塞业务删除
    if let Err(e) = embedding_repo::delete_by_project(pool, id).await {
        tracing::warn!("Failed to clean summary embeddings for project {}: {}", id, e);
    }
    if let Err(e) = embedding_chunk_repo::delete_by_project(pool, id).await {
        tracing::warn!("Failed to clean chunk embeddings for project {}: {}", id, e);
    }
    // 清理项目级伏笔（schema 已 ON DELETE CASCADE，但显式调用避免遗漏）
    if let Err(e) = foreshadow_repo::delete_by_project(pool, id).await {
        tracing::warn!("Failed to clean foreshadows for project {}: {}", id, e);
    }
    project_repo::delete(pool, id).await.map_err(AppError::from)
}
