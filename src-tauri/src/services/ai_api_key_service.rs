use sqlx::SqlitePool;
use std::path::Path;

use crate::crypto;
use crate::db::repo::ai_api_key_repo;
use crate::error::AppError;
use crate::models::ai_api_key::{AiApiKey, CreateAiApiKey};

pub async fn list_by_provider(
    pool: &SqlitePool,
    provider_id: &str,
) -> Result<Vec<AiApiKey>, AppError> {
    ai_api_key_repo::list_by_provider(pool, provider_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<AiApiKey, AppError> {
    ai_api_key_repo::get(pool, id).await.map_err(AppError::from)
}

pub async fn create(
    pool: &SqlitePool,
    app_data_dir: &Path,
    input: &CreateAiApiKey,
) -> Result<AiApiKey, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Key name must not be empty".into()));
    }
    if input.api_key.trim().is_empty() {
        return Err(AppError::Validation("API key must not be empty".into()));
    }

    // 加密 API key
    let encrypted_key = crypto::encrypt(&input.api_key, app_data_dir)?;

    // 如果设置为默认，先清除该 provider 下的其他默认
    if input.is_default.unwrap_or(false) {
        clear_default_for_provider(pool, &input.provider_id).await?;
    }

    ai_api_key_repo::create(pool, input, &encrypted_key)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    ai_api_key_repo::delete(pool, id).await.map_err(AppError::from)
}

pub async fn get_decrypted(
    pool: &SqlitePool,
    app_data_dir: &Path,
    id: &str,
) -> Result<String, AppError> {
    let key = get(pool, id).await?;
    crypto::decrypt(&key.encrypted_key, app_data_dir)
}

pub async fn get_default_for_provider(
    pool: &SqlitePool,
    provider_id: &str,
) -> Result<Option<AiApiKey>, AppError> {
    ai_api_key_repo::get_default_for_provider(pool, provider_id)
        .await
        .map_err(AppError::from)
}

pub async fn set_default(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let key = get(pool, id).await?;
    clear_default_for_provider(pool, &key.provider_id).await?;
    ai_api_key_repo::set_default(pool, id).await.map_err(AppError::from)
}

async fn clear_default_for_provider(pool: &SqlitePool, provider_id: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE ai_api_keys SET is_default = 0 WHERE provider_id = ? AND is_default = 1")
        .bind(provider_id)
        .execute(pool)
        .await?;
    Ok(())
}
