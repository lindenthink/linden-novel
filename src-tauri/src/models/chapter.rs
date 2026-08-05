use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chapter {
    pub id: String,
    pub volume_id: String,
    pub project_id: String,
    pub title: String,
    pub order_index: i32,
    pub status: String,
    pub word_count: i64,
    pub summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChapter {
    pub volume_id: String,
    pub project_id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChapterMeta {
    pub title: Option<String>,
    pub status: Option<String>,
    pub summary: Option<String>,
}
