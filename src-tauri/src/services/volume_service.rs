use sqlx::SqlitePool;

use crate::db::repo::{embedding_chunk_repo, embedding_repo, volume_repo};
use crate::error::AppError;
use crate::models::volume::{CreateVolume, UpdateVolume, Volume};

pub async fn list_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Volume>, AppError> {
    volume_repo::list_by_project(pool, project_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Volume, AppError> {
    volume_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Volume '{id}' not found")))
}

pub async fn create(pool: &SqlitePool, input: &CreateVolume) -> Result<Volume, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title must not be empty".into()));
    }
    volume_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(pool: &SqlitePool, id: &str, input: &UpdateVolume) -> Result<Volume, AppError> {
    get(pool, id).await?;
    volume_repo::update(pool, id, input).await.map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    // 先清理该卷下所有章节的向量（依赖 chapters 表存在，必须在级联删除前执行）
    // 失败仅告警：向量清理不应阻塞业务删除，残留向量可由后续 sync 重建时清理
    if let Err(e) = embedding_chunk_repo::delete_by_volume(pool, id).await {
        tracing::warn!("Failed to clean chunk embeddings for volume {}: {}", id, e);
    }
    if let Err(e) = embedding_repo::delete_chapters_by_volume(pool, id).await {
        tracing::warn!("Failed to clean summary embeddings for volume {}: {}", id, e);
    }
    volume_repo::delete(pool, id).await.map_err(AppError::from)
}

pub async fn reorder(pool: &SqlitePool, volume_ids: &[String]) -> Result<(), AppError> {
    volume_repo::reorder(pool, volume_ids)
        .await
        .map_err(AppError::from)
}
