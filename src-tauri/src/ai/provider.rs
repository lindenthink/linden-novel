use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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
    pub content: String,
    pub done: bool,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    /// 非流式完成请求
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AppError>;
    
    /// 流式完成请求，返回 chunk 迭代器
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn Iterator<Item = Result<StreamChunk, AppError>> + Send>, AppError>;
    
    /// 获取 provider 名称
    fn name(&self) -> &str;
    
    /// 获取支持的模型列表
    fn models(&self) -> Vec<String>;
}
