use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Inspiration {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub tag: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInspiration {
    pub project_id: String,
    pub content: String,
    pub tag: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInspiration {
    pub content: Option<String>,
    pub tag: Option<String>,
    pub status: Option<String>,
}
