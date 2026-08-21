use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

use crate::ai::local_provider::LocalEmbedder;
use crate::ai::openai_provider::OpenAiProvider;
use crate::ai::provider::AiProvider;
use crate::error::AppError;
use crate::models::ai_provider::AiProvider as AiProviderModel;

/// 全局缓存的本地嵌入实例（避免重复加载模型）
static LOCAL_EMBEDDER: OnceCell<Box<dyn AiProvider>> = OnceCell::new();

/// 直接获取本地嵌入 provider（不走 DB provider 配置，纯 hypembed）
///
/// 模型目录解析顺序：
/// 1. LINDEN_EMBEDDER_DIR 环境变量
/// 2. app_data_dir/embedder_model
///
/// 首次调用加载模型并缓存，后续直接返回引用。
pub fn get_local_embedder(
    app_data_dir: &Path,
) -> Result<&'static Box<dyn AiProvider>, AppError> {
    LOCAL_EMBEDDER.get_or_try_init(|| {
        let model_dir = std::env::var("LINDEN_EMBEDDER_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| app_data_dir.join("embedder_model"));
        
        // 检查必需文件是否就绪
        let required = ["config.json", "vocab.txt", "model.safetensors"];
        let missing: Vec<&str> = required
            .iter()
            .filter(|f| !model_dir.join(f).exists())
            .copied()
            .collect();

        if !missing.is_empty() {
            return Err(AppError::Validation(format!(
                "嵌入模型文件缺失: {:?}\n模型可能正在后台下载中，请稍后重试；或手动设置 LINDEN_EMBEDDER_DIR 环境变量指定模型目录",
                missing
            )));
        }

        let embedder = LocalEmbedder::new(&model_dir, "bge-small-zh-v1.5".to_string())?;
        Ok(Box::new(embedder) as Box<dyn AiProvider>)
    })
}

/// 根据数据库配置创建 AI Provider 实例
///
/// provider_type 映射：
/// - `openai` / `deepseek`：OpenAI 兼容 HTTP API
/// - `local_embedder`：hypembed 纯 Rust 本地嵌入（base_url 字段存模型目录路径）
pub fn create_provider(
    config: &AiProviderModel,
    api_key: &str,
) -> Result<Box<dyn AiProvider>, AppError> {
    let models = config.models_json.clone();

    match config.provider_type.as_str() {
        "openai" => {
            let provider = OpenAiProvider::new(
                config.name.clone(),
                config.base_url.clone(),
                api_key.to_string(),
                None,
            )?;
            Ok(Box::new(provider))
        }
        "deepseek" => {
            let provider = OpenAiProvider::new(
                config.name.clone(),
                config.base_url.clone(),
                api_key.to_string(),
                Some("text-embedding-v3".to_string()),
            )?;
            Ok(Box::new(provider))
        }
        "local_embedder" => {
            // base_url 存储本地模型目录路径（可覆盖 LINDEN_EMBEDDER_DIR 环境变量）
            let model_dir = if !config.base_url.is_empty() {
                std::path::PathBuf::from(&config.base_url)
            } else if let Ok(env_path) = std::env::var("LINDEN_EMBEDDER_DIR") {
                std::path::PathBuf::from(env_path)
            } else {
                return Err(AppError::Validation(
                    "Local embedder requires model dir path in base_url or LINDEN_EMBEDDER_DIR env".into(),
                ));
            };

            // 嵌入模型名：优先从 models_json 读取，否则给默认名
            let model_name = if !models.is_empty() && models != "[]" {
                models.trim_matches(&['[', ']', '"'][..]).to_string()
            } else {
                "local-minilm".to_string()
            };

            let embedder = LocalEmbedder::new(&model_dir, model_name)?;
            Ok(Box::new(embedder))
        }
        other => Err(AppError::Validation(format!(
            "Unsupported provider type: {}",
            other
        ))),
    }
}
