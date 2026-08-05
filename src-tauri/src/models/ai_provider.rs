use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub models_json: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAiProvider {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub models_json: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAiProvider {
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub base_url: Option<String>,
    pub models_json: Option<String>,
    pub is_default: Option<bool>,
}
