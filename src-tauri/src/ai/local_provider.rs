use async_trait::async_trait;
use hypembed::{Embedder, EmbeddingOptions, PoolingStrategy};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ai::provider::{
    AiProvider, BatchEmbeddingRequest, BatchEmbeddingResponse, CompletionRequest,
    CompletionResponse, EmbeddingRequest, EmbeddingResponse, StreamChunkStream,
};
use crate::error::AppError;

/// 本地嵌入 provider（基于 hypembed，纯 Rust 推理）
///
/// - 仅支持 embed / embed_batch；complete 走 NotSupported
/// - 推理为 CPU 同步阻塞，所有嵌入调用都通过 spawn_blocking 异步化
/// - 模型目录需包含 config.json / vocab.txt / model.safetensors
pub struct LocalEmbedder {
    embedder: Arc<Mutex<Embedder>>,
    model_name: String,
    dim: usize,
}

impl LocalEmbedder {
    pub fn new(model_dir: &Path, model_name: String) -> Result<Self, AppError> {
        let embedder = Embedder::load(model_dir).map_err(|e| {
            AppError::Internal(format!(
                "Load local embedder from {:?} failed: {}",
                model_dir, e
            ))
        })?;

        // 用空向量做一次探测，拿到实际维度
        let options = EmbeddingOptions::default()
            .with_pooling(PoolingStrategy::Mean)
            .with_normalize(true);
        let probe = embedder
            .embed(&["probe embedding dimension"], &options)
            .map_err(|e| AppError::Internal(format!("Embedder probe failed: {}", e)))?;
        let dim = probe[0].len();

        tracing::info!(
            "LocalEmbedder loaded: model={}, dir={:?}, dim={}",
            model_name,
            model_dir,
            dim
        );

        Ok(Self {
            embedder: Arc::new(Mutex::new(embedder)),
            model_name,
            dim,
        })
    }

    fn options() -> EmbeddingOptions {
        EmbeddingOptions::default()
            .with_pooling(PoolingStrategy::Mean)
            .with_normalize(true)
    }
}

#[async_trait]
impl AiProvider for LocalEmbedder {
    async fn complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionResponse, AppError> {
        Err(AppError::Validation(
            "Local embedder does not support chat completion".into(),
        ))
    }

    async fn complete_stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<StreamChunkStream, AppError> {
        Err(AppError::Validation(
            "Local embedder does not support streaming".into(),
        ))
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, AppError> {
        let embedder = self.embedder.clone();
        let options = Self::options();
        let model_name = self.model_name.clone();
        let dim = self.dim;

        let vectors = tokio::task::spawn_blocking(move || {
            let guard = embedder.blocking_lock();
            let texts: Vec<&str> = vec![request.input.as_str()];
            guard.embed(&texts, &options)
        })
        .await
        .map_err(|e| AppError::Internal(format!("spawn_blocking join error: {}", e)))?
        .map_err(|e| AppError::Internal(format!("Local embed failed: {}", e)))?;

        let vector = vectors
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Local embed returned empty".into()))?;

        Ok(EmbeddingResponse {
            vector,
            model: model_name,
            dim,
        })
    }

    async fn embed_batch(
        &self,
        request: BatchEmbeddingRequest,
    ) -> Result<BatchEmbeddingResponse, AppError> {
        let embedder = self.embedder.clone();
        let options = Self::options();
        let model_name = self.model_name.clone();
        let dim = self.dim;
        let inputs = request.inputs;

        let vectors = tokio::task::spawn_blocking(move || {
            let guard = embedder.blocking_lock();
            let slices: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
            // hypembed 内部已用 rayon 并行
            guard.embed(&slices, &options)
        })
        .await
        .map_err(|e| AppError::Internal(format!("spawn_blocking join error: {}", e)))?
        .map_err(|e| AppError::Internal(format!("Local batch embed failed: {}", e)))?;

        Ok(BatchEmbeddingResponse {
            vectors,
            model: model_name,
            dim,
        })
    }

    fn name(&self) -> &str {
        "local_embedder"
    }

    fn models(&self) -> Vec<String> {
        vec![self.model_name.clone()]
    }
}
