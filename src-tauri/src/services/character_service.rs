use sqlx::SqlitePool;

use crate::db::repo::{chapter_element_repo, character_repo, embedding_repo};
use crate::error::AppError;
use crate::models::character::{Character, CreateCharacter, UpdateCharacter};
use crate::models::embedding::EmbeddingSourceType;

pub async fn list(pool: &SqlitePool, project_id: &str) -> Result<Vec<Character>, AppError> {
    character_repo::list_by_project(pool, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Character, AppError> {
    character_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Character '{id}' not found")))
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateCharacter,
) -> Result<Character, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name must not be empty".into()));
    }
    character_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateCharacter,
) -> Result<Character, AppError> {
    get(pool, id).await?;
    character_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    // 清理关联向量（embeddings）
    if let Err(e) = embedding_repo::delete_by_source(pool, EmbeddingSourceType::Character, id).await {
        tracing::warn!("Failed to clean character embeddings {}: {}", id, e);
    }
    // 清理所有章节对该角色的引用
    if let Err(e) = chapter_element_repo::remove_by_element(pool, "character", id).await {
        tracing::warn!("Failed to clean chapter elements for character {}: {}", id, e);
    }
    character_repo::delete(pool, id).await.map_err(AppError::from)
}
