use sqlx::SqlitePool;
use std::path::Path;

use crate::db::repo::{chapter_element_repo, character_repo, embedding_repo, entity_snapshot_repo};
use crate::error::AppError;
use crate::models::character::{Character, CreateCharacter, UpdateCharacter};
use crate::models::embedding::EmbeddingSourceType;
use crate::models::entity_snapshot::EntityType;
use crate::services::embedding_service;

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
    app_data_dir: &Path,
    input: &CreateCharacter,
) -> Result<Character, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name must not be empty".into()));
    }
    let character = character_repo::create(pool, input).await.map_err(AppError::from)?;
    
    // 异步触发嵌入
    if let Some(desc) = &character.description {
        if !desc.trim().is_empty() {
            let pool_bg = pool.clone();
            let dir_bg = app_data_dir.to_path_buf();
            let proj_bg = character.project_id.clone();
            let id_bg = character.id.clone();
            let desc_bg = desc.clone();
            tokio::spawn(async move {
                if let Err(e) = embedding_service::generate_and_store(
                    &pool_bg, &dir_bg, &proj_bg, "character", &id_bg, &desc_bg, "",
                ).await {
                    tracing::warn!("Failed to embed new character {}: {}", id_bg, e);
                }
            });
        }
    }
    Ok(character)
}

pub async fn update(
    pool: &SqlitePool,
    app_data_dir: &Path,
    id: &str,
    input: &UpdateCharacter,
) -> Result<Character, AppError> {
    let character = get(pool, id).await?;
    let updated = character_repo::update(pool, id, input)
        .await
        .map_err(AppError::from)?;
        
    // 如果描述发生了变更，异步触发嵌入
    // 简化处理：只要有描述就重跑（hash 检测会自动跳过未变的情况）
    if let Some(desc) = &updated.description {
        if !desc.trim().is_empty() {
            let pool_bg = pool.clone();
            let dir_bg = app_data_dir.to_path_buf();
            let proj_bg = updated.project_id.clone();
            let id_bg = updated.id.clone();
            let desc_bg = desc.clone();
            tokio::spawn(async move {
                if let Err(e) = embedding_service::generate_and_store(
                    &pool_bg, &dir_bg, &proj_bg, "character", &id_bg, &desc_bg, "",
                ).await {
                    tracing::warn!("Failed to embed updated character {}: {}", id_bg, e);
                }
            });
        }
    }
    Ok(updated)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    // 清理关联向量（embeddings）
    if let Err(e) = embedding_repo::delete_by_source(pool, EmbeddingSourceType::Character, id).await {
        tracing::warn!("Failed to clean character embeddings {}: {}", id, e);
    }
    // 清理关联 entity_snapshots
    if let Err(e) = entity_snapshot_repo::delete_by_entity(pool, EntityType::Character, id).await {
        tracing::warn!("Failed to clean character snapshots {}: {}", id, e);
    }
    // 清理所有章节对该角色的引用
    if let Err(e) = chapter_element_repo::remove_by_element(pool, "character", id).await {
        tracing::warn!("Failed to clean chapter elements for character {}: {}", id, e);
    }
    character_repo::delete(pool, id).await.map_err(AppError::from)
}
