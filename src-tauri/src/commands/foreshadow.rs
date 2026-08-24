use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::foreshadow::{CreateForeshadow, Foreshadow, UpdateForeshadow};
use crate::services::foreshadow_service;

#[tauri::command]
pub async fn list_foreshadows(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Foreshadow>, AppError> {
    foreshadow_service::list(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_foreshadow(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<Foreshadow, AppError> {
    foreshadow_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_foreshadow(
    pool: State<'_, SqlitePool>,
    input: CreateForeshadow,
) -> Result<Foreshadow, AppError> {
    foreshadow_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_foreshadow(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateForeshadow,
) -> Result<Foreshadow, AppError> {
    foreshadow_service::update(&pool, &id, &input).await
}

#[tauri::command]
pub async fn delete_foreshadow(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    foreshadow_service::delete(&pool, &id).await
}
