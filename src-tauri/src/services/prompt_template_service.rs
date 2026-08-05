use sqlx::SqlitePool;

use crate::db::repo::prompt_template_repo;
use crate::error::AppError;
use crate::models::prompt_template::{CreatePromptTemplate, PromptTemplate, UpdatePromptTemplate};

pub async fn list(pool: &SqlitePool) -> Result<Vec<PromptTemplate>, AppError> {
    prompt_template_repo::list(pool).await.map_err(AppError::from)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<PromptTemplate, AppError> {
    prompt_template_repo::get(pool, id).await.map_err(AppError::from)
}

pub async fn list_by_type(
    pool: &SqlitePool,
    template_type: &str,
) -> Result<Vec<PromptTemplate>, AppError> {
    prompt_template_repo::list_by_type(pool, template_type)
        .await
        .map_err(AppError::from)
}

pub async fn create(
    pool: &SqlitePool,
    input: &CreatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Template name must not be empty".into()));
    }
    if input.content.trim().is_empty() {
        return Err(AppError::Validation("Template content must not be empty".into()));
    }

    // 验证 variables_json
    if let Some(ref variables_json) = input.variables_json {
        let _: Vec<String> = serde_json::from_str(variables_json)
            .map_err(|_| AppError::Validation("variables_json must be a valid JSON array".into()))?;
    }

    prompt_template_repo::create(pool, input).await.map_err(AppError::from)
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    input: &UpdatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    // 确保存在
    get(pool, id).await?;

    // 验证 variables_json
    if let Some(ref variables_json) = input.variables_json {
        let _: Vec<String> = serde_json::from_str(variables_json)
            .map_err(|_| AppError::Validation("variables_json must be a valid JSON array".into()))?;
    }

    prompt_template_repo::update(pool, id, input).await.map_err(AppError::from)
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    prompt_template_repo::delete(pool, id).await.map_err(AppError::from)
}

/// 渲染模板，替换变量占位符
pub fn render_template(template: &str, variables: &std::collections::HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}
