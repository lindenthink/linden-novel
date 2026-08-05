use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::models::project::{CreateProject, Project, UpdateProject};
use crate::services::project_service;

#[tauri::command]
pub async fn list_projects(pool: State<'_, SqlitePool>) -> Result<Vec<Project>, AppError> {
    project_service::list(&pool).await
}

#[tauri::command]
pub async fn get_project(pool: State<'_, SqlitePool>, id: String) -> Result<Project, AppError> {
    project_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_project(
    pool: State<'_, SqlitePool>,
    input: CreateProject,
) -> Result<Project, AppError> {
    project_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_project(
    pool: State<'_, SqlitePool>,
    id: String,
    input: UpdateProject,
) -> Result<Project, AppError> {
    project_service::update(&pool, &id, &input).await
}

#[tauri::command]
pub async fn delete_project(pool: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    project_service::delete(&pool, &id).await
}
