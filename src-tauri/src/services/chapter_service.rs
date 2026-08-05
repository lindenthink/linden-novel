use sqlx::SqlitePool;

use crate::db::repo::{chapter_repo, content_repo};
use crate::error::AppError;
use crate::models::chapter::{Chapter, CreateChapter, UpdateChapterMeta};
use crate::models::content::ChapterContent;

pub async fn list_by_volume(pool: &SqlitePool, volume_id: &str) -> Result<Vec<Chapter>, AppError> {
    chapter_repo::list_by_volume(pool, volume_id)
        .await
        .map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Chapter, AppError> {
    chapter_repo::get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Chapter '{id}' not found")))
}

pub async fn create(pool: &SqlitePool, input: &CreateChapter) -> Result<Chapter, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title must not be empty".into()));
    }
    let chapter = chapter_repo::create(pool, input).await?;
    // 同时创建空的 chapter_content
    let ts = crate::db::pool::now();
    sqlx::query(
        "INSERT INTO chapter_contents (chapter_id, content_json, content_text, updated_at)
         VALUES (?, '{}', '', ?)",
    )
    .bind(&chapter.id)
    .bind(&ts)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(chapter)
}

pub async fn update_meta(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateChapterMeta,
) -> Result<Chapter, AppError> {
    get(pool, id).await?;
    chapter_repo::update_meta(pool, id, input)
        .await
        .map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    chapter_repo::delete(pool, id).await.map_err(AppError::from)
}

pub async fn reorder(pool: &SqlitePool, chapter_ids: &[String]) -> Result<(), AppError> {
    chapter_repo::reorder(pool, chapter_ids)
        .await
        .map_err(AppError::from)
}

// ---- 正文操作 ----

pub async fn get_content(pool: &SqlitePool, chapter_id: &str) -> Result<ChapterContent, AppError> {
    content_repo::get(pool, chapter_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Content for chapter '{chapter_id}' not found")))
}

/// 保存正文，Rust 权威计算 word_count 并返回
pub async fn save_content(
    pool: &SqlitePool,
    chapter_id: &str,
    content_json: &str,
    content_text: &str,
) -> Result<i64, AppError> {
    let input = crate::models::content::SaveContent {
        chapter_id: chapter_id.into(),
        content_json: content_json.into(),
        content_text: content_text.into(),
    };
    content_repo::save(pool, &input).await.map_err(AppError::from)
}
