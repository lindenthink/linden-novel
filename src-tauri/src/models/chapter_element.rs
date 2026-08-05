use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChapterElement {
    pub id: String,
    pub chapter_id: String,
    pub element_type: String,
    pub element_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChapterElement {
    pub chapter_id: String,
    pub element_type: String,
    pub element_id: String,
}
