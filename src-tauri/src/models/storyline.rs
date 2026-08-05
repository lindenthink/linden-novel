use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Storyline {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub order_index: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStoryline {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStoryline {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}
