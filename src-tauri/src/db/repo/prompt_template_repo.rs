use crate::db::pool::now;
use crate::error::AppError;
use crate::models::prompt_template::{CreatePromptTemplate, PromptTemplate, UpdatePromptTemplate};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn list(pool: &SqlitePool) -> Result<Vec<PromptTemplate>, AppError> {
    let templates = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, is_builtin, created_at, updated_at
         FROM prompt_templates ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(templates)
}

pub async fn list_by_type(
    pool: &SqlitePool,
    template_type: &str,
) -> Result<Vec<PromptTemplate>, AppError> {
    let templates = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, is_builtin, created_at, updated_at
         FROM prompt_templates WHERE template_type = ? ORDER BY is_builtin DESC, created_at DESC"
    )
    .bind(template_type)
    .fetch_all(pool)
    .await?;

    Ok(templates)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<PromptTemplate, AppError> {
    let template = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, is_builtin, created_at, updated_at
         FROM prompt_templates WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Prompt template {} not found", id)))?;

    Ok(template)
}

/// 按类型取第一条（内置优先），找不到时返回 None
pub async fn get_first_by_type(
    pool: &SqlitePool,
    template_type: &str,
) -> Result<Option<PromptTemplate>, AppError> {
    let template = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, is_builtin, created_at, updated_at
         FROM prompt_templates WHERE template_type = ?
         ORDER BY is_builtin DESC, created_at ASC LIMIT 1"
    )
    .bind(template_type)
    .fetch_optional(pool)
    .await?;

    Ok(template)
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    let id = input
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = now();
    let is_builtin = input.is_builtin.unwrap_or(false);

    sqlx::query_as::<_, PromptTemplate>(
        "INSERT INTO prompt_templates (id, name, template_type, content, variables_json, description, is_builtin, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.template_type)
    .bind(&input.content)
    .bind(&input.variables_json)
    .bind(&input.description)
    .bind(is_builtin)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    get(pool, &id).await
}

pub async fn upsert(
    pool: &SqlitePool,
    input: &CreatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    let id = input
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = now();
    let is_builtin = input.is_builtin.unwrap_or(false);

    // SQLite 无原生 ON CONFLICT 可在列约束无 PRIMARY KEY/UNIQUE 时工作；
    // 这里先按 id 判断存在性，再 insert 或 update。
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM prompt_templates WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(pool)
    .await
    .unwrap_or(0) > 0;

    if exists {
        sqlx::query(
            "UPDATE prompt_templates SET name = ?, template_type = ?, content = ?, variables_json = ?, description = ?, is_builtin = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&input.name)
        .bind(&input.template_type)
        .bind(&input.content)
        .bind(&input.variables_json)
        .bind(&input.description)
        .bind(is_builtin)
        .bind(&now)
        .bind(&id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO prompt_templates (id, name, template_type, content, variables_json, description, is_builtin, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.template_type)
        .bind(&input.content)
        .bind(&input.variables_json)
        .bind(&input.description)
        .bind(is_builtin)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    get(pool, &id).await
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    let existing = get(pool, id).await?;
    let now = now();

    let name = input.name.as_deref().unwrap_or(&existing.name);
    let template_type = input.template_type.as_deref().unwrap_or(&existing.template_type);
    let content = input.content.as_deref().unwrap_or(&existing.content);
    let variables_json = input.variables_json.as_deref().or(existing.variables_json.as_deref());
    let description = input.description.as_deref().or(existing.description.as_deref());
    let is_builtin = input.is_builtin.unwrap_or(existing.is_builtin);

    sqlx::query_as::<_, PromptTemplate>(
        "UPDATE prompt_templates SET name = ?, template_type = ?, content = ?, variables_json = ?, description = ?, is_builtin = ?, updated_at = ?
         WHERE id = ? RETURNING *"
    )
    .bind(name)
    .bind(template_type)
    .bind(content)
    .bind(variables_json)
    .bind(description)
    .bind(is_builtin)
    .bind(&now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM prompt_templates WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Prompt template {} not found", id)));
    }

    Ok(())
}
