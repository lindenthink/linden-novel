use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::worldview::{WorldviewEntry, CreateWorldviewEntry, UpdateWorldviewEntry};
use crate::services::worldview_service;

#[tauri::command]
pub async fn list_worldview(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<WorldviewEntry>, AppError> {
    worldview_service::list(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_worldview(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<WorldviewEntry, AppError> {
    worldview_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_worldview(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: CreateWorldviewEntry,
) -> Result<WorldviewEntry, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    worldview_service::create(&pool, &app_data_dir, &input).await
}

#[tauri::command]
pub async fn update_worldview(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateWorldviewEntry,
) -> Result<WorldviewEntry, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    worldview_service::update(&pool, &app_data_dir, &id, &input).await
}

#[tauri::command]
pub async fn delete_worldview(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    worldview_service::delete(&pool, &id).await
}
