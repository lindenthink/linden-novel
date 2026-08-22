use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::prompt_template::{CreatePromptTemplate, PromptTemplate, UpdatePromptTemplate};
use crate::services::prompt_template_service;

#[tauri::command]
pub async fn list_prompt_templates(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<PromptTemplate>, AppError> {
    prompt_template_service::list(pool.inner()).await
}

#[tauri::command]
pub async fn list_prompt_templates_by_type(
    pool: State<'_, SqlitePool>,
    template_type: String,
) -> Result<Vec<PromptTemplate>, AppError> {
    prompt_template_service::list_by_type(pool.inner(), &template_type).await
}

#[tauri::command]
pub async fn get_prompt_template(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<PromptTemplate, AppError> {
    prompt_template_service::get(pool.inner(), &id).await
}

#[tauri::command]
pub async fn create_prompt_template(
    pool: State<'_, SqlitePool>,
    input: CreatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    prompt_template_service::create(pool.inner(), &input).await
}

#[tauri::command]
pub async fn update_prompt_template(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdatePromptTemplate,
) -> Result<PromptTemplate, AppError> {
    prompt_template_service::update(pool.inner(), &id, &input).await
}

#[tauri::command]
pub async fn delete_prompt_template(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    prompt_template_service::delete(pool.inner(), &id).await
}

#[tauri::command]
pub async fn reset_prompt_template_builtin(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<PromptTemplate, AppError> {
    prompt_template_service::reset_builtin(pool.inner(), &id).await
}
