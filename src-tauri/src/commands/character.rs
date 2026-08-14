use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::character::{Character, CreateCharacter, UpdateCharacter};
use crate::services::character_service;

#[tauri::command]
pub async fn list_characters(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Character>, AppError> {
    character_service::list(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_character(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<Character, AppError> {
    character_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_character(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    input: CreateCharacter,
) -> Result<Character, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    character_service::create(&pool, &app_data_dir, &input).await
}

#[tauri::command]
pub async fn update_character(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateCharacter,
) -> Result<Character, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    character_service::update(&pool, &app_data_dir, &id, &input).await
}

#[tauri::command]
pub async fn delete_character(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    character_service::delete(&pool, &id).await
}
