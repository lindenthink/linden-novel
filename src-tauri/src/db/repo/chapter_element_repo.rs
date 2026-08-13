use sqlx::SqlitePool;

use crate::models::chapter_element::{ChapterElement, CreateChapterElement};

pub async fn list_by_chapter(pool: &SqlitePool, chapter_id: &str) -> Result<Vec<ChapterElement>, sqlx::Error> {
    sqlx::query_as::<_, ChapterElement>(
        "SELECT * FROM chapter_elements WHERE chapter_id = ?",
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await
}

pub async fn add(pool: &SqlitePool, input: &CreateChapterElement) -> Result<ChapterElement, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query_as::<_, ChapterElement>(
        "INSERT INTO chapter_elements (id, chapter_id, element_type, element_id)
         VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.chapter_id)
    .bind(&input.element_type)
    .bind(&input.element_id)
    .fetch_one(pool)
    .await
}

pub async fn remove(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chapter_elements WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_by_chapter_and_element(
    pool: &SqlitePool,
    chapter_id: &str,
    element_type: &str,
    element_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chapter_elements WHERE chapter_id = ? AND element_type = ? AND element_id = ?")
        .bind(chapter_id)
        .bind(element_type)
        .bind(element_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 删除所有章节中对该元素的引用（element_type + element_id 批量）
///
/// 用于角色/故事线/世界观被删除时，清理 chapter_elements 表中遗留的无效引用。
pub async fn remove_by_element(
    pool: &SqlitePool,
    element_type: &str,
    element_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chapter_elements WHERE element_type = ? AND element_id = ?")
        .bind(element_type)
        .bind(element_id)
        .execute(pool)
        .await?;
    Ok(())
}
