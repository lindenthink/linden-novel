use sqlx::SqlitePool;

use crate::db::repo::{chapter_element_repo, embedding_repo, worldview_repo};
use crate::error::AppError;
use crate::models::embedding::EmbeddingSourceType;
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
    // 清理关联向量（embeddings）
    if let Err(e) = embedding_repo::delete_by_source(pool, EmbeddingSourceType::Worldview, id).await {
        tracing::warn!("Failed to clean worldview embeddings {}: {}", id, e);
    }
    // 清理所有章节对该世界观条目的引用（snapshots 不支持 worldview，跳过）
    if let Err(e) = chapter_element_repo::remove_by_element(pool, "worldview", id).await {
        tracing::warn!("Failed to clean chapter elements for worldview {}: {}", id, e);
    }
    worldview_repo::delete(pool, id).await.map_err(AppError::from)
}
