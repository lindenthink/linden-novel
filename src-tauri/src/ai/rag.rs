use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;

use crate::db::repo::{character_repo, chapter_repo, storyline_repo, worldview_repo};
use crate::error::AppError;
use crate::models::embedding::RetrievedItem;
use crate::services::{chunk_embedding_service, embedding_service};

/// RAG 检索命中的上下文片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagContext {
    pub related_chapter_summaries: Vec<RagChapterSummary>,
    pub related_characters: Vec<RagCharacter>,
    pub related_storylines: Vec<RagStoryline>,
    pub related_worldviews: Vec<RagWorldview>,
    /// 切片级：章节正文片段（更细粒度的相关内容）
    pub related_chunks: Vec<RagChunk>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub chapter_id: String,
    pub chunk_index: i32,
    pub chapter_title: String,
    pub chunk_text: String,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct RagConfig {
    pub top_k: usize,
    pub min_score: f32,
    pub exclude_chapter_ids: Vec<String>,
    pub exclude_element_ids: Vec<String>,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: 3,
            min_score: 0.3,
            exclude_chapter_ids: Vec::new(),
            exclude_element_ids: Vec::new(),
        }
    }
}

/// 执行 RAG 检索
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
            related_chunks: Vec::new(),
        });
    }

    // 一次 embed，复用给摘要级 + 切片级检索，避免对同一 query 重复推理
    let embed_start = std::time::Instant::now();
    let provider = crate::ai::provider_factory::get_local_embedder(app_data_dir)?;
    let query_vec = provider
        .embed(crate::ai::provider::EmbeddingRequest {
            model: String::new(),
            input: query.to_string(),
        })
        .await?
        .vector;
    tracing::info!("RAG query embedded: {} dims, {:?}", query_vec.len(), embed_start.elapsed());

    // 并行执行摘要级 + 切片级 vec0 查询
    // 两者无依赖，串行会让总耗时 = summary_search + chunk_search
    let search_k = config.top_k.saturating_mul(4).max(config.top_k);
    let search_start = std::time::Instant::now();
    let (results, chunk_results) = tokio::join!(
        embedding_service::search_by_vector(pool, project_id, &query_vec, search_k),
        chunk_embedding_service::search_chunks_by_vector(
            pool,
            project_id,
            &query_vec,
            config.top_k * 2,
            &config.exclude_chapter_ids,
        ),
    );
    tracing::info!("RAG vector search done: {:?}", search_start.elapsed());

    let results = results?;
    let chunk_results = chunk_results.unwrap_or_else(|e| {
        tracing::warn!("Chunk RAG skipped (continuing without): {}", e);
        Vec::new()
    });

    let mut chapter_hits: Vec<&RetrievedItem> = Vec::new();
    let mut character_hits: Vec<&RetrievedItem> = Vec::new();
    let mut storyline_hits: Vec<&RetrievedItem> = Vec::new();
    let mut worldview_hits: Vec<&RetrievedItem> = Vec::new();

    for item in &results {
        if item.score < config.min_score {
            continue;
        }
        if config.exclude_element_ids.contains(&item.source_id) {
            continue;
        }
        if item.source_type == "chapter"
            && config.exclude_chapter_ids.contains(&item.source_id)
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

    // 并行加载 chapter/character/storyline/worldview 详情（无依赖）
    let detail_start = std::time::Instant::now();

    let chapter_futs: Vec<_> = chapter_hits
        .iter()
        .map(|hit| async move {
            chapter_repo::get(pool, &hit.source_id).await.ok().flatten().map(|ch| (ch, hit.score))
        })
        .collect();
    let character_futs: Vec<_> = character_hits
        .iter()
        .map(|hit| async move {
            character_repo::get(pool, &hit.source_id).await.ok().flatten().map(|c| (c, hit.score))
        })
        .collect();
    let storyline_futs: Vec<_> = storyline_hits
        .iter()
        .map(|hit| async move {
            storyline_repo::get(pool, &hit.source_id).await.ok().flatten().map(|s| (s, hit.score))
        })
        .collect();
    let worldview_futs: Vec<_> = worldview_hits
        .iter()
        .map(|hit| async move {
            worldview_repo::get(pool, &hit.source_id).await.ok().flatten().map(|w| (w, hit.score))
        })
        .collect();

    let (chapter_res, character_res, storyline_res, worldview_res) = tokio::join!(
        futures::future::join_all(chapter_futs),
        futures::future::join_all(character_futs),
        futures::future::join_all(storyline_futs),
        futures::future::join_all(worldview_futs),
    );

    let mut related_chapter_summaries = Vec::new();
    for (chapter, score) in chapter_res.into_iter().flatten() {
        if let Some(summary) = &chapter.summary {
            if !summary.trim().is_empty() {
                related_chapter_summaries.push(RagChapterSummary {
                    chapter_id: chapter.id,
                    title: chapter.title,
                    summary: summary.clone(),
                    score,
                });
            }
        }
    }

    let mut related_characters = Vec::new();
    for (character, score) in character_res.into_iter().flatten() {
        related_characters.push(RagCharacter {
            id: character.id,
            name: character.name,
            description: character.description,
            score,
        });
    }

    let mut related_storylines = Vec::new();
    for (storyline, score) in storyline_res.into_iter().flatten() {
        related_storylines.push(RagStoryline {
            id: storyline.id,
            name: storyline.name,
            description: storyline.description,
            score,
        });
    }

    let mut related_worldviews = Vec::new();
    for (worldview, score) in worldview_res.into_iter().flatten() {
        related_worldviews.push(RagWorldview {
            id: worldview.id,
            name: worldview.name,
            description: worldview.description,
            score,
        });
    }

    tracing::info!("RAG detail load done: {:?}", detail_start.elapsed());

    // 切片结果（已在外层并行查完，这里只做 chapter_title 回查）
    let mut related_chunks = Vec::with_capacity(chunk_results.len());
    for c in chunk_results {
        if c.score < config.min_score {
            continue;
        }
        let chapter = match chapter_repo::get(pool, &c.chapter_id).await {
            Ok(Some(ch)) => ch,
            _ => {
                tracing::warn!(
                    "Skipping orphan chunk: chapter {} not found",
                    c.chapter_id
                );
                continue;
            }
        };
        related_chunks.push(RagChunk {
            chapter_id: c.chapter_id,
            chunk_index: c.chunk_index,
            chapter_title: chapter.title,
            chunk_text: c.chunk_text,
            score: c.score,
        });
    }
    related_chunks.truncate(config.top_k);

    Ok(RagContext {
        related_chapter_summaries,
        related_characters,
        related_storylines,
        related_worldviews,
        related_chunks,
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

    // 切片级（细粒度片段）放在最前面信息密度最高
    if !ctx.related_chunks.is_empty() {
        out.push_str("## 相关章节片段（基于切片语义检索）\n");
        for c in &ctx.related_chunks {
            let preview: String = c.chunk_text.chars().take(250).collect();
            let more = if c.chunk_text.chars().count() > 250 { "…" } else { "" };
            out.push_str(&format!(
                "- 《{}》 片段#{}（相似度 {:.2}）：{}{}\n",
                c.chapter_title, c.chunk_index, c.score, preview, more
            ));
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
            related_chunks: Vec::new(),
        };
        let rendered = render_rag_context(&ctx);
        assert!(rendered.is_empty());
    }

    #[test]
    fn test_render_with_chunks_and_chapters() {
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
            related_chunks: vec![RagChunk {
                chapter_id: "c2".into(),
                chunk_index: 2,
                chapter_title: "第二章".into(),
                chunk_text: "这是主角遇到反派的片段内容".into(),
                score: 0.85,
            }],
        };
        let rendered = render_rag_context(&ctx);
        assert!(rendered.contains("相关章节摘要"));
        assert!(rendered.contains("相关章节片段"));
        assert!(rendered.contains("主角遇到反派"));
    }
}
