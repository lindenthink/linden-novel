use sqlx::SqlitePool;

use crate::db::repo::chapter_element_repo;
use crate::error::AppError;
use crate::models::chapter_element::{ChapterElement, CreateChapterElement};

pub async fn list_by_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<ChapterElement>, AppError> {
    chapter_element_repo::list_by_chapter(pool, chapter_id)
        .await
        .map_err(AppError::from)
}

pub async fn add(
    pool: &SqlitePool,
    input: &CreateChapterElement,
) -> Result<ChapterElement, AppError> {
    chapter_element_repo::add(pool, input)
        .await
        .map_err(AppError::from)
}

pub async fn remove(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    chapter_element_repo::remove(pool, id)
        .await
        .map_err(AppError::from)
}

pub async fn remove_by_chapter_and_element(
    pool: &SqlitePool,
    chapter_id: &str,
    element_type: &str,
    element_id: &str,
) -> Result<(), AppError> {
    chapter_element_repo::remove_by_chapter_and_element(pool, chapter_id, element_type, element_id)
        .await
        .map_err(AppError::from)
}
