use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::ai_provider::{AiProvider, CreateAiProvider, UpdateAiProvider};
use crate::services::ai_provider_service;

#[tauri::command]
pub async fn list_ai_providers(pool: State<'_, SqlitePool>) -> Result<Vec<AiProvider>, AppError> {
    ai_provider_service::list(pool.inner()).await
}

#[tauri::command]
pub async fn get_ai_provider(pool: State<'_, SqlitePool>, id: String) -> Result<AiProvider, AppError> {
    ai_provider_service::get(pool.inner(), &id).await
}

#[tauri::command]
pub async fn create_ai_provider(
    pool: State<'_, SqlitePool>,
    input: CreateAiProvider,
) -> Result<AiProvider, AppError> {
    ai_provider_service::create(pool.inner(), &input).await
}

#[tauri::command]
pub async fn update_ai_provider(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateAiProvider,
) -> Result<AiProvider, AppError> {
    ai_provider_service::update(pool.inner(), &id, &input).await
}

#[tauri::command]
pub async fn delete_ai_provider(pool: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    ai_provider_service::delete(pool.inner(), &id).await
}

#[tauri::command]
pub async fn get_default_ai_provider(
    pool: State<'_, SqlitePool>,
) -> Result<Option<AiProvider>, AppError> {
    ai_provider_service::get_default(pool.inner()).await
}
