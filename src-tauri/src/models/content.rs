use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChapterContent {
    pub chapter_id: String,
    pub content_json: String,
    pub content_text: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveContent {
    pub chapter_id: String,
    pub content_json: String,
    pub content_text: String,
}
