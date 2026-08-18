use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::ai_generation::{AiGenerationHistory, GenerationParameters};
use crate::services::ai_generation_service;

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub chapter_id: String,
    pub mode: String,
    pub user_instruction: Option<String>,
    pub parameters: Option<GenerationParameters>,
}

#[derive(Serialize)]
pub struct GenerateResponse {
    pub content: String,
    pub history: AiGenerationHistory,
}

#[tauri::command]
pub async fn ai_generate(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: GenerateRequest,
) -> Result<GenerateResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let (content, history) = ai_generation_service::generate(
        pool.inner(),
        &app_data_dir,
        &request.chapter_id,
        &request.mode,
        request.user_instruction.as_deref(),
        request.parameters,
    ).await?;

    Ok(GenerateResponse { content, history })
}

/// 流式 AI 生成命令
///
/// 不返回生成内容（通过事件 `ai-generation-chunk` / `ai-generation-done` 推送），
/// 仅返回调用是否成功启动。生成内容、历史记录均通过事件传递。
#[tauri::command]
pub async fn ai_generate_stream(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: GenerateRequest,
) -> Result<(), AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    ai_generation_service::generate_stream(
        pool.inner(),
        &app,
        &app_data_dir,
        &request.chapter_id,
        &request.mode,
        request.user_instruction.as_deref(),
        request.parameters,
    )
    .await
}

#[tauri::command]
pub async fn list_ai_generation_history(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
) -> Result<Vec<AiGenerationHistory>, AppError> {
    ai_generation_service::list_history(pool.inner(), &chapter_id).await
}

#[tauri::command]
pub async fn get_ai_generation_history(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<AiGenerationHistory, AppError> {
    ai_generation_service::get_history(pool.inner(), &id).await
}

#[tauri::command]
pub async fn delete_ai_generation_history(
    pool: State<'_, SqlitePool>,
    id: String,
) -> Result<(), AppError> {
    ai_generation_service::delete_history(pool.inner(), &id).await
}

#[tauri::command]
pub async fn delete_ai_generation_history_by_chapter(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
) -> Result<u64, AppError> {
    ai_generation_service::delete_history_by_chapter(pool.inner(), &chapter_id).await
}
