use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::storyline::{Storyline, CreateStoryline, UpdateStoryline};
use crate::services::storyline_service;

#[tauri::command]
pub async fn list_storylines(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Storyline>, AppError> {
    storyline_service::list(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_storyline(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<Storyline, AppError> {
    storyline_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_storyline(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: CreateStoryline,
) -> Result<Storyline, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    storyline_service::create(&pool, &app_data_dir, &input).await
}

#[tauri::command]
pub async fn update_storyline(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateStoryline,
) -> Result<Storyline, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    storyline_service::update(&pool, &app_data_dir, &id, &input).await
}

#[tauri::command]
pub async fn delete_storyline(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    storyline_service::delete(&pool, &id).await
}
