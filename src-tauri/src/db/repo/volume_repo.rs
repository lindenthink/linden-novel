use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::volume::{CreateVolume, UpdateVolume, Volume};

pub async fn list_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Volume>, sqlx::Error> {
    sqlx::query_as::<_, Volume>(
        "SELECT * FROM volumes WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Volume>, sqlx::Error> {
    sqlx::query_as::<_, Volume>("SELECT * FROM volumes WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, input: &CreateVolume) -> Result<Volume, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    // 取当前最大 order_index + 1
    let max_order: Option<i32> =
        sqlx::query_scalar("SELECT MAX(order_index) FROM volumes WHERE project_id = ?")
            .bind(&input.project_id)
            .fetch_one(pool)
            .await?;
    let order_index = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, Volume>(
        "INSERT INTO volumes (id, project_id, title, order_index, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.title)
    .bind(order_index)
    .bind(&ts)
    .bind(&ts)
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &SqlitePool, id: &str, input: &UpdateVolume) -> Result<Volume, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Volume>(
        "UPDATE volumes SET title = COALESCE(?, title), updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.title)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM volumes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 按 id 列表顺序批量重写 order_index（事务保证一致性）
pub async fn reorder(pool: &SqlitePool, volume_ids: &[String]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (i, id) in volume_ids.iter().enumerate() {
        sqlx::query("UPDATE volumes SET order_index = ? WHERE id = ?")
            .bind(i as i32)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
