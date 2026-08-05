use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Character {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub role: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub order_index: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCharacter {
    pub project_id: String,
    pub name: String,
    pub role: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacter {
    pub name: Option<String>,
    pub role: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
}
