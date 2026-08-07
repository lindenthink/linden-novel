use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::stream::StreamExt;

use crate::ai::provider::{
    AiProvider, CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse,
    StreamChunk, Usage,
};
use crate::error::AppError;

#[derive(Debug)]
pub struct OpenAiProvider {
    name: String,
    base_url: String,
    api_key: String,
    models: Vec<String>,
    embedding_model: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(
        name: String,
        base_url: String,
        api_key: String,
        models: Vec<String>,
        embedding_model: Option<String>,
    ) -> Result<Self, AppError> {
        let client = Client::builder()
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            name,
            base_url,
            api_key,
            models,
            embedding_model: embedding_model
                .unwrap_or_else(|| "text-embedding-3-small".to_string()),
            client,
        })
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}/v1{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

// --- Embeddings API structs ---

#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
    model: String,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
    #[allow(dead_code)]
    index: i32,
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AppError> {
        let url = self.build_url("/chat/completions");
        
        let openai_req = OpenAiRequest {
            model: request.model,
            messages: request.messages.into_iter().map(|m| OpenAiMessage {
                role: m.role,
                content: m.content,
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("API error {}: {}", status, body)));
        }

        let openai_resp: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse response: {}", e)))?;

        let content = openai_resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .map(|m| m.content.clone())
            .unwrap_or_default();

        Ok(CompletionResponse {
            id: openai_resp.id,
            content,
            model: openai_resp.model,
            usage: openai_resp.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn Iterator<Item = Result<StreamChunk, AppError>> + Send>, AppError> {
        let url = self.build_url("/chat/completions");
        
        let openai_req = OpenAiRequest {
            model: request.model,
            messages: request.messages.into_iter().map(|m| OpenAiMessage {
                role: m.role,
                content: m.content,
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("API error {}: {}", status, body)));
        }

        let stream = response.bytes_stream();
        let buffer = Arc::new(Mutex::new(String::new()));
        let stream_clone = stream;

        // 将字节流转换为 SSE 解析器
        let chunk_stream = stream_clone
            .map(move |chunk_result| {
                let buffer = buffer.clone();
                async move {
                    match chunk_result {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes).to_string();
                            let mut buf = buffer.lock().await;
                            buf.push_str(&text);
                            
                            // 解析 SSE 数据
                            let mut chunks = Vec::new();
                            while let Some(pos) = buf.find("\n\n") {
                                let line = buf[..pos].to_string();
                                *buf = buf[pos + 2..].to_string();
                                
                                for line in line.lines() {
                                    if line.starts_with("data: ") {
                                        let data = &line[6..];
                                        if data == "[DONE]" {
                                            chunks.push(Ok(StreamChunk {
                                                content: String::new(),
                                                done: true,
                                            }));
                                        } else if let Ok(chunk) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                                            if let Some(choice) = chunk.choices.first() {
                                                if let Some(content) = &choice.delta.content {
                                                    chunks.push(Ok(StreamChunk {
                                                        content: content.clone(),
                                                        done: false,
                                                    }));
                                                }
                                                if choice.finish_reason.is_some() {
                                                    chunks.push(Ok(StreamChunk {
                                                        content: String::new(),
                                                        done: true,
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            chunks
                        }
                        Err(e) => vec![Err(AppError::Internal(format!("Stream error: {}", e)))],
                    }
                }
            })
            .buffer_unordered(1)
            .flat_map(futures::stream::iter)
            .collect::<Vec<_>>()
            .await;

        Ok(Box::new(chunk_stream.into_iter()))
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, AppError> {
        let url = self.build_url("/embeddings");

        let model = if request.model.is_empty() {
            self.embedding_model.clone()
        } else {
            request.model
        };

        let openai_req = OpenAiEmbeddingRequest {
            model,
            input: request.input,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Embedding HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Embedding API error {}: {}",
                status, body
            )));
        }

        let openai_resp: OpenAiEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse embedding response: {}", e)))?;

        let data = openai_resp
            .data
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Empty embedding response".into()))?;

        let dim = data.embedding.len();
        Ok(EmbeddingResponse {
            vector: data.embedding,
            model: openai_resp.model,
            dim,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn models(&self) -> Vec<String> {
        self.models.clone()
    }
}
