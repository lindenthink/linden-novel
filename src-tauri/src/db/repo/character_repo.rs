use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::character::{Character, CreateCharacter, UpdateCharacter};

pub async fn list_by_project(pool: &SqlitePool, project_id: &str) -> Result<Vec<Character>, sqlx::Error> {
    sqlx::query_as::<_, Character>(
        "SELECT * FROM characters WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Character>, sqlx::Error> {
    sqlx::query_as::<_, Character>("SELECT * FROM characters WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, input: &CreateCharacter) -> Result<Character, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    let max_order: Option<i32> =
        sqlx::query_scalar("SELECT MAX(order_index) FROM characters WHERE project_id = ?")
            .bind(&input.project_id)
            .fetch_one(pool)
            .await?;
    let order_index = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, Character>(
        "INSERT INTO characters (id, project_id, name, role, description, avatar, order_index, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.name)
    .bind(&input.role)
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
    input: &UpdateCharacter,
) -> Result<Character, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Character>(
        "UPDATE characters SET
            name = COALESCE(?, name),
            role = COALESCE(?, role),
            description = COALESCE(?, description),
            avatar = COALESCE(?, avatar),
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.name)
    .bind(&input.role)
    .bind(&input.description)
    .bind(&input.avatar)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM characters WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
