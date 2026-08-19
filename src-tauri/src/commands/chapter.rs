use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::chapter::{Chapter, CreateChapter, UpdateChapterMeta};
use crate::models::content::ChapterContent;
use crate::services::chapter_service;

#[tauri::command]
pub async fn list_chapters(
    pool: State<'_, SqlitePool>,
    volume_id: String,
) -> Result<Vec<Chapter>, AppError> {
    chapter_service::list_by_volume(&pool, &volume_id).await
}

/// 一次性查询项目下所有章节（按卷顺序 + 章节顺序排序）
/// 用于编辑器首次加载，替代多次 list_chapters 调用
#[tauri::command]
pub async fn list_chapters_by_project(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Chapter>, AppError> {
    chapter_service::list_by_project(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_chapter(pool: State<'_, SqlitePool>, id: String) -> Result<Chapter, AppError> {
    chapter_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_chapter(
    pool: State<'_, SqlitePool>,
    input: CreateChapter,
) -> Result<Chapter, AppError> {
    chapter_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_chapter_meta(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateChapterMeta,
) -> Result<Chapter, AppError> {
    chapter_service::update_meta(&pool, &id, &input).await
}

#[tauri::command]
pub async fn delete_chapter(pool: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    chapter_service::delete(&pool, &id).await
}

#[tauri::command]
pub async fn reorder_chapters(
    pool: State<'_, SqlitePool>,
    chapter_ids: Vec<String>,
) -> Result<(), AppError> {
    chapter_service::reorder(&pool, &chapter_ids).await
}

#[tauri::command]
pub async fn get_chapter_content(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
) -> Result<ChapterContent, AppError> {
    chapter_service::get_content(&pool, &chapter_id).await
}

/// 保存正文，返回权威 word_count
#[tauri::command]
pub async fn save_chapter_content(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
    content_json: String,
    content_text: String,
) -> Result<i64, AppError> {
    chapter_service::save_content(&pool, &chapter_id, &content_json, &content_text).await
}
