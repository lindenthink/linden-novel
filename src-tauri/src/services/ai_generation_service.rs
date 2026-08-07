use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::ai_generation::{AiGenerationHistory, CreateAiGeneration, GenerationContext};
use crate::db::repo::ai_generation_repo;
use crate::ai::context_collector;
use crate::ai::generation_prompts;
use crate::ai::provider_factory;
use crate::ai::provider::{AiProvider, CompletionRequest, Message};
use crate::services::{ai_provider_service, ai_api_key_service};
use std::path::Path;

pub async fn generate(
    pool: &SqlitePool,
    app_data_dir: &Path,
    chapter_id: &str,
    mode: &str,
    user_instruction: Option<&str>,
    parameters: Option<crate::models::ai_generation::GenerationParameters>,
) -> Result<(String, AiGenerationHistory), AppError> {
    // 收集上下文（启用 RAG 检索）
    let context = context_collector::collect_context_with_rag(pool, Some(app_data_dir), chapter_id).await?;

    // 构建提示词
    let prompt = generation_prompts::build_generation_prompt(&context, mode, user_instruction);

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

    let params = parameters.unwrap_or(crate::models::ai_generation::GenerationParameters {
        max_tokens: None,
        temperature: None,
        style: None,
    });

    let request = CompletionRequest {
        model: serde_json::from_str::<Vec<String>>(&provider.models_json)
            .map_err(|e| AppError::Internal(format!("Failed to parse models_json: {}", e)))?
            .first()
            .cloned()
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
        messages,
        temperature: params.temperature,
        max_tokens: params.max_tokens,
        stream: false,
    };

    // 调用 AI
    let response = ai_provider.complete(request).await?;

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
