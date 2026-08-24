use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::foreshadow::{CreateForeshadow, Foreshadow, UpdateForeshadow};

pub async fn list_by_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<Foreshadow>, sqlx::Error> {
    sqlx::query_as::<_, Foreshadow>(
        "SELECT * FROM foreshadows WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Foreshadow>, sqlx::Error> {
    sqlx::query_as::<_, Foreshadow>("SELECT * FROM foreshadows WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// 按 status 过滤（用于查询"未回收"等）
#[allow(dead_code)]
pub async fn list_by_project_and_status(
    pool: &SqlitePool,
    project_id: &str,
    status: &str,
) -> Result<Vec<Foreshadow>, sqlx::Error> {
    sqlx::query_as::<_, Foreshadow>(
        "SELECT * FROM foreshadows WHERE project_id = ? AND status = ? ORDER BY order_index",
    )
    .bind(project_id)
    .bind(status)
    .fetch_all(pool)
    .await
}

/// 查询某章需埋下的伏笔（plant_chapter_id = chapter_id）
pub async fn list_to_plant_in_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<Foreshadow>, sqlx::Error> {
    sqlx::query_as::<_, Foreshadow>(
        "SELECT * FROM foreshadows WHERE plant_chapter_id = ? ORDER BY order_index",
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await
}

/// 查询某章可回收的伏笔（已埋下未回收，或 resolve_chapter_id = chapter_id）
pub async fn list_resolvable_in_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<Foreshadow>, sqlx::Error> {
    sqlx::query_as::<_, Foreshadow>(
        "SELECT * FROM foreshadows
         WHERE (status = 'planted' OR resolve_chapter_id = ?)
         ORDER BY importance DESC, order_index",
    )
    .bind(chapter_id)
    .fetch_all(pool)
    .await
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreateForeshadow,
) -> Result<Foreshadow, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();

    let max_order: Option<i32> =
        sqlx::query_scalar("SELECT MAX(order_index) FROM foreshadows WHERE project_id = ?")
            .bind(&input.project_id)
            .fetch_one(pool)
            .await?;
    let order_index = max_order.unwrap_or(-1) + 1;

    sqlx::query_as::<_, Foreshadow>(
        "INSERT INTO foreshadows
            (id, project_id, title, description, importance, status,
             plant_chapter_id, resolve_chapter_id, plant_note, resolve_note,
             order_index, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.importance.as_deref().unwrap_or("normal"))
    .bind(input.status.as_deref().unwrap_or("planted"))
    .bind(&input.plant_chapter_id)
    .bind(&input.resolve_chapter_id)
    .bind(&input.plant_note)
    .bind(&input.resolve_note)
    .bind(order_index)
    .bind(&ts)
    .bind(&ts)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateForeshadow,
) -> Result<Foreshadow, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Foreshadow>(
        "UPDATE foreshadows SET
            title = COALESCE(?, title),
            description = COALESCE(?, description),
            importance = COALESCE(?, importance),
            status = COALESCE(?, status),
            plant_chapter_id = COALESCE(?, plant_chapter_id),
            resolve_chapter_id = COALESCE(?, resolve_chapter_id),
            plant_note = COALESCE(?, plant_note),
            resolve_note = COALESCE(?, resolve_note),
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.importance)
    .bind(&input.status)
    .bind(&input.plant_chapter_id)
    .bind(&input.resolve_chapter_id)
    .bind(&input.plant_note)
    .bind(&input.resolve_note)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM foreshadows WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 删除项目所有伏笔（项目删除时级联调用）
pub async fn delete_by_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM foreshadows WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
