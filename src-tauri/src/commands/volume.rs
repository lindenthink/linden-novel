use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::volume::{CreateVolume, UpdateVolume, Volume};
use crate::services::volume_service;

#[tauri::command]
pub async fn list_volumes(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Volume>, AppError> {
    volume_service::list_by_project(&pool, &project_id).await
}

#[tauri::command]
pub async fn create_volume(
    pool: State<'_, SqlitePool>,
    input: CreateVolume,
) -> Result<Volume, AppError> {
    volume_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_volume(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateVolume,
) -> Result<Volume, AppError> {
    volume_service::update(&pool, &id, &input).await
}

#[tauri::command]
pub async fn delete_volume(pool: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    volume_service::delete(&pool, &id).await
}

#[tauri::command]
pub async fn reorder_volumes(
    pool: State<'_, SqlitePool>,
    volume_ids: Vec<String>,
) -> Result<(), AppError> {
    volume_service::reorder(&pool, &volume_ids).await
}
