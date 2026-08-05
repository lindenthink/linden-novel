use crate::error::AppError;
use crate::models::prompt_template::{CreatePromptTemplate, PromptTemplate, UpdatePromptTemplate};
use crate::db::pool::now;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn list(pool: &SqlitePool) -> Result<Vec<PromptTemplate>, AppError> {
    let templates = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, 
                is_builtin, created_at, updated_at 
         FROM prompt_templates ORDER BY is_builtin DESC, created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(templates)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<PromptTemplate, AppError> {
    let template = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, 
                is_builtin, created_at, updated_at 
         FROM prompt_templates WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;

    Ok(template)
}

pub async fn list_by_type(pool: &SqlitePool, template_type: &str) -> Result<Vec<PromptTemplate>, AppError> {
    let templates = sqlx::query_as::<_, PromptTemplate>(
        "SELECT id, name, template_type, content, variables_json, description, 
                is_builtin, created_at, updated_at 
         FROM prompt_templates WHERE template_type = ? ORDER BY is_builtin DESC, created_at DESC"
    )
    .bind(template_type)
    .fetch_all(pool)
    .await?;

    Ok(templates)
}

pub async fn create(pool: &SqlitePool, input: &CreatePromptTemplate) -> Result<PromptTemplate, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = now();

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
    .bind(false)
    .bind(&now)
    .bind(&now)
    .fetch_one(pool)
    .await?;

    get(pool, &id).await
}

pub async fn update(pool: &SqlitePool, id: &str, input: &UpdatePromptTemplate) -> Result<PromptTemplate, AppError> {
    let existing = get(pool, id).await?;
    
    // 内置模板不允许修改
    if existing.is_builtin {
        return Err(AppError::Validation("Cannot modify builtin template".to_string()));
    }
    
    let now = now();

    let name = input.name.as_deref().unwrap_or(&existing.name);
    let template_type = input.template_type.as_deref().unwrap_or(&existing.template_type);
    let content = input.content.as_deref().unwrap_or(&existing.content);
    let variables_json = input.variables_json.as_deref().or(existing.variables_json.as_deref());
    let description = input.description.as_deref().or(existing.description.as_deref());

    sqlx::query_as::<_, PromptTemplate>(
        "UPDATE prompt_templates 
         SET name = ?, template_type = ?, content = ?, variables_json = ?, description = ?, updated_at = ?
         WHERE id = ? RETURNING *"
    )
    .bind(name)
    .bind(template_type)
    .bind(content)
    .bind(variables_json)
    .bind(description)
    .bind(&now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let existing = get(pool, id).await?;
    
    // 内置模板不允许删除
    if existing.is_builtin {
        return Err(AppError::Validation("Cannot delete builtin template".to_string()));
    }
    
    let result = sqlx::query("DELETE FROM prompt_templates WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Template {} not found", id)));
    }

    Ok(())
}
