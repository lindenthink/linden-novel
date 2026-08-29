use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::inspiration::{CreateInspiration, Inspiration, UpdateInspiration};

/// 项目内灵感列表，按创建时间倒序（最新在最上）
pub async fn list_by_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<Inspiration>, sqlx::Error> {
    sqlx::query_as::<_, Inspiration>(
        "SELECT * FROM inspirations WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Inspiration>, sqlx::Error> {
    sqlx::query_as::<_, Inspiration>("SELECT * FROM inspirations WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateInspiration,
) -> Result<Inspiration, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    sqlx::query_as::<_, Inspiration>(
        "INSERT INTO inspirations (id, project_id, content, tag, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.content)
    .bind(&input.tag)
    .bind(input.status.as_deref().unwrap_or("new"))
    .bind(&ts)
    .bind(&ts)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateInspiration,
) -> Result<Inspiration, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Inspiration>(
        "UPDATE inspirations SET
            content = COALESCE(?, content),
            tag = COALESCE(?, tag),
            status = COALESCE(?, status),
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.content)
    .bind(&input.tag)
    .bind(&input.status)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM inspirations WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 删除项目所有灵感（项目删除时级联调用）
pub async fn delete_by_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM inspirations WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
