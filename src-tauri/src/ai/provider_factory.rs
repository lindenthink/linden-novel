use crate::ai::provider::AiProvider;
use crate::ai::openai_provider::OpenAiProvider;
use crate::error::AppError;
use crate::models::ai_provider::AiProvider as AiProviderModel;

/// 根据数据库配置创建 AI Provider 实例
pub fn create_provider(
    config: &AiProviderModel,
    api_key: &str,
) -> Result<Box<dyn AiProvider>, AppError> {
    let models: Vec<String> = serde_json::from_str(&config.models_json)
        .map_err(|e| AppError::Validation(format!("Invalid models_json: {}", e)))?;

    match config.provider_type.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                config.name.clone(),
                config.base_url.clone(),
                api_key.to_string(),
                models,
                None, // 使用默认 embedding model (text-embedding-3-small)
            )?;
            Ok(Box::new(provider))
        }
        // 未来可以添加更多 provider 类型
        // "claude" => { ... }
        // "custom" => { ... }
        other => Err(AppError::Validation(format!(
            "Unsupported provider type: {}",
            other
        ))),
    }
}
