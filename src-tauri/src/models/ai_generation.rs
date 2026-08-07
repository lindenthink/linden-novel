use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiGenerationHistory {
    pub id: String,
    pub chapter_id: String,
    pub mode: String,
    pub input_context: String,
    pub output_content: String,
    pub parameters_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParameters {
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAiGeneration {
    pub chapter_id: String,
    pub mode: String,
    pub input_context: String,
    pub output_content: String,
    pub parameters: GenerationParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext {
    pub chapter_id: String,
    pub chapter_title: String,
    pub chapter_summary: Option<String>,
    pub chapter_content: String,
    pub characters: Vec<CharacterSummary>,
    pub storylines: Vec<StorylineSummary>,
    pub worldviews: Vec<WorldviewSummary>,
    pub previous_chapter_summary: Option<String>,
    pub next_chapter_summary: Option<String>,
    /// RAG 检索到的相关上下文（渲染为 Prompt 片段）
    #[serde(default)]
    pub rag_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub name: String,
    pub description: Option<String>,
    pub personality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorylineSummary {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldviewSummary {
    pub name: String,
    pub description: Option<String>,
}
