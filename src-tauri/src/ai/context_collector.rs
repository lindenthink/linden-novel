use sqlx::SqlitePool;
use std::path::Path;

use crate::ai::rag::{self, RagConfig};
use crate::db::repo::{chapter_element_repo, chapter_repo, character_repo, foreshadow_repo, storyline_repo, worldview_repo};
use crate::error::AppError;
use crate::models::ai_generation::{
    CharacterSummary, ForeshadowSummary, GenerationContext, StorylineSummary, WorldviewSummary,
};
use crate::models::chapter::Chapter;

/// 相邻章节信息（含 id 和摘要）
///
/// id 用于 RAG 检索排除（前一章摘要已作为独立字段拼进 prompt，避免在检索结果中重复）；
/// summary 用于 prompt 的「相邻章节摘要」段落。
#[derive(Debug, Clone)]
struct AdjacentChapter {
    id: String,
    summary: String,
}

/// 获取相邻章节的摘要
///
/// # 逻辑
/// - 前一章：同卷 order_index 最大且小于当前章的章节
/// - 后一章：同卷 order_index 最小且大于当前章的章节
/// - 跨卷不回溯（保持简单，避免引入复杂排序）
async fn get_adjacent_summaries(
    pool: &SqlitePool,
    chapter: &Chapter,
) -> Result<(Option<AdjacentChapter>, Option<AdjacentChapter>), AppError> {
    // 前一章：order_index < current 且最大
    let prev: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, summary FROM chapters
         WHERE volume_id = ? AND order_index < ?
         ORDER BY order_index DESC LIMIT 1",
    )
    .bind(&chapter.volume_id)
    .bind(chapter.order_index)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    let previous = prev.and_then(|(id, s)| {
        s.filter(|s| !s.trim().is_empty())
            .map(|summary| AdjacentChapter { id, summary })
    });

    // 后一章：order_index > current 且最小
    let next: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT id, summary FROM chapters
         WHERE volume_id = ? AND order_index > ?
         ORDER BY order_index ASC LIMIT 1",
    )
    .bind(&chapter.volume_id)
    .bind(chapter.order_index)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    let next_chapter = next.and_then(|(id, s)| {
        s.filter(|s| !s.trim().is_empty())
            .map(|summary| AdjacentChapter { id, summary })
    });

    Ok((previous, next_chapter))
}

/// 收集章节生成上下文（含前后章节摘要 + RAG 检索）
///
/// # 流程
/// 1. 读取章节元信息和正文
/// 2. 收集章节关联的元素（角色/故事线/世界观）
/// 3. 查询前后章节摘要
/// 4. 用章节标题+摘要作为 query 执行 RAG 检索
/// 5. 渲染 RAG 上下文为 Prompt 片段
pub async fn collect_context_with_rag_and_instruction(
    pool: &SqlitePool,
    app_data_dir: Option<&Path>,
    chapter_id: &str,
    user_instruction: Option<&str>,
) -> Result<GenerationContext, AppError> {
    // 1. 章节信息
    let chapter = chapter_repo::get(pool, chapter_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Chapter not found".to_string()))?;

    // 2. 章节正文
    let content: String = sqlx::query_as::<_, (String,)>(
        "SELECT content_text FROM chapter_contents WHERE chapter_id = ?",
    )
    .bind(chapter_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?
    .map(|(t,)| t)
    .unwrap_or_default();

    // 3. 关联元素
    let elements = chapter_element_repo::list_by_chapter(pool, chapter_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list chapter elements: {}", e)))?;

    let mut characters = Vec::new();
    let mut storylines = Vec::new();
    let mut worldviews = Vec::new();
    let mut exclude_element_ids: Vec<String> = Vec::new();

    for element in &elements {
        exclude_element_ids.push(element.element_id.clone());
        match element.element_type.as_str() {
            "character" => {
                if let Ok(Some(c)) = character_repo::get(pool, &element.element_id).await {
                    characters.push(CharacterSummary {
                        name: c.name,
                        description: c.description,
                        personality: c.role,
                    });
                }
            }
            "storyline" => {
                if let Ok(Some(s)) = storyline_repo::get(pool, &element.element_id).await {
                    storylines.push(StorylineSummary {
                        title: s.name,
                        description: s.description,
                    });
                }
            }
            "worldview" => {
                if let Ok(Some(w)) = worldview_repo::get(pool, &element.element_id).await {
                    worldviews.push(WorldviewSummary {
                        name: w.name,
                        description: w.description,
                    });
                }
            }
            _ => {}
        }
    }

    // 如果章节没有关联角色，自动包含项目中的主要角色（按 order_index 排序的前 3 个）
    if characters.is_empty() {
        tracing::info!("Chapter has no associated characters, loading project's main characters");
        if let Ok(project_characters) = character_repo::list_by_project(pool, &chapter.project_id).await {
            for c in project_characters.iter().take(3) {
                characters.push(CharacterSummary {
                    name: c.name.clone(),
                    description: c.description.clone(),
                    personality: c.role.clone(),
                });
            }
        }
    }

    // 4. 前后章节摘要
    let (prev_adjacent, next_adjacent) = get_adjacent_summaries(pool, &chapter).await?;
    let previous_chapter_summary = prev_adjacent.as_ref().map(|a| a.summary.clone());
    let next_chapter_summary = next_adjacent.as_ref().map(|a| a.summary.clone());

    // 5. RAG 检索
    // 排除当前章节 + 前一章：前一章摘要已作为独立字段拼进 prompt，避免在 rag_context 中重复
    let mut exclude_chapter_ids = vec![chapter_id.to_string()];
    if let Some(prev) = &prev_adjacent {
        exclude_chapter_ids.push(prev.id.clone());
    }

    let rag_context = if let Some(dir) = app_data_dir {
        let query = build_rag_query(&chapter, previous_chapter_summary.as_deref(), user_instruction);
        tracing::info!("RAG query: {:?}", query);
        if query.trim().is_empty() {
            None
        } else {
            let config = RagConfig {
                exclude_chapter_ids,
                exclude_element_ids,
                ..Default::default()
            };
            let rag_start = std::time::Instant::now();
            match rag::retrieve(pool, dir, &chapter.project_id, &query, &config).await {
                Ok(rag_ctx) => {
                    tracing::info!(
                        "RAG results: {} chapters, {} chunks, {} characters, {} storylines, {} worldviews (total {:?})",
                        rag_ctx.related_chapter_summaries.len(),
                        rag_ctx.related_chunks.len(),
                        rag_ctx.related_characters.len(),
                        rag_ctx.related_storylines.len(),
                        rag_ctx.related_worldviews.len(),
                        rag_start.elapsed()
                    );
                    let rendered = rag::render_rag_context(&rag_ctx);
                    if rendered.is_empty() {
                        None
                    } else {
                        Some(rendered)
                    }
                }
                Err(e) => {
                    tracing::warn!("RAG retrieval failed (continuing without): {}", e);
                    None
                }
            }
        }
    } else {
        None
    };

    // 6. 伏笔：本章需埋下的 + 本章可回收的
    let foreshadows_to_plant = match foreshadow_repo::list_to_plant_in_chapter(pool, chapter_id).await {
        Ok(list) => list.into_iter().map(|f| ForeshadowSummary {
            title: f.title,
            description: f.description,
            importance: f.importance,
            plant_note: f.plant_note,
            resolve_note: None,
        }).collect(),
        Err(e) => {
            tracing::warn!("Failed to load foreshadows to plant: {}", e);
            Vec::new()
        }
    };
    let foreshadows_to_resolve = match foreshadow_repo::list_resolvable_in_chapter(pool, chapter_id).await {
        Ok(list) => list.into_iter().map(|f| ForeshadowSummary {
            title: f.title,
            description: f.description,
            importance: f.importance,
            plant_note: None,
            resolve_note: f.resolve_note,
        }).collect(),
        Err(e) => {
            tracing::warn!("Failed to load foreshadows to resolve: {}", e);
            Vec::new()
        }
    };

    Ok(GenerationContext {
        chapter_id: chapter.id,
        chapter_title: chapter.title,
        chapter_summary: chapter.summary,
        chapter_content: content,
        characters,
        storylines,
        worldviews,
        previous_chapter_summary,
        next_chapter_summary,
        rag_context,
        foreshadows_to_plant,
        foreshadows_to_resolve,
    })
}

/// 构建 RAG 检索的 query
///
/// 续写新章节时，当前章节内容/摘要为空，仅靠标题语义信息不足。
/// 因此将前一章摘要 + 用户续写指令纳入 query，提供更丰富的语义线索。
fn build_rag_query(
    chapter: &Chapter,
    prev_summary: Option<&str>,
    user_instruction: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // 当前章节标题
    if !chapter.title.trim().is_empty() {
        parts.push(chapter.title.trim().to_string());
    }

    // 当前章节摘要（续写时通常为空）
    if let Some(summary) = &chapter.summary {
        if !summary.trim().is_empty() {
            parts.push(summary.trim().to_string());
        }
    }

    // 前一章摘要（续写时最重要的语义来源）
    if let Some(prev) = prev_summary {
        if !prev.trim().is_empty() {
            parts.push(prev.trim().to_string());
        }
    }

    // 用户续写指令
    if let Some(instruction) = user_instruction {
        if !instruction.trim().is_empty() {
            parts.push(instruction.trim().to_string());
        }
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chapter(title: &str, summary: Option<&str>) -> Chapter {
        Chapter {
            id: "test".into(),
            volume_id: "v".into(),
            project_id: "p".into(),
            title: title.into(),
            order_index: 0,
            status: "draft".into(),
            word_count: 0,
            summary: summary.map(String::from),
            created_at: "2026-01-01 00:00:00".into(),
            updated_at: "2026-01-01 00:00:00".into(),
        }
    }

    #[test]
    fn test_build_rag_query_with_summary() {
        let ch = make_chapter("第三章 风暴", Some("主角遭遇海上风暴"));
        let q = build_rag_query(&ch, None, None);
        assert!(q.contains("第三章 风暴"));
        assert!(q.contains("主角遭遇海上风暴"));
    }

    #[test]
    fn test_build_rag_query_without_summary() {
        let ch = make_chapter("第一章", None);
        let q = build_rag_query(&ch, None, None);
        assert_eq!(q, "第一章");
    }

    #[test]
    fn test_build_rag_query_empty_summary() {
        let ch = make_chapter("第二章", Some("   "));
        let q = build_rag_query(&ch, None, None);
        assert_eq!(q, "第二章");
    }

    #[test]
    fn test_build_rag_query_with_prev_and_instruction() {
        let ch = make_chapter("第八章", None);
        let q = build_rag_query(
            &ch,
            Some("小强携带短刀赶路"),
            Some("续写山中遭遇情节"),
        );
        assert!(q.contains("第八章"));
        assert!(q.contains("小强携带短刀赶路"));
        assert!(q.contains("续写山中遭遇情节"));
    }
}
