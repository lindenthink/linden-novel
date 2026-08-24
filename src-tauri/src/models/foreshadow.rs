use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Foreshadow {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub importance: String,
    pub status: String,
    pub plant_chapter_id: Option<String>,
    pub resolve_chapter_id: Option<String>,
    pub plant_note: Option<String>,
    pub resolve_note: Option<String>,
    pub order_index: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateForeshadow {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub importance: Option<String>,
    pub status: Option<String>,
    pub plant_chapter_id: Option<String>,
    pub resolve_chapter_id: Option<String>,
    pub plant_note: Option<String>,
    pub resolve_note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateForeshadow {
    pub title: Option<String>,
    pub description: Option<String>,
    pub importance: Option<String>,
    pub status: Option<String>,
    pub plant_chapter_id: Option<String>,
    pub resolve_chapter_id: Option<String>,
    pub plant_note: Option<String>,
    pub resolve_note: Option<String>,
}
