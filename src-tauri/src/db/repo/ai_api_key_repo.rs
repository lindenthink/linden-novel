use crate::error::AppError;
use crate::models::ai_api_key::{AiApiKey, CreateAiApiKey};
use crate::db::pool::now;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn list_by_provider(pool: &SqlitePool, provider_id: &str) -> Result<Vec<AiApiKey>, AppError> {
    let keys = sqlx::query_as::<_, AiApiKey>(
        "SELECT id, provider_id, name, encrypted_key, is_default, created_at 
         FROM ai_api_keys WHERE provider_id = ? ORDER BY created_at DESC"
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await?;

    Ok(keys)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<AiApiKey, AppError> {
    let key = sqlx::query_as::<_, AiApiKey>(
        "SELECT id, provider_id, name, encrypted_key, is_default, created_at 
         FROM ai_api_keys WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("API Key {} not found", id)))?;

    Ok(key)
}

pub async fn create(pool: &SqlitePool, input: &CreateAiApiKey, encrypted_key: &str) -> Result<AiApiKey, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now();
    let is_default = input.is_default.unwrap_or(false);

    sqlx::query_as::<_, AiApiKey>(
        "INSERT INTO ai_api_keys (id, provider_id, name, encrypted_key, is_default, created_at)
         VALUES (?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(&id)
    .bind(&input.provider_id)
    .bind(&input.name)
    .bind(encrypted_key)
    .bind(is_default)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    get(pool, &id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM ai_api_keys WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("API Key {} not found", id)));
    }

    Ok(())
}

pub async fn get_default_for_provider(pool: &SqlitePool, provider_id: &str) -> Result<Option<AiApiKey>, AppError> {
    let key = sqlx::query_as::<_, AiApiKey>(
        "SELECT id, provider_id, name, encrypted_key, is_default, created_at 
         FROM ai_api_keys WHERE provider_id = ? AND is_default = 1 LIMIT 1"
    )
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    Ok(key)
}

pub async fn set_default(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let key = get(pool, id).await?;
    
    // 先取消该 provider 下所有 key 的默认状态
    sqlx::query("UPDATE ai_api_keys SET is_default = 0 WHERE provider_id = ?")
        .bind(&key.provider_id)
        .execute(pool)
        .await?;
    
    // 设置指定 key 为默认
    sqlx::query("UPDATE ai_api_keys SET is_default = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(())
}
