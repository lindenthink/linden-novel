use sqlx::SqlitePool;

use crate::db::repo::ai_provider_repo;
use crate::error::AppError;
use crate::models::ai_provider::{AiProvider, CreateAiProvider, UpdateAiProvider};

pub async fn list(pool: &SqlitePool) -> Result<Vec<AiProvider>, AppError> {
    ai_provider_repo::list(pool).await.map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<AiProvider, AppError> {
    ai_provider_repo::get(pool, id).await.map_err(AppError::from)
}

pub async fn create(pool: &SqlitePool, input: &CreateAiProvider) -> Result<AiProvider, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Provider name must not be empty".into()));
    }
    if input.base_url.trim().is_empty() {
        return Err(AppError::Validation("Base URL must not be empty".into()));
    }
    if input.models_json.trim().is_empty() {
        return Err(AppError::Validation("Model name must not be empty".into()));
    }
    
    // 如果设置为默认，先清除其他默认
    if input.is_default.unwrap_or(false) {
        clear_default(pool).await?;
    }
    
    ai_provider_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateAiProvider,
) -> Result<AiProvider, AppError> {
    // 确保存在
    get(pool, id).await?;
    
    // 验证 models_json 非空
    if let Some(ref models_json) = input.models_json {
        if models_json.trim().is_empty() {
            return Err(AppError::Validation("Model name must not be empty".into()));
        }
    }
    
    // 如果设置为默认，先清除其他默认
    if input.is_default.unwrap_or(false) {
        clear_default(pool).await?;
    }
    
    ai_provider_repo::update(pool, id, input).await.map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    ai_provider_repo::delete(pool, id).await.map_err(AppError::from)
}

pub async fn get_default(pool: &SqlitePool) -> Result<Option<AiProvider>, AppError> {
    ai_provider_repo::get_default(pool).await.map_err(AppError::from)
}

async fn clear_default(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("UPDATE ai_providers SET is_default = 0 WHERE is_default = 1")
        .execute(pool)
        .await?;
    Ok(())
}
