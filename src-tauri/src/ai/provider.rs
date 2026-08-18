use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// 正文内容增量
    pub content: String,
    pub done: bool,
    /// 推理过程增量（仅 reasoning 模型，如 DeepSeek V3/V4 thinking 模式）
    #[serde(default)]
    pub reasoning: String,
}

/// 单条嵌入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub vector: Vec<f32>,
    pub model: String,
    pub dim: usize,
}

/// 批量嵌入请求（一次 API 调用生成 N 条向量）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEmbeddingRequest {
    pub model: String,
    pub inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEmbeddingResponse {
    pub vectors: Vec<Vec<f32>>,
    pub model: String,
    pub dim: usize,
}

/// 真流式 Stream 类型别名
///
/// `Pin<Box<dyn Stream + Send>>` 既是 `Stream` 又是 `Unpin`，
/// 可直接用 `StreamExt::next()` 消费
pub type StreamChunkStream = Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>>;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AppError>;

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<StreamChunkStream, AppError>;

    /// 单条嵌入
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, AppError>;

    /// 批量嵌入（默认实现：逐条调用 embed）
    ///
    /// 子类可覆盖为 HTTP 原生批量或 rayon 并行加速
    async fn embed_batch(
        &self,
        request: BatchEmbeddingRequest,
    ) -> Result<BatchEmbeddingResponse, AppError> {
        let mut vectors = Vec::with_capacity(request.inputs.len());
        let mut dim = 0usize;
        let mut model = String::new();
        for input in request.inputs {
            let resp = self
                .embed(EmbeddingRequest {
                    model: request.model.clone(),
                    input,
                })
                .await?;
            dim = resp.dim;
            model = resp.model;
            vectors.push(resp.vector);
        }
        Ok(BatchEmbeddingResponse { vectors, model, dim })
    }

    fn name(&self) -> &str;

    fn models(&self) -> Vec<String>;
}
