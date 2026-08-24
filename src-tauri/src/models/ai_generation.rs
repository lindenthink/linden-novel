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
    /// 期望章节字数（作为 prompt 引导，而非硬性 token 限制）
    /// 反序列化时兼容旧字段名 "max_tokens"
    #[serde(default, alias = "max_tokens")]
    pub target_words: Option<i32>,
    pub temperature: Option<f32>,
    pub style: Option<String>,
    /// 叙事规则约束程度："loose"（宽松）或 "strict"（严格），默认 "strict"
    #[serde(default)]
    pub constraint: Option<String>,
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
    /// 本章需埋下的伏笔（plant_chapter_id = 当前章）
    #[serde(default)]
    pub foreshadows_to_plant: Vec<ForeshadowSummary>,
    /// 本章可回收的伏笔（status=planted 未回收，或 resolve_chapter_id = 当前章）
    #[serde(default)]
    pub foreshadows_to_resolve: Vec<ForeshadowSummary>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeshadowSummary {
    pub title: String,
    pub description: Option<String>,
    pub importance: String,
    pub plant_note: Option<String>,
    pub resolve_note: Option<String>,
}
