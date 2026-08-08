use sqlx::SqlitePool;
use std::path::Path;

use crate::ai::rag::{self, RagConfig};
use crate::db::repo::{chapter_element_repo, chapter_repo, character_repo, storyline_repo, worldview_repo};
use crate::error::AppError;
use crate::models::ai_generation::{
    CharacterSummary, GenerationContext, StorylineSummary, WorldviewSummary,
};
use crate::models::chapter::Chapter;

/// 获取相邻章节的摘要
///
/// # 逻辑
/// - 前一章：同卷 order_index 最大且小于当前章的章节
/// - 后一章：同卷 order_index 最小且大于当前章的章节
/// - 跨卷不回溯（保持简单，避免引入复杂排序）
async fn get_adjacent_summaries(
    pool: &SqlitePool,
    chapter: &Chapter,
) -> Result<(Option<String>, Option<String>), AppError> {
    // 前一章：order_index < current 且最大
    let prev: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT summary FROM chapters
         WHERE volume_id = ? AND order_index < ?
         ORDER BY order_index DESC LIMIT 1",
    )
    .bind(&chapter.volume_id)
    .bind(chapter.order_index)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    let previous = prev
        .and_then(|(s,)| s)
        .filter(|s| !s.trim().is_empty());

    // 后一章：order_index > current 且最小
    let next: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT summary FROM chapters
         WHERE volume_id = ? AND order_index > ?
         ORDER BY order_index ASC LIMIT 1",
    )
    .bind(&chapter.volume_id)
    .bind(chapter.order_index)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    let next_summary = next
        .and_then(|(s,)| s)
        .filter(|s| !s.trim().is_empty());

    Ok((previous, next_summary))
}

/// 收集章节生成上下文（含前后章节摘要 + RAG 检索）
///
/// # 流程
/// 1. 读取章节元信息和正文
/// 2. 收集章节关联的元素（角色/故事线/世界观）
/// 3. 查询前后章节摘要
/// 4. 用章节标题+摘要作为 query 执行 RAG 检索
/// 5. 渲染 RAG 上下文为 Prompt 片段
pub async fn collect_context(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<GenerationContext, AppError> {
    collect_context_with_rag(pool, None, chapter_id).await
}

/// 带应用数据目录的上下文收集（启用 RAG）
pub async fn collect_context_with_rag(
    pool: &SqlitePool,
    app_data_dir: Option<&Path>,
    chapter_id: &str,
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
    let (previous_chapter_summary, next_chapter_summary) =
        get_adjacent_summaries(pool, &chapter).await?;

    // 5. RAG 检索
    let rag_context = if let Some(dir) = app_data_dir {
        let query = build_rag_query(&chapter);
        if query.trim().is_empty() {
            None
        } else {
            let config = RagConfig {
                exclude_chapter_id: Some(chapter_id.to_string()),
                exclude_element_ids,
                ..Default::default()
            };
            match rag::retrieve(pool, dir, &chapter.project_id, &query, &config).await {
                Ok(rag_ctx) => {
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
    })
}

/// 构建 RAG 检索的 query
fn build_rag_query(chapter: &Chapter) -> String {
    let mut q = chapter.title.clone();
    if let Some(summary) = &chapter.summary {
        if !summary.trim().is_empty() {
            if !q.is_empty() {
                q.push(' ');
            }
            q.push_str(summary);
        }
    }
    q
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
        let q = build_rag_query(&ch);
        assert!(q.contains("第三章 风暴"));
        assert!(q.contains("主角遭遇海上风暴"));
    }

    #[test]
    fn test_build_rag_query_without_summary() {
        let ch = make_chapter("第一章", None);
        let q = build_rag_query(&ch);
        assert_eq!(q, "第一章");
    }

    #[test]
    fn test_build_rag_query_empty_summary() {
        let ch = make_chapter("第二章", Some("   "));
        let q = build_rag_query(&ch);
        assert_eq!(q, "第二章");
    }
}
