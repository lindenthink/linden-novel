use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiApiKey {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub encrypted_key: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAiApiKey {
    pub provider_id: String,
    pub name: String,
    pub api_key: String, // 明文密钥，会在 service 层加密
    pub is_default: Option<bool>,
}
