use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::ai::provider::{CompletionRequest, Message};
use crate::ai::provider_factory;
use crate::error::AppError;
use crate::services::{ai_api_key_service, ai_provider_service};

#[derive(Deserialize)]
pub struct CompleteRequest {
    pub provider_id: Option<String>,
    pub api_key_id: Option<String>,
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub stream: Option<bool>,
}

#[derive(Serialize)]
pub struct CompleteResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<UsageInfo>,
}

#[derive(Serialize)]
pub struct UsageInfo {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Serialize, Clone)]
pub struct StreamChunkEvent {
    pub content: String,
    pub done: bool,
}

#[tauri::command]
pub async fn ai_complete(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: CompleteRequest,
) -> Result<CompleteResponse, AppError> {
    tracing::info!(
        provider_id = ?request.provider_id,
        model = %request.model,
        msg_count = request.messages.len(),
        "ai_complete command invoked"
    );

    // 获取 provider
    let provider = if let Some(provider_id) = &request.provider_id {
        ai_provider_service::get(pool.inner(), provider_id).await?
    } else {
        ai_provider_service::get_default(pool.inner())
            .await?
            .ok_or_else(|| AppError::NotFound("No default AI provider configured".to_string()))?
    };

    // 获取 API key
    let api_key = if let Some(api_key_id) = &request.api_key_id {
        let app_data_dir = app.path().app_data_dir().map_err(|e| {
            AppError::Internal(format!("Failed to get app data dir: {}", e))
        })?;
        ai_api_key_service::get_decrypted(pool.inner(), &app_data_dir, api_key_id).await?
    } else {
        // 使用默认 key
        let default_key = ai_api_key_service::get_default_for_provider(pool.inner(), &provider.id)
            .await?
            .ok_or_else(|| AppError::NotFound("No API key configured for this provider".to_string()))?;
        
        let app_data_dir = app.path().app_data_dir().map_err(|e| {
            AppError::Internal(format!("Failed to get app data dir: {}", e))
        })?;
        ai_api_key_service::get_decrypted(pool.inner(), &app_data_dir, &default_key.id).await?
    };

    // 创建 provider 实例
    let ai_provider = provider_factory::create_provider(&provider, &api_key)?;

    // 构建请求
    let completion_request = CompletionRequest {
        model: request.model.clone(),
        messages: request.messages,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        stream: false,
    };

    // 调用 AI
    let response = ai_provider.complete(completion_request).await?;

    tracing::info!(
        model = %response.model,
        content_len = response.content.len(),
        "ai_complete command completed"
    );

    Ok(CompleteResponse {
        content: response.content,
        model: response.model,
        usage: response.usage.map(|u| UsageInfo {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
    })
}

#[tauri::command]
pub async fn ai_complete_stream(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: CompleteRequest,
) -> Result<(), AppError> {
    tracing::info!(
        provider_id = ?request.provider_id,
        model = %request.model,
        msg_count = request.messages.len(),
        "ai_complete_stream command invoked"
    );

    // 获取 provider
    let provider = if let Some(provider_id) = &request.provider_id {
        ai_provider_service::get(pool.inner(), provider_id).await?
    } else {
        ai_provider_service::get_default(pool.inner())
            .await?
            .ok_or_else(|| AppError::NotFound("No default AI provider configured".to_string()))?
    };

    // 获取 API key
    let api_key = if let Some(api_key_id) = &request.api_key_id {
        let app_data_dir = app.path().app_data_dir().map_err(|e| {
            AppError::Internal(format!("Failed to get app data dir: {}", e))
        })?;
        ai_api_key_service::get_decrypted(pool.inner(), &app_data_dir, api_key_id).await?
    } else {
        let default_key = ai_api_key_service::get_default_for_provider(pool.inner(), &provider.id)
            .await?
            .ok_or_else(|| AppError::NotFound("No API key configured for this provider".to_string()))?;
        
        let app_data_dir = app.path().app_data_dir().map_err(|e| {
            AppError::Internal(format!("Failed to get app data dir: {}", e))
        })?;
        ai_api_key_service::get_decrypted(pool.inner(), &app_data_dir, &default_key.id).await?
    };

    // 创建 provider 实例
    let ai_provider = provider_factory::create_provider(&provider, &api_key)?;

    // 构建请求
    let completion_request = CompletionRequest {
        model: request.model.clone(),
        messages: request.messages,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        stream: true,
    };

    // 调用 AI 流式接口
    let stream = ai_provider.complete_stream(completion_request).await?;

    // 发送流式事件
    for chunk_result in stream {
        match chunk_result {
            Ok(chunk) => {
                let event = StreamChunkEvent {
                    content: chunk.content,
                    done: chunk.done,
                };
                app.emit("ai-stream-chunk", event).map_err(|e| {
                    AppError::Internal(format!("Failed to emit stream chunk: {}", e))
                })?;

                if chunk.done {
                    break;
                }
            }
            Err(e) => {
                app.emit("ai-stream-error", e.to_string()).map_err(|e| {
                    AppError::Internal(format!("Failed to emit stream error: {}", e))
                })?;
                break;
            }
        }
    }

    app.emit("ai-stream-done", ()).map_err(|e| {
        AppError::Internal(format!("Failed to emit stream done: {}", e))
    })?;

    tracing::info!(model = %request.model, "ai_complete_stream command completed");

    Ok(())
}
