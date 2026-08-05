use sqlx::SqlitePool;
use tauri::State;

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
    pool: State<'_, SqlitePool>,
    input: CreateCharacter,
) -> Result<Character, AppError> {
    character_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_character(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateCharacter,
) -> Result<Character, AppError> {
    character_service::update(&pool, &id, &input).await
}

#[tauri::command]
pub async fn delete_character(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    character_service::delete(&pool, &id).await
}
