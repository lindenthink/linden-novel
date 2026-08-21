use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 嵌入向量的来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum EmbeddingSourceType {
    Chapter,
    Character,
    Storyline,
    Worldview,
}

impl EmbeddingSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chapter => "chapter",
            Self::Character => "character",
            Self::Storyline => "storyline",
            Self::Worldview => "worldview",
        }
    }
}

impl std::fmt::Display for EmbeddingSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EmbeddingSourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chapter" => Ok(Self::Chapter),
            "character" => Ok(Self::Character),
            "storyline" => Ok(Self::Storyline),
            "worldview" => Ok(Self::Worldview),
            other => Err(format!("unknown embedding source type: {}", other)),
        }
    }
}

/// 数据库行：embeddings 表
///
/// 字段镜像 DB schema；部分字段当前未在 Rust 侧读取，但保留以完整描述表结构。
#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct EmbeddingRow {
    pub id: String,
    pub project_id: String,
    pub source_type: String,
    pub source_id: String,
    pub content_hash: String,
    pub embedding: Vec<u8>,
    pub dim: i64,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建/更新嵌入
#[derive(Debug, Clone)]
pub struct UpsertEmbedding {
    pub project_id: String,
    pub source_type: EmbeddingSourceType,
    pub source_id: String,
    pub content_hash: String,
    pub embedding: Vec<f32>,
    pub model: String,
}

/// 检索命中项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedItem {
    pub source_type: String,
    pub source_id: String,
    pub score: f32,
}
