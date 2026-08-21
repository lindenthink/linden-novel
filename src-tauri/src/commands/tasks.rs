use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::async_task::{AsyncTask, NewTaskRequest};
use crate::services::task_manager::TaskManagerState;

#[tauri::command]
pub async fn submit_task(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    task_manager_state: State<'_, TaskManagerState>,
    mut input: NewTaskRequest,
) -> Result<AsyncTask, AppError> {
    // 注入公共上下文：项目级任务需要 app_data_dir，但前端没有
    if matches!(
        input.task_type.as_str(),
        "sync_embeddings" | "generate_summary"
    ) {
        let app_data_dir = app.path().app_data_dir().map_err(|e| {
            AppError::Internal(format!("Failed to get app data dir: {}", e))
        })?;
        let mut payload = input.payload_json.unwrap_or_else(|| serde_json::json!({}));
        payload["app_data_dir"] = serde_json::Value::String(app_data_dir.to_string_lossy().to_string());
        input.payload_json = Some(payload);
    }

    let manager = task_manager_state.read().await.clone()
        .ok_or_else(|| AppError::Internal("TaskManager not initialized".into()))?;
    
    let task_id = manager.submit(input.clone()).await?;
    
    // 返回提交的任务
    let task = crate::db::repo::async_task_repo::get_by_id(&pool, &task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task '{}' not found after creation", task_id)))?;
    Ok(task)
}

#[tauri::command]
pub async fn list_tasks(
    pool: State<'_, SqlitePool>,
    project_id: String,
    status_filter: Option<String>,
) -> Result<Vec<AsyncTask>, AppError> {
    crate::db::repo::async_task_repo::list_by_project(&pool, &project_id, status_filter.as_deref()).await
}

#[tauri::command]
pub async fn get_task(
    pool: State<'_, SqlitePool>,
    task_id: String,
) -> Result<AsyncTask, AppError> {
    crate::db::repo::async_task_repo::get_by_id(&pool, &task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task '{}' not found", task_id)))
}

#[tauri::command]
pub async fn cancel_task(
    pool: State<'_, SqlitePool>,
    task_id: String,
) -> Result<(), AppError> {
    crate::db::repo::async_task_repo::update_status(
        &pool,
        &task_id,
        crate::models::async_task::TaskStatus::Cancelled,
        None,
        None,
        None,
        Some("Cancelled by user"),
    )
    .await
}
