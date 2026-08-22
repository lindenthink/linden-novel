use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub content: String,
    pub variables_json: Option<String>,
    pub description: Option<String>,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePromptTemplate {
    pub id: Option<String>,
    pub name: String,
    pub template_type: String,
    pub content: String,
    pub variables_json: Option<String>,
    pub description: Option<String>,
    pub is_builtin: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePromptTemplate {
    pub name: Option<String>,
    pub template_type: Option<String>,
    pub content: Option<String>,
    pub variables_json: Option<String>,
    pub description: Option<String>,
    pub is_builtin: Option<bool>,
}
