use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::inspiration::{CreateInspiration, Inspiration, UpdateInspiration};
use crate::services::inspiration_service;

#[tauri::command]
pub async fn list_inspirations(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Inspiration>, AppError> {
    inspiration_service::list(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_inspiration(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<Inspiration, AppError> {
    inspiration_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_inspiration(
    pool: State<'_, SqlitePool>,
    input: CreateInspiration,
) -> Result<Inspiration, AppError> {
    inspiration_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_inspiration(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateInspiration,
) -> Result<Inspiration, AppError> {
    inspiration_service::update(&pool, &id, &input).await
}

#[tauri::command]
pub async fn delete_inspiration(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    inspiration_service::delete(&pool, &id).await
}
