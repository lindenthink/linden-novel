use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::storyline::{Storyline, CreateStoryline, UpdateStoryline};

pub async fn list_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Storyline>, sqlx::Error> {
    sqlx::query_as::<_, Storyline>(
        "SELECT * FROM storylines WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Storyline>, sqlx::Error> {
    sqlx::query_as::<_, Storyline>("SELECT * FROM storylines WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, input: &CreateStoryline) -> Result<Storyline, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    let max_order: Option<i32> =
        sqlx::query_scalar("SELECT MAX(order_index) FROM storylines WHERE project_id = ?")
            .bind(&input.project_id)
            .fetch_one(pool)
            .await?;
    let order_index = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, Storyline>(
        "INSERT INTO storylines (id, project_id, name, description, status, order_index, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'active', ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.name)
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
    input: &UpdateStoryline,
) -> Result<Storyline, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Storyline>(
        "UPDATE storylines SET
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            status = COALESCE(?, status),
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.status)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM storylines WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
