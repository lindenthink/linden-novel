use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::chapter::{CreateChapter, Chapter, UpdateChapterMeta};

pub async fn list_by_volume(pool: &SqlitePool, volume_id: &str) -> Result<Vec<Chapter>, sqlx::Error> {
    sqlx::query_as::<_, Chapter>(
        "SELECT * FROM chapters WHERE volume_id = ? ORDER BY order_index",
    )
    .bind(volume_id)
    .fetch_all(pool)
    .await
}

/// 一次性查询项目下所有章节，按卷顺序 + 章节顺序排序
/// 用于编辑器首次加载，避免 N 次 IPC（每卷一次）
pub async fn list_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Chapter>, sqlx::Error> {
    sqlx::query_as::<_, Chapter>(
        "SELECT c.*
         FROM chapters c
         JOIN volumes v ON v.id = c.volume_id
         WHERE c.project_id = ?
         ORDER BY v.order_index ASC, c.order_index ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Chapter>, sqlx::Error> {
    sqlx::query_as::<_, Chapter>("SELECT * FROM chapters WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, input: &CreateChapter) -> Result<Chapter, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    let max_order: Option<i32> =
        sqlx::query_scalar("SELECT MAX(order_index) FROM chapters WHERE volume_id = ?")
            .bind(&input.volume_id)
            .fetch_one(pool)
            .await?;
    let order_index = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, Chapter>(
        "INSERT INTO chapters (id, volume_id, project_id, title, order_index, status, word_count, summary, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'draft', 0, NULL, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.volume_id)
    .bind(&input.project_id)
    .bind(&input.title)
    .bind(order_index)
    .bind(&ts)
    .bind(&ts)
    .fetch_one(pool)
    .await
}

pub async fn update_meta(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateChapterMeta,
) -> Result<Chapter, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Chapter>(
        "UPDATE chapters SET
            title = COALESCE(?, title),
            status = COALESCE(?, status),
            summary = COALESCE(?, summary),
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.status)
    .bind(&input.summary)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chapters WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 按 id 列表顺序批量重写 order_index
pub async fn reorder(pool: &SqlitePool, chapter_ids: &[String]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (i, id) in chapter_ids.iter().enumerate() {
        sqlx::query("UPDATE chapters SET order_index = ? WHERE id = ?")
            .bind(i as i32)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
