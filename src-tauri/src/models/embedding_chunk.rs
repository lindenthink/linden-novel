use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 切片嵌入数据库行
#[derive(Debug, Clone, FromRow)]
pub struct EmbeddingChunkRow {
    pub id: String,
    pub project_id: String,
    pub chapter_id: String,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub char_count: i64,
    pub content_hash: String,
    pub embedding: Vec<u8>,
    pub dim: i64,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建/更新切片嵌入输入
#[derive(Debug, Clone)]
pub struct UpsertChunk {
    pub project_id: String,
    pub chapter_id: String,
    pub chunk_index: usize,
    pub chunk_text: String,
    pub content_hash: String,
    pub embedding: Vec<f32>,
    pub model: String,
}

/// 检索命中的切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk {
    pub chapter_id: String,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub score: f32,
}
