use sqlx::SqlitePool;

use crate::db::pool;
use crate::models::project::{CreateProject, Project, UpdateProject};

pub async fn list(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, input: &CreateProject) -> Result<Project, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = pool::now();
    sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, title, genre, summary, target_words, settings_json, cover_path, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?) RETURNING *",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.genre)
    .bind(&input.summary)
    .bind(input.target_words)
    .bind(&input.cover_path)
    .bind(&ts)
    .bind(&ts)
    .fetch_one(pool)
    .await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateProject,
) -> Result<Project, sqlx::Error> {
    let ts = pool::now();
    sqlx::query_as::<_, Project>(
        "UPDATE projects SET
            title = COALESCE(?, title),
            genre = COALESCE(?, genre),
            summary = COALESCE(?, summary),
            target_words = COALESCE(?, target_words),
            settings_json = COALESCE(?, settings_json),
            cover_path = CASE
                WHEN ? IS NULL THEN cover_path
                WHEN ? = '' THEN NULL
                ELSE ?
            END,
            updated_at = ?
         WHERE id = ? RETURNING *",
    )
    .bind(&input.title)
    .bind(&input.genre)
    .bind(&input.summary)
    .bind(input.target_words)
    .bind(&input.settings_json)
    .bind(&input.cover_path)
    .bind(&input.cover_path)
    .bind(&input.cover_path)
    .bind(&ts)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
