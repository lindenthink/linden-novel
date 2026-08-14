use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::async_task::NewTaskRequest;
use crate::models::storyline::{Storyline, CreateStoryline, UpdateStoryline};
use crate::services::storyline_service;
use crate::services::task_manager::TaskManagerState;

#[tauri::command]
pub async fn list_storylines(
    pool: State<'_, SqlitePool>,
    project_id: String,
) -> Result<Vec<Storyline>, AppError> {
    storyline_service::list(&pool, &project_id).await
}

#[tauri::command]
pub async fn get_storyline(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<Storyline, AppError> {
    storyline_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_storyline(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    task_manager_state: State<'_, TaskManagerState>,
    input: CreateStoryline,
) -> Result<Storyline, AppError> {
    let storyline = storyline_service::create(&pool, &input).await?;
    
    // 成功创建后，提交嵌入任务
    if let Some(desc) = &storyline.description {
        if !desc.trim().is_empty() {
            submit_embed_task(&app, &task_manager_state, &storyline, desc).await?;
        }
    }
    
    Ok(storyline)
}

#[tauri::command]
pub async fn update_storyline(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    task_manager_state: State<'_, TaskManagerState>,
    id: String,
    input: UpdateStoryline,
) -> Result<Storyline, AppError> {
    let updated = storyline_service::update(&pool, &id, &input).await?;
    
    // 成功更新后，提交嵌入任务
    if let Some(desc) = &updated.description {
        if !desc.trim().is_empty() {
            submit_embed_task(&app, &task_manager_state, &updated, desc).await?;
        }
    }
    
    Ok(updated)
}

#[tauri::command]
pub async fn delete_storyline(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    storyline_service::delete(&pool, &id).await
}

/// 提交元素嵌入任务到 TaskManager
async fn submit_embed_task(
    app: &AppHandle,
    task_manager_state: &State<'_, TaskManagerState>,
    storyline: &Storyline,
    text: &str,
) -> Result<(), AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    let app_data_dir_str = app_data_dir.to_string_lossy().to_string();
    
    // 计算内容的 hash
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());
    
    let payload = serde_json::json!({
        "text": text,
        "app_data_dir": app_data_dir_str
    });
    
    let task_manager = task_manager_state.read().await.clone()
        .ok_or_else(|| AppError::Internal("TaskManager not initialized".into()))?;
    
    task_manager.submit(NewTaskRequest {
        task_type: "embed_element".to_string(),
        project_id: storyline.project_id.clone(),
        target_type: Some("storyline".to_string()),
        target_id: Some(storyline.id.clone()),
        content_hash: Some(content_hash),
        payload_json: Some(payload),
    }).await?;
    
    Ok(())
}
