use sqlx::SqlitePool;

use crate::db::repo::{chapter_element_repo, embedding_repo, entity_snapshot_repo, storyline_repo};
use crate::error::AppError;
use crate::models::embedding::EmbeddingSourceType;
use crate::models::entity_snapshot::EntityType;
use crate::models::storyline::{Storyline, CreateStoryline, UpdateStoryline};

pub async fn list(pool: &SqlitePool, project_id: &str) -> Result<Vec<Storyline>, AppError> {
    storyline_repo::list_by_project(pool, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Storyline, AppError> {
    storyline_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Storyline '{id}' not found")))
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateStoryline,
) -> Result<Storyline, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name must not be empty".into()));
    }
    storyline_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateStoryline,
) -> Result<Storyline, AppError> {
    get(pool, id).await?;
    storyline_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    // 清理关联向量（embeddings）
    if let Err(e) = embedding_repo::delete_by_source(pool, EmbeddingSourceType::Storyline, id).await {
        tracing::warn!("Failed to clean storyline embeddings {}: {}", id, e);
    }
    // 清理关联 entity_snapshots
    if let Err(e) = entity_snapshot_repo::delete_by_entity(pool, EntityType::Storyline, id).await {
        tracing::warn!("Failed to clean storyline snapshots {}: {}", id, e);
    }
    // 清理所有章节对该故事线的引用
    if let Err(e) = chapter_element_repo::remove_by_element(pool, "storyline", id).await {
        tracing::warn!("Failed to clean chapter elements for storyline {}: {}", id, e);
    }
    storyline_repo::delete(pool, id).await.map_err(AppError::from)
}
