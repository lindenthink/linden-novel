use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorldviewEntry {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub order_index: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorldviewEntry {
    pub project_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorldviewEntry {
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}
