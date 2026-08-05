use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::services::settings_service;

#[tauri::command]
pub async fn get_setting(
    pool: State<'_, SqlitePool>,
    key: String,
) -> Result<Option<String>, AppError> {
    settings_service::get(&pool, &key).await
}

#[tauri::command]
pub async fn set_setting(
    pool: State<'_, SqlitePool>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    settings_service::set(&pool, &key, &value).await
}
