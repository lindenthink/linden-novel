use crate::ai::provider::StreamChunk;
use crate::error::AppError;
use futures::stream::Stream;
use std::pin::Pin;

/// SSE 事件类型
#[derive(Debug, Clone)]
pub enum SseEvent {
    /// 数据块
    Data(String),
    /// 流结束
    Done,
    /// 错误
    Error(String),
}

/// 解析 SSE 格式的字节流
/// 
/// SSE 格式：
/// ```
/// data: {"content": "hello"}
/// 
/// data: {"content": " world"}
/// 
/// data: [DONE]
/// ```
pub fn parse_sse_line(line: &str) -> Option<SseEvent> {
    let line = line.trim();
    
    if line.is_empty() {
        return None;
    }
    
    if line.starts_with("data: ") {
        let data = &line[6..];
        
        if data == "[DONE]" {
            return Some(SseEvent::Done);
        }
        
        return Some(SseEvent::Data(data.to_string()));
    }
    
    if line.starts_with("event: error") {
        return Some(SseEvent::Error("Stream error".to_string()));
    }
    
    None
}

/// 将 SSE 字节流转换为 StreamChunk 流
pub async fn sse_stream_to_chunks(
    byte_stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
) -> Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>> {
    use futures::StreamExt;
    
    let mut buffer = String::new();
    
    let stream = byte_stream
        .map(move |chunk_result| {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));
                    
                    // 按行解析
                    let mut chunks = Vec::new();
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();
                        
                        if let Some(event) = parse_sse_line(&line) {
                            match event {
                                SseEvent::Data(data) => {
                                    // 尝试解析 JSON
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                                        if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                                            chunks.push(Ok(StreamChunk {
                                                content: content.to_string(),
                                                done: false,
                                            }));
                                        }
                                    }
                                }
                                SseEvent::Done => {
                                    chunks.push(Ok(StreamChunk {
                                        content: String::new(),
                                        done: true,
                                    }));
                                }
                                SseEvent::Error(msg) => {
                                    chunks.push(Err(AppError::Internal(msg)));
                                }
                            }
                        }
                    }
                    
                    futures::stream::iter(chunks)
                }
                Err(e) => {
                    futures::stream::iter(vec![Err(AppError::Internal(format!("Stream error: {}", e)))])
                }
            }
        })
        .flatten();
    
    Box::pin(stream)
}
