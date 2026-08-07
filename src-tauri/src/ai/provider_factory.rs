use crate::ai::provider::AiProvider;
use crate::ai::openai_provider::OpenAiProvider;
use crate::error::AppError;
use crate::models::ai_provider::AiProvider as AiProviderModel;

/// 根据数据库配置创建 AI Provider 实例
pub fn create_provider(
    config: &AiProviderModel,
    api_key: &str,
) -> Result<Box<dyn AiProvider>, AppError> {
    let model = config.models_json.clone();

    match config.provider_type.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                config.name.clone(),
                config.base_url.clone(),
                api_key.to_string(),
                vec![model],
                None, // 使用默认 embedding model (text-embedding-3-small)
            )?;
            Ok(Box::new(provider))
        }
        "deepseek" => {
            // DeepSeek 兼容 OpenAI API
            let provider = OpenAiProvider::new(
                config.name.clone(),
                config.base_url.clone(),
                api_key.to_string(),
                vec![model],
                Some("text-embedding-v3".to_string()),
            )?;
            Ok(Box::new(provider))
        }
        other => Err(AppError::Validation(format!(
            "Unsupported provider type: {}",
            other
        ))),
    }
}
