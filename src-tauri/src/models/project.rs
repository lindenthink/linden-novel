use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub genre: Option<String>,
    pub summary: Option<String>,
    pub target_words: Option<i64>,
    pub settings_json: Option<String>,
    pub cover_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub title: String,
    pub genre: Option<String>,
    pub summary: Option<String>,
    pub target_words: Option<i64>,
    pub cover_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub title: Option<String>,
    pub genre: Option<String>,
    pub summary: Option<String>,
    pub target_words: Option<i64>,
    pub settings_json: Option<String>,
    pub cover_path: Option<String>,
}
