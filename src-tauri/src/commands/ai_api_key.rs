use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::ai_api_key::{AiApiKey, CreateAiApiKey};
use crate::services::ai_api_key_service;

#[derive(Clone, Serialize)]
pub struct AiStreamEvent {
    pub chunk: String,
    pub done: bool,
}

#[tauri::command]
pub async fn list_ai_api_keys(
    pool: State<'_, SqlitePool>,
    provider_id: String,
) -> Result<Vec<AiApiKey>, AppError> {
    ai_api_key_service::list_by_provider(pool.inner(), &provider_id).await
}

#[tauri::command]
pub async fn create_ai_api_key(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: CreateAiApiKey,
) -> Result<AiApiKey, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    
    ai_api_key_service::create(pool.inner(), &app_data_dir, &input).await
}

#[tauri::command]
pub async fn delete_ai_api_key(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    ai_api_key_service::delete(pool.inner(), &id).await
}

#[tauri::command]
pub async fn set_default_ai_api_key(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    ai_api_key_service::set_default(pool.inner(), &id).await
}
