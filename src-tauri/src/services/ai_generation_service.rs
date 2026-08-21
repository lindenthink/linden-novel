use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};
use futures::StreamExt;
use crate::error::AppError;
use crate::models::ai_generation::{AiGenerationHistory, CreateAiGeneration};
use crate::db::repo::ai_generation_repo;
use crate::ai::context_collector;
use crate::ai::generation_prompts;
use crate::ai::provider_factory;
use crate::ai::provider::{CompletionRequest, Message};
use crate::services::{ai_provider_service, ai_api_key_service};
use std::path::Path;

/// 流式生成事件 payload（content chunk）
#[derive(serde::Serialize, Clone)]
pub struct GenerationChunkEvent {
    pub content: String,
    pub done: bool,
}

/// 流式生成事件 payload（reasoning chunk，仅 reasoning 模型）
#[derive(serde::Serialize, Clone)]
pub struct ReasoningChunkEvent {
    pub reasoning: String,
    pub done: bool,
}

/// 流式生成事件 payload（流结束，带保存的 history）
#[derive(serde::Serialize, Clone)]
pub struct GenerationDoneEvent {
    pub content: String,
    pub history: AiGenerationHistory,
}

pub async fn generate(
    pool: &SqlitePool,
    app_data_dir: &Path,
    chapter_id: &str,
    mode: &str,
    user_instruction: Option<&str>,
    parameters: Option<crate::models::ai_generation::GenerationParameters>,
) -> Result<(String, AiGenerationHistory), AppError> {
    // 收集上下文（启用 RAG 检索，传入用户指令增强 query）
    let context = context_collector::collect_context_with_rag_and_instruction(
        pool,
        Some(app_data_dir),
        chapter_id,
        user_instruction,
    )
    .await?;

    // 日志：输出上下文信息
    tracing::info!("=== AI Generation Context ===");
    tracing::info!("Chapter ID: {}", chapter_id);
    tracing::info!("Chapter Title: {}", context.chapter_title);
    tracing::info!("Chapter Summary: {:?}", context.chapter_summary);
    tracing::info!("Chapter Content Length: {} chars", context.chapter_content.len());
    tracing::info!("Previous Chapter Summary: {:?}", context.previous_chapter_summary);
    tracing::info!("Next Chapter Summary: {:?}", context.next_chapter_summary);
    tracing::info!("Characters Count: {}", context.characters.len());
    for (i, char) in context.characters.iter().enumerate() {
        tracing::info!("  Character {}: {} - {:?}", i + 1, char.name, char.description);
    }
    tracing::info!("Storylines Count: {}", context.storylines.len());
    for (i, storyline) in context.storylines.iter().enumerate() {
        tracing::info!("  Storyline {}: {} - {:?}", i + 1, storyline.title, storyline.description);
    }
    tracing::info!("Worldviews Count: {}", context.worldviews.len());
    for (i, worldview) in context.worldviews.iter().enumerate() {
        tracing::info!("  Worldview {}: {} - {:?}", i + 1, worldview.name, worldview.description);
    }
    if let Some(rag_ctx) = &context.rag_context {
        tracing::info!("RAG Context Length: {} chars", rag_ctx.len());
        tracing::debug!("RAG Context Content:\n{}", rag_ctx);
    } else {
        tracing::info!("RAG Context: None");
    }

    // 构建提示词
    let params = parameters.unwrap_or(crate::models::ai_generation::GenerationParameters {
        target_words: None,
        temperature: None,
        style: None,
    });
    let prompt = generation_prompts::build_generation_prompt(
        &context,
        mode,
        user_instruction,
        params.target_words,
    );

    // 日志：输出完整提示词
    tracing::info!("=== AI Generation Prompt ===");
    tracing::info!("Mode: {}", mode);
    tracing::info!("User Instruction: {:?}", user_instruction);
    tracing::info!("Prompt Length: {} chars", prompt.len());
    tracing::debug!("Full Prompt:\n{}", prompt);

    // 获取默认 provider
    let provider = ai_provider_service::get_default(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("No default AI provider configured".to_string()))?;

    // 获取 API key
    let default_key = ai_api_key_service::get_default_for_provider(pool, &provider.id)
        .await?
        .ok_or_else(|| AppError::NotFound("No API key configured for this provider".to_string()))?;
    
    let api_key = ai_api_key_service::get_decrypted(pool, app_data_dir, &default_key.id).await?;

    // 创建 provider 实例
    let ai_provider = provider_factory::create_provider(&provider, &api_key)?;

    // 构建请求
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "你是一位专业的小说创作助手。请根据用户的指令和上下文，生成高质量的文本内容。直接输出文本，不要包含任何解释或标记。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: prompt.clone(),
        },
    ];

    let request = CompletionRequest {
        model: provider.models_json.clone(),
        messages,
        temperature: params.temperature,
        // 不设置 max_tokens：推理模型（如 DeepSeek V4）默认开启 thinking，
        // 硬 token 限制会被 thinking 阶段全部消耗，导致 content 为空。
        max_tokens: None,
        stream: false,
    };

    // 调用 AI
    tracing::info!(
        chapter_id = chapter_id,
        mode = mode,
        provider = %ai_provider.name(),
        model = %request.model,
        "AI generation calling provider"
    );

    let response = ai_provider.complete(request).await?;

    tracing::info!(
        chapter_id = chapter_id,
        mode = mode,
        content_len = response.content.len(),
        model = %response.model,
        "AI generation completed"
    );

    // 保存历史记录
    let history = ai_generation_repo::create(
        pool,
        &CreateAiGeneration {
            chapter_id: chapter_id.to_string(),
            mode: mode.to_string(),
            input_context: prompt,
            output_content: response.content.clone(),
            parameters: params,
        },
    ).await?;

    Ok((response.content, history))
}

pub async fn list_history(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<Vec<AiGenerationHistory>, AppError> {
    ai_generation_repo::list_by_chapter(pool, chapter_id).await
}

pub async fn get_history(
    pool: &SqlitePool,
    id: &str,
) -> Result<AiGenerationHistory, AppError> {
    ai_generation_repo::get(pool, id).await
}

pub async fn delete_history(
    pool: &SqlitePool,
    id: &str,
) -> Result<(), AppError> {
    ai_generation_repo::delete(pool, id).await
}

pub async fn delete_history_by_chapter(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<u64, AppError> {
    ai_generation_repo::delete_by_chapter(pool, chapter_id).await
}

/// 流式生成（真流式：首 token 即通过事件推送前端，边收边存）
///
/// 事件协议：
/// - `ai-generation-reasoning`：`ReasoningChunkEvent { reasoning, done }`，推理过程增量（仅 reasoning 模型）
/// - `ai-generation-chunk`：`GenerationChunkEvent { content, done }`，正文内容增量
/// - `ai-generation-error`：`String`，错误信息
/// - `ai-generation-done`：`GenerationDoneEvent { content, history }`，流结束并返回完整 history
pub async fn generate_stream(
    pool: &SqlitePool,
    app_handle: &AppHandle,
    app_data_dir: &Path,
    chapter_id: &str,
    mode: &str,
    user_instruction: Option<&str>,
    parameters: Option<crate::models::ai_generation::GenerationParameters>,
) -> Result<(), AppError> {
    // 复用 generate 的上下文收集 + prompt 构建
    let context = context_collector::collect_context_with_rag_and_instruction(
        pool,
        Some(app_data_dir),
        chapter_id,
        user_instruction,
    )
    .await?;

    tracing::info!("=== AI Generation (stream) Context ===");
    tracing::info!("Chapter ID: {}", chapter_id);
    tracing::info!("Chapter Title: {}", context.chapter_title);
    tracing::info!("Chapter Summary: {:?}", context.chapter_summary);
    tracing::info!("Chapter Content Length: {} chars", context.chapter_content.len());
    tracing::info!("Previous Chapter Summary: {:?}", context.previous_chapter_summary);
    tracing::info!("Next Chapter Summary: {:?}", context.next_chapter_summary);
    tracing::info!("Characters Count: {}", context.characters.len());
    for (i, char) in context.characters.iter().enumerate() {
        tracing::info!("  Character {}: {} - {:?}", i + 1, char.name, char.description);
    }
    tracing::info!("Storylines Count: {}", context.storylines.len());
    for (i, storyline) in context.storylines.iter().enumerate() {
        tracing::info!("  Storyline {}: {} - {:?}", i + 1, storyline.title, storyline.description);
    }
    tracing::info!("Worldviews Count: {}", context.worldviews.len());
    for (i, worldview) in context.worldviews.iter().enumerate() {
        tracing::info!("  Worldview {}: {} - {:?}", i + 1, worldview.name, worldview.description);
    }
    if let Some(rag_ctx) = &context.rag_context {
        tracing::info!("RAG Context Length: {} chars", rag_ctx.len());
        tracing::debug!("RAG Context Content:\n{}", rag_ctx);
    } else {
        tracing::info!("RAG Context: None");
    }

    let params = parameters.unwrap_or(crate::models::ai_generation::GenerationParameters {
        target_words: None,
        temperature: None,
        style: None,
    });
    let prompt = generation_prompts::build_generation_prompt(
        &context,
        mode,
        user_instruction,
        params.target_words,
    );

    tracing::info!("=== AI Generation (stream) Prompt ===");
    tracing::info!("Mode: {}", mode);
    tracing::info!("User Instruction: {:?}", user_instruction);
    tracing::info!("Prompt Length: {} chars", prompt.len());
    tracing::debug!("Full Prompt:\n{}", prompt);

    let provider = ai_provider_service::get_default(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("No default AI provider configured".to_string()))?;

    let default_key = ai_api_key_service::get_default_for_provider(pool, &provider.id)
        .await?
        .ok_or_else(|| AppError::NotFound("No API key configured for this provider".to_string()))?;

    let api_key = ai_api_key_service::get_decrypted(pool, app_data_dir, &default_key.id).await?;

    let ai_provider = provider_factory::create_provider(&provider, &api_key)?;

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "你是一位专业的小说创作助手。请根据用户的指令和上下文，生成高质量的文本内容。直接输出文本，不要包含任何解释或标记。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: prompt.clone(),
        },
    ];

    let request = CompletionRequest {
        model: provider.models_json.clone(),
        messages,
        temperature: params.temperature,
        max_tokens: None,
        stream: true,
    };

    tracing::info!(
        chapter_id = chapter_id,
        mode = mode,
        provider = %ai_provider.name(),
        model = %request.model,
        "AI stream generation calling provider"
    );

    // 调用真流式接口
    let mut stream = ai_provider.complete_stream(request).await?;

    // 边收边发，累积完整 content
    let mut full_content = String::new();
    let mut stream_done = false;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // 推理过程增量（DeepSeek thinking 模式）
                if !chunk.reasoning.is_empty() {
                    app_handle
                        .emit(
                            "ai-generation-reasoning",
                            ReasoningChunkEvent {
                                reasoning: chunk.reasoning,
                                done: false,
                            },
                        )
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to emit reasoning: {}", e))
                        })?;
                }
                // 正文内容增量
                if !chunk.content.is_empty() {
                    full_content.push_str(&chunk.content);
                    app_handle
                        .emit(
                            "ai-generation-chunk",
                            GenerationChunkEvent {
                                content: chunk.content,
                                done: false,
                            },
                        )
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to emit chunk: {}", e))
                        })?;
                }
                if chunk.done {
                    stream_done = true;
                    app_handle
                        .emit(
                            "ai-generation-chunk",
                            GenerationChunkEvent {
                                content: String::new(),
                                done: true,
                            },
                        )
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to emit done chunk: {}", e))
                        })?;
                    break;
                }
            }
            Err(e) => {
                tracing::error!(chapter_id = chapter_id, error = %e, "AI stream generation error");
                app_handle
                    .emit("ai-generation-error", e.to_string())
                    .map_err(|e| AppError::Internal(format!("Failed to emit error: {}", e)))?;
                return Err(e);
            }
        }
    }

    if !stream_done {
        tracing::warn!(chapter_id = chapter_id, "AI stream ended without done signal");
    }

    tracing::info!(
        chapter_id = chapter_id,
        mode = mode,
        content_len = full_content.len(),
        "AI stream generation completed"
    );

    // 保存历史记录
    let history = ai_generation_repo::create(
        pool,
        &CreateAiGeneration {
            chapter_id: chapter_id.to_string(),
            mode: mode.to_string(),
            input_context: prompt,
            output_content: full_content.clone(),
            parameters: params,
        },
    )
    .await?;

    // 发送流结束事件（携带完整 history，前端可设置 currentGeneration）
    app_handle
        .emit(
            "ai-generation-done",
            GenerationDoneEvent {
                content: full_content,
                history,
            },
        )
        .map_err(|e| AppError::Internal(format!("Failed to emit done: {}", e)))?;

    Ok(())
}
