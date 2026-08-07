use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

use crate::db::repo::{character_repo, chapter_repo, storyline_repo, worldview_repo};
use crate::error::AppError;
use crate::models::embedding::RetrievedItem;
use crate::services::embedding_service;

/// RAG 检索命中的上下文片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagContext {
    /// 相关章节摘要（不含当前章节）
    pub related_chapter_summaries: Vec<RagChapterSummary>,
    /// 相关角色描述（不含已通过 chapter_elements 关联的）
    pub related_characters: Vec<RagCharacter>,
    /// 相关故事线描述
    pub related_storylines: Vec<RagStoryline>,
    /// 相关世界观设定
    pub related_worldviews: Vec<RagWorldview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChapterSummary {
    pub chapter_id: String,
    pub title: String,
    pub summary: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagCharacter {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagStoryline {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagWorldview {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub score: f32,
}

/// RAG 检索配置
#[derive(Debug, Clone)]
pub struct RagConfig {
    /// 检索的 top_k 数量（每种类型）
    pub top_k: usize,
    /// 最低相似度阈值（低于此值不纳入上下文）
    pub min_score: f32,
    /// 要排除的章节 ID（通常是当前章节）
    pub exclude_chapter_id: Option<String>,
    /// 要排除的元素 ID 集合（已通过 chapter_elements 关联的）
    pub exclude_element_ids: Vec<String>,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: 3,
            min_score: 0.3,
            exclude_chapter_id: None,
            exclude_element_ids: Vec::new(),
        }
    }
}

/// 执行 RAG 检索，返回相关上下文片段
///
/// # 流程
/// 1. 将 query 文本嵌入为向量
/// 2. 在项目向量库中搜索 top_k 相似项
/// 3. 按类型分组，过滤已关联元素和当前章节
/// 4. 从 DB 加载完整内容
pub async fn retrieve(
    pool: &SqlitePool,
    app_data_dir: &Path,
    project_id: &str,
    query: &str,
    config: &RagConfig,
) -> Result<RagContext, AppError> {
    if query.trim().is_empty() {
        return Ok(RagContext {
            related_chapter_summaries: Vec::new(),
            related_characters: Vec::new(),
            related_storylines: Vec::new(),
            related_worldviews: Vec::new(),
        });
    }

    // 向量搜索：top_k * 4 确保过滤后每类仍有足够数量
    let search_k = config.top_k.saturating_mul(4).max(config.top_k);
    let results =
        embedding_service::search(pool, app_data_dir, project_id, query, search_k).await?;

    let mut chapter_hits: Vec<&RetrievedItem> = Vec::new();
    let mut character_hits: Vec<&RetrievedItem> = Vec::new();
    let mut storyline_hits: Vec<&RetrievedItem> = Vec::new();
    let mut worldview_hits: Vec<&RetrievedItem> = Vec::new();

    for item in &results {
        if item.score < config.min_score {
            continue;
        }
        // 排除已关联的元素
        if config.exclude_element_ids.contains(&item.source_id) {
            continue;
        }
        // 排除当前章节
        if item.source_type == "chapter"
            && config.exclude_chapter_id.as_deref() == Some(&item.source_id)
        {
            continue;
        }

        match item.source_type.as_str() {
            "chapter" => {
                if chapter_hits.len() < config.top_k {
                    chapter_hits.push(item);
                }
            }
            "character" => {
                if character_hits.len() < config.top_k {
                    character_hits.push(item);
                }
            }
            "storyline" => {
                if storyline_hits.len() < config.top_k {
                    storyline_hits.push(item);
                }
            }
            "worldview" => {
                if worldview_hits.len() < config.top_k {
                    worldview_hits.push(item);
                }
            }
            _ => {}
        }
    }

    // 从 DB 加载完整内容
    let mut related_chapter_summaries = Vec::new();
    for hit in chapter_hits {
        if let Ok(Some(chapter)) = chapter_repo::get(pool, &hit.source_id).await {
            if let Some(summary) = &chapter.summary {
                if !summary.trim().is_empty() {
                    related_chapter_summaries.push(RagChapterSummary {
                        chapter_id: chapter.id,
                        title: chapter.title,
                        summary: summary.clone(),
                        score: hit.score,
                    });
                }
            }
        }
    }

    let mut related_characters = Vec::new();
    for hit in character_hits {
        if let Ok(Some(character)) = character_repo::get(pool, &hit.source_id).await {
            related_characters.push(RagCharacter {
                id: character.id,
                name: character.name,
                description: character.description,
                score: hit.score,
            });
        }
    }

    let mut related_storylines = Vec::new();
    for hit in storyline_hits {
        if let Ok(Some(storyline)) = storyline_repo::get(pool, &hit.source_id).await {
            related_storylines.push(RagStoryline {
                id: storyline.id,
                name: storyline.name,
                description: storyline.description,
                score: hit.score,
            });
        }
    }

    let mut related_worldviews = Vec::new();
    for hit in worldview_hits {
        if let Ok(Some(worldview)) = worldview_repo::get(pool, &hit.source_id).await {
            related_worldviews.push(RagWorldview {
                id: worldview.id,
                name: worldview.name,
                description: worldview.description,
                score: hit.score,
            });
        }
    }

    Ok(RagContext {
        related_chapter_summaries,
        related_characters,
        related_storylines,
        related_worldviews,
    })
}

/// 将 RAG 上下文渲染为 Prompt 片段
pub fn render_rag_context(ctx: &RagContext) -> String {
    let mut out = String::new();

    if !ctx.related_chapter_summaries.is_empty() {
        out.push_str("## 相关章节摘要（基于语义检索）\n");
        for s in &ctx.related_chapter_summaries {
            out.push_str(&format!("- 《{}》：{}\n", s.title, s.summary));
        }
        out.push('\n');
    }

    if !ctx.related_characters.is_empty() {
        out.push_str("## 相关角色（基于语义检索）\n");
        for c in &ctx.related_characters {
            out.push_str(&format!("- {}", c.name));
            if let Some(desc) = &c.description {
                out.push_str(&format!("：{}", desc));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !ctx.related_storylines.is_empty() {
        out.push_str("## 相关情节线索（基于语义检索）\n");
        for s in &ctx.related_storylines {
            out.push_str(&format!("- {}", s.name));
            if let Some(desc) = &s.description {
                out.push_str(&format!("：{}", desc));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !ctx.related_worldviews.is_empty() {
        out.push_str("## 相关世界观设定（基于语义检索）\n");
        for w in &ctx.related_worldviews {
            out.push_str(&format!("- {}", w.name));
            if let Some(desc) = &w.description {
                out.push_str(&format!("：{}", desc));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_context() {
        let ctx = RagContext {
            related_chapter_summaries: Vec::new(),
            related_characters: Vec::new(),
            related_storylines: Vec::new(),
            related_worldviews: Vec::new(),
        };
        let rendered = render_rag_context(&ctx);
        assert!(rendered.is_empty());
    }

    #[test]
    fn test_render_with_chapters() {
        let ctx = RagContext {
            related_chapter_summaries: vec![RagChapterSummary {
                chapter_id: "c1".into(),
                title: "第一章".into(),
                summary: "主角登场".into(),
                score: 0.9,
            }],
            related_characters: Vec::new(),
            related_storylines: Vec::new(),
            related_worldviews: Vec::new(),
        };
        let rendered = render_rag_context(&ctx);
        assert!(rendered.contains("相关章节摘要"));
        assert!(rendered.contains("第一章"));
        assert!(rendered.contains("主角登场"));
    }

    #[test]
    fn test_render_with_all_types() {
        let ctx = RagContext {
            related_chapter_summaries: vec![RagChapterSummary {
                chapter_id: "c1".into(),
                title: "章".into(),
                summary: "摘要".into(),
                score: 0.9,
            }],
            related_characters: vec![RagCharacter {
                id: "ch1".into(),
                name: "角色A".into(),
                description: Some("描述".into()),
                score: 0.8,
            }],
            related_storylines: vec![RagStoryline {
                id: "s1".into(),
                name: "线索X".into(),
                description: None,
                score: 0.7,
            }],
            related_worldviews: vec![RagWorldview {
                id: "w1".into(),
                name: "世界Y".into(),
                description: Some("设定".into()),
                score: 0.6,
            }],
        };
        let rendered = render_rag_context(&ctx);
        assert!(rendered.contains("相关章节摘要"));
        assert!(rendered.contains("相关角色"));
        assert!(rendered.contains("相关情节线索"));
        assert!(rendered.contains("相关世界观设定"));
        assert!(rendered.contains("角色A"));
        assert!(rendered.contains("线索X"));
        assert!(rendered.contains("世界Y"));
    }

    #[test]
    fn test_rag_config_default() {
        let config = RagConfig::default();
        assert_eq!(config.top_k, 3);
        assert!((config.min_score - 0.3).abs() < 1e-6);
        assert!(config.exclude_chapter_id.is_none());
        assert!(config.exclude_element_ids.is_empty());
    }
}
