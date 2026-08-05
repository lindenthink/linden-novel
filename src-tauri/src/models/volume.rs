use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Volume {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub order_index: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateVolume {
    pub project_id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVolume {
    pub title: Option<String>,
}
