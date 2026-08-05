use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::ai_generation::{
    GenerationContext, CharacterSummary, StorylineSummary, WorldviewSummary,
};
use crate::db::repo::{chapter_repo, character_repo, storyline_repo, worldview_repo, chapter_element_repo};

pub async fn collect_context(
    pool: &SqlitePool,
    chapter_id: &str,
) -> Result<GenerationContext, AppError> {
    // 获取章节信息
    let chapter = chapter_repo::get(pool, chapter_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get chapter: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Chapter not found".to_string()))?;

    // 获取章节内容
    let content = sqlx::query_as::<_, (String,)>(
        "SELECT content_text FROM chapter_contents WHERE chapter_id = ?"
    )
    .bind(chapter_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to get chapter content: {}", e)))?
    .map(|(text,)| text)
    .unwrap_or_default();

    // 获取章节关联的元素
    let elements = chapter_element_repo::list_by_chapter(pool, chapter_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list chapter elements: {}", e)))?;

    // 收集角色信息
    let mut characters = Vec::new();
    for element in &elements {
        if element.element_type == "character" {
            if let Ok(Some(character)) = character_repo::get(pool, &element.element_id).await {
                characters.push(CharacterSummary {
                    name: character.name,
                    description: character.description,
                    personality: character.role,
                });
            }
        }
    }

    // 收集情节线索
    let mut storylines = Vec::new();
    for element in &elements {
        if element.element_type == "storyline" {
            if let Ok(Some(storyline)) = storyline_repo::get(pool, &element.element_id).await {
                storylines.push(StorylineSummary {
                    title: storyline.name,
                    description: storyline.description,
                });
            }
        }
    }

    // 收集世界观设定
    let mut worldviews = Vec::new();
    for element in &elements {
        if element.element_type == "worldview" {
            if let Ok(Some(worldview)) = worldview_repo::get(pool, &element.element_id).await {
                worldviews.push(WorldviewSummary {
                    name: worldview.name,
                    description: worldview.description,
                });
            }
        }
    }

    // 获取前后章节摘要（简化实现，实际可以从章节摘要字段获取）
    let previous_chapter_summary = None;
    let next_chapter_summary = None;

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
    })
}
