use sqlx::SqlitePool;
use std::path::Path;

use crate::db::repo::{chapter_element_repo, embedding_repo, entity_snapshot_repo, storyline_repo};
use crate::error::AppError;
use crate::models::embedding::EmbeddingSourceType;
use crate::models::entity_snapshot::EntityType;
use crate::models::storyline::{Storyline, CreateStoryline, UpdateStoryline};
use crate::services::embedding_service;

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
    app_data_dir: &Path,
    input: &CreateStoryline,
) -> Result<Storyline, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name must not be empty".into()));
    }
    let storyline = storyline_repo::create(pool, input).await.map_err(AppError::from)?;
    
    // 异步触发嵌入
    if let Some(desc) = &storyline.description {
        if !desc.trim().is_empty() {
            let pool_bg = pool.clone();
            let dir_bg = app_data_dir.to_path_buf();
            let proj_bg = storyline.project_id.clone();
            let id_bg = storyline.id.clone();
            let desc_bg = desc.clone();
            tokio::spawn(async move {
                if let Err(e) = embedding_service::generate_and_store(
                    &pool_bg, &dir_bg, &proj_bg, "storyline", &id_bg, &desc_bg, "",
                ).await {
                    tracing::warn!("Failed to embed new storyline {}: {}", id_bg, e);
                }
            });
        }
    }
    Ok(storyline)
}

pub async fn update(
    pool: &SqlitePool,
    app_data_dir: &Path,
    id: &str,
    input: &UpdateStoryline,
) -> Result<Storyline, AppError> {
    let _old = get(pool, id).await?;
    let updated = storyline_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)?;
        
    // 异步触发嵌入
    if let Some(desc) = &updated.description {
        if !desc.trim().is_empty() {
            let pool_bg = pool.clone();
            let dir_bg = app_data_dir.to_path_buf();
            let proj_bg = updated.project_id.clone();
            let id_bg = updated.id.clone();
            let desc_bg = desc.clone();
            tokio::spawn(async move {
                if let Err(e) = embedding_service::generate_and_store(
                    &pool_bg, &dir_bg, &proj_bg, "storyline", &id_bg, &desc_bg, "",
                ).await {
                    tracing::warn!("Failed to embed updated storyline {}: {}", id_bg, e);
                }
            });
        }
    }
    Ok(updated)
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
