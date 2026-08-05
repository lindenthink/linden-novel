use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::worldview::{WorldviewEntry, CreateWorldviewEntry, UpdateWorldviewEntry};

pub async fn list_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<WorldviewEntry>, sqlx::Error> {
    sqlx::query_as::<_, WorldviewEntry>(
        "SELECT * FROM worldview WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<WorldviewEntry>, sqlx::Error> {
    sqlx::query_as::<_, WorldviewEntry>("SELECT * FROM worldview WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, input: &CreateWorldviewEntry) -> Result<WorldviewEntry, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    let max_order: Option<i32> =
        sqlx::query_scalar("SELECT MAX(order_index) FROM worldview WHERE project_id = ?")
            .bind(&input.project_id)
            .fetch_one(pool)
            .await?;
    let order_index = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, WorldviewEntry>(
        "INSERT INTO worldview (id, project_id, name, category, description, order_index, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.name)
    .bind(&input.category)
    .bind(&input.description)
    .bind(order_index)
    .bind(&ts)
    .bind(&ts)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateWorldviewEntry,
) -> Result<WorldviewEntry, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, WorldviewEntry>(
        "UPDATE worldview SET
            name = COALESCE(?, name),
            category = COALESCE(?, category),
            description = COALESCE(?, description),
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.category)
    .bind(&input.description)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM worldview WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
