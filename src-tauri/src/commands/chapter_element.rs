use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::chapter_element::{ChapterElement, CreateChapterElement};
use crate::services::chapter_element_service;

#[tauri::command]
pub async fn list_chapter_elements(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
) -> Result<Vec<ChapterElement>, AppError> {
    chapter_element_service::list_by_chapter(&pool, &chapter_id).await
}

#[tauri::command]
pub async fn add_chapter_element(
    pool: State<'_, SqlitePool>,
    input: CreateChapterElement,
) -> Result<ChapterElement, AppError> {
    chapter_element_service::add(&pool, &input).await
}

#[tauri::command]
pub async fn remove_chapter_element(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    chapter_element_service::remove(&pool, &id).await
}

#[tauri::command]
pub async fn remove_chapter_element_by_ref(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
    element_type: String,
    element_id: String,
) -> Result<(), AppError> {
    chapter_element_service::remove_by_chapter_and_element(
        &pool,
        &chapter_id,
        &element_type,
        &element_id,
    )
    .await
}
