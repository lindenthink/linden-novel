use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::stream::StreamExt;

use crate::ai::provider::{
    AiProvider, BatchEmbeddingRequest, BatchEmbeddingResponse, CompletionRequest,
    CompletionResponse, EmbeddingRequest, EmbeddingResponse, StreamChunk, StreamChunkStream, Usage,
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
    /// DeepSeek reasoning 模型的推理过程（V3+ thinking 模式输出）
    #[serde(default)]
    reasoning_content: Option<String>,
}

// --- Embeddings API structs ---

#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Serialize)]
struct OpenAiBatchEmbeddingRequest {
    model: String,
    input: Vec<String>,
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

        tracing::info!(
            provider = %self.name,
            model = %request.model,
            msg_count = request.messages.len(),
            stream = false,
            "AI complete request"
        );

        let openai_req = OpenAiRequest {
            model: request.model.clone(),
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
            .map_err(|e| {
                tracing::error!(provider = %self.name, model = %request.model, error = %e, "AI complete HTTP request failed");
                AppError::Internal(format!("HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(provider = %self.name, model = %request.model, status = %status, body = %body, "AI complete API error");
            return Err(AppError::Internal(format!("API error {}: {}", status, body)));
        }

        let openai_resp: OpenAiResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!(provider = %self.name, model = %request.model, error = %e, "Failed to parse AI complete response");
                AppError::Internal(format!("Failed to parse response: {}", e))
            })?;

        let content = openai_resp.choices
            .first()
            .and_then(|c| c.message.as_ref())
            .map(|m| m.content.clone())
            .unwrap_or_default();

        tracing::info!(
            provider = %self.name,
            model = %openai_resp.model,
            content_len = content.len(),
            prompt_tokens = openai_resp.usage.as_ref().map(|u| u.prompt_tokens),
            completion_tokens = openai_resp.usage.as_ref().map(|u| u.completion_tokens),
            total_tokens = openai_resp.usage.as_ref().map(|u| u.total_tokens),
            "AI complete response"
        );

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
    ) -> Result<StreamChunkStream, AppError> {
        let url = self.build_url("/chat/completions");

        tracing::info!(
            provider = %self.name,
            model = %request.model,
            msg_count = request.messages.len(),
            stream = true,
            "AI complete stream request"
        );

        let openai_req = OpenAiRequest {
            model: request.model.clone(),
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
            .map_err(|e| {
                tracing::error!(provider = %self.name, model = %request.model, error = %e, "AI stream HTTP request failed");
                AppError::Internal(format!("HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(provider = %self.name, model = %request.model, status = %status, body = %body, "AI stream API error");
            return Err(AppError::Internal(format!("API error {}: {}", status, body)));
        }

        tracing::info!(provider = %self.name, model = %request.model, "AI stream response started");

        // 真流式：用 then + flatten 把字节流逐步解析为 StreamChunk 流
        // 不再 collect 成 Vec，首 token 到达即向前端推送
        //
        // 字节缓冲：累积原始字节，按 UTF-8 字符边界安全切分，
        // 避免多字节字符（中文 3 字节）被 chunk 边界切断后变乱码
        let byte_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
        let sse_buffer = Arc::new(Mutex::new(String::new()));
        let chunk_stream = response
            .bytes_stream()
            .then(move |chunk_result| {
                let byte_buffer = byte_buffer.clone();
                let sse_buffer = sse_buffer.clone();
                async move {
                    match chunk_result {
                        Ok(bytes) => {
                            // 累积原始字节
                            let mut byte_buf = byte_buffer.lock().await;
                            byte_buf.extend_from_slice(&bytes);

                            // 按 UTF-8 字符边界安全切分：只解码完整字符，残缺字节保留
                            let safe_end = match std::str::from_utf8(&byte_buf) {
                                Ok(_) => byte_buf.len(),
                                Err(e) => e.valid_up_to(),
                            };
                            if safe_end == 0 {
                                // 当前缓冲全是残缺字节（罕见），等下一 chunk
                                drop(byte_buf);
                                return futures::stream::iter(
                                    Vec::<Result<StreamChunk, AppError>>::new(),
                                );
                            }
                            let text = String::from_utf8_lossy(&byte_buf[..safe_end]).to_string();
                            // 保留不完整字节给下一 chunk
                            let remaining = byte_buf[safe_end..].to_vec();
                            *byte_buf = remaining;
                            drop(byte_buf);

                            // SSE 块解析（跨 chunk 的 \n\n 边界由 sse_buffer 保留）
                            let mut buf = sse_buffer.lock().await;
                            buf.push_str(&text);

                            let mut chunks: Vec<Result<StreamChunk, AppError>> = Vec::new();
                            while let Some(pos) = buf.find("\n\n") {
                                let block = buf[..pos].to_string();
                                *buf = buf[pos + 2..].to_string();

                                for line in block.lines() {
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if data == "[DONE]" {
                                            chunks.push(Ok(StreamChunk {
                                                content: String::new(),
                                                done: true,
                                                reasoning: String::new(),
                                            }));
                                        } else if let Ok(chunk) =
                                            serde_json::from_str::<OpenAiStreamChunk>(data)
                                        {
                                            if let Some(choice) = chunk.choices.first() {
                                                // 推理过程增量（DeepSeek thinking 模式）
                                                if let Some(reasoning) =
                                                    &choice.delta.reasoning_content
                                                {
                                                    if !reasoning.is_empty() {
                                                        chunks.push(Ok(StreamChunk {
                                                            content: String::new(),
                                                            done: false,
                                                            reasoning: reasoning.clone(),
                                                        }));
                                                    }
                                                }
                                                // 正文内容增量
                                                if let Some(content) = &choice.delta.content {
                                                    if !content.is_empty() {
                                                        chunks.push(Ok(StreamChunk {
                                                            content: content.clone(),
                                                            done: false,
                                                            reasoning: String::new(),
                                                        }));
                                                    }
                                                }
                                                if choice.finish_reason.is_some() {
                                                    chunks.push(Ok(StreamChunk {
                                                        content: String::new(),
                                                        done: true,
                                                        reasoning: String::new(),
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            drop(buf);
                            futures::stream::iter(chunks)
                        }
                        Err(e) => {
                            futures::stream::iter(vec![Err(AppError::Internal(format!(
                                "Stream error: {}",
                                e
                            )))])
                        }
                    }
                }
            })
            .flatten();

        Ok(Box::pin(chunk_stream))
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, AppError> {
        let url = self.build_url("/embeddings");

        let model = if request.model.is_empty() {
            self.embedding_model.clone()
        } else {
            request.model
        };

        tracing::info!(
            provider = %self.name,
            model = %model,
            input_len = request.input.len(),
            "AI embedding request"
        );

        let openai_req = OpenAiEmbeddingRequest {
            model: model.clone(),
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
            .map_err(|e| {
                tracing::error!(provider = %self.name, model = %model, error = %e, "AI embedding HTTP request failed");
                AppError::Internal(format!("Embedding HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(provider = %self.name, model = %model, status = %status, body = %body, "AI embedding API error");
            return Err(AppError::Internal(format!(
                "Embedding API error {}: {}",
                status, body
            )));
        }

        let openai_resp: OpenAiEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!(provider = %self.name, model = %model, error = %e, "Failed to parse embedding response");
                AppError::Internal(format!("Failed to parse embedding response: {}", e))
            })?;

        let data = openai_resp
            .data
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Empty embedding response".into()))?;

        let dim = data.embedding.len();
        tracing::info!(
            provider = %self.name,
            model = %openai_resp.model,
            dim = dim,
            "AI embedding response"
        );
        Ok(EmbeddingResponse {
            vector: data.embedding,
            model: openai_resp.model,
            dim,
        })
    }

    async fn embed_batch(
        &self,
        request: BatchEmbeddingRequest,
    ) -> Result<BatchEmbeddingResponse, AppError> {
        let url = self.build_url("/embeddings");
        let model = if request.model.is_empty() {
            self.embedding_model.clone()
        } else {
            request.model
        };

        let batch_size = 64usize;
        let mut all_vectors = Vec::with_capacity(request.inputs.len());
        let mut dim = 0usize;
        let mut resp_model = String::new();

        for chunk in request.inputs.chunks(batch_size) {
            tracing::info!(
                provider = %self.name,
                model = %model,
                batch = chunk.len(),
                "AI batch embedding request"
            );

            let openai_req = OpenAiBatchEmbeddingRequest {
                model: model.clone(),
                input: chunk.to_vec(),
            };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&openai_req)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Batch embed HTTP failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!("Batch embed API {}: {}", status, body)));
            }

            let openai_resp: OpenAiEmbeddingResponse = response.json().await
                .map_err(|e| AppError::Internal(format!("Parse batch embed failed: {}", e)))?;

            resp_model = openai_resp.model;
            let mut data = openai_resp.data;
            data.sort_by_key(|d| d.index);
            for d in data {
                if dim == 0 { dim = d.embedding.len(); }
                all_vectors.push(d.embedding);
            }
        }

        tracing::info!(
            provider = %self.name,
            model = %resp_model,
            count = all_vectors.len(),
            dim,
            "Batch embedding done"
        );

        Ok(BatchEmbeddingResponse {
            vectors: all_vectors,
            model: resp_model,
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
