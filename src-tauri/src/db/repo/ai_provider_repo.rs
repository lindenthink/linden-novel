use crate::error::AppError;
use crate::models::ai_provider::{AiProvider, CreateAiProvider, UpdateAiProvider};
use crate::db::pool::now;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn list(pool: &SqlitePool) -> Result<Vec<AiProvider>, AppError> {
    let providers = sqlx::query_as::<_, AiProvider>(
        "SELECT id, name, provider_type, base_url, models_json, is_default, created_at, updated_at FROM ai_providers ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(providers)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<AiProvider, AppError> {
    let provider = sqlx::query_as::<_, AiProvider>(
        "SELECT id, name, provider_type, base_url, models_json, is_default, created_at, updated_at FROM ai_providers WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Provider {} not found", id)))?;

    Ok(provider)
}

pub async fn create(pool: &SqlitePool, input: &CreateAiProvider) -> Result<AiProvider, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now();
    let is_default = input.is_default.unwrap_or(false);

    sqlx::query_as::<_, AiProvider>(
        "INSERT INTO ai_providers (id, name, provider_type, base_url, models_json, is_default, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.provider_type)
    .bind(&input.base_url)
    .bind(&input.models_json)
    .bind(is_default)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: &UpdateAiProvider) -> Result<AiProvider, AppError> {
    let existing = get(pool, id).await?;
    let now = now();

    let name = input.name.as_deref().unwrap_or(&existing.name);
    let provider_type = input.provider_type.as_deref().unwrap_or(&existing.provider_type);
    let base_url = input.base_url.as_deref().unwrap_or(&existing.base_url);
    let models_json = input.models_json.as_deref().unwrap_or(&existing.models_json);
    let is_default = input.is_default.unwrap_or(existing.is_default);

    sqlx::query_as::<_, AiProvider>(
        "UPDATE ai_providers SET name = ?, provider_type = ?, base_url = ?, models_json = ?, is_default = ?, updated_at = ?
         WHERE id = ? RETURNING *"
    )
    .bind(name)
    .bind(provider_type)
    .bind(base_url)
    .bind(models_json)
    .bind(is_default)
    .bind(&now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM ai_providers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Provider {} not found", id)));
    }

    Ok(())
}

pub async fn get_default(pool: &SqlitePool) -> Result<Option<AiProvider>, AppError> {
    let provider = sqlx::query_as::<_, AiProvider>(
        "SELECT id, name, provider_type, base_url, models_json, is_default, created_at, updated_at FROM ai_providers WHERE is_default = 1 LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(provider)
}
