use sqlx::SqlitePool;
use std::path::Path;

use crate::ai::provider::{CompletionRequest, Message};
use crate::ai::provider_factory;
use crate::db::repo::chapter_repo;
use crate::error::AppError;
use crate::models::chapter::Chapter;
use crate::services::{ai_api_key_service, ai_provider_service, embedding_service};

/// 摘要生成的目标长度（中文字符）
const SUMMARY_TARGET_CHARS: usize = 200;

/// 获取章节正文（content_text）
async fn get_chapter_text(pool: &SqlitePool, chapter_id: &str) -> Result<String, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT content_text FROM chapter_contents WHERE chapter_id = ?",
    )
    .bind(chapter_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    Ok(row.map(|(t,)| t).unwrap_or_default())
}

/// 构建摘要生成的系统提示
fn build_summary_system_prompt() -> String {
    format!(
        "你是一位专业的小说编辑助手。你的任务是生成简洁、准确的章节摘要。\n\n\
         要求：\n\
         1. 摘要长度约 {} 个中文字符\n\
         2. 概括章节的核心情节、关键事件、人物发展\n\
         3. 保持客观叙述，不加入评价\n\
         4. 直接输出摘要内容，不要包含任何解释、前缀或标记\n\
         5. 不要使用 Markdown 格式",
        SUMMARY_TARGET_CHARS
    )
}

/// 构建摘要生成的用户消息
fn build_summary_user_prompt(chapter: &Chapter, content: &str) -> String {
    let mut prompt = String::new();
    prompt.push_str(&format!("章节标题：{}\n\n", chapter.title));
    if !content.trim().is_empty() {
        prompt.push_str("章节正文：\n");
        prompt.push_str(content);
    } else {
        prompt.push_str("（章节正文为空）");
    }
    prompt
}

/// 为指定章节生成摘要
///
/// # 流程
/// 1. 读取章节正文
/// 2. 调用 AI 生成摘要
/// 3. 写入 chapters.summary 字段
/// 4. 触发嵌入生成（异步）
///
/// # 参数
/// - `chapter_id`: 章节 ID
/// - `force`: 是否强制重新生成（即使已有摘要）
///
/// # 返回
/// 生成的摘要文本
pub async fn generate_chapter_summary(
    pool: &SqlitePool,
    app_data_dir: &Path,
    chapter_id: &str,
    force: bool,
) -> Result<String, AppError> {
    // 1. 读取章节
    let chapter = chapter_repo::get(pool, chapter_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Chapter '{}' not found", chapter_id)))?;

    // 已有摘要且非强制 → 直接返回
    if !force {
        if let Some(existing) = &chapter.summary {
            if !existing.trim().is_empty() {
                return Ok(existing.clone());
            }
        }
    }

    // 2. 读取正文
    let content = get_chapter_text(pool, chapter_id).await?;

    if content.trim().is_empty() {
        return Err(AppError::Validation(
            "Cannot generate summary for empty chapter".into(),
        ));
    }

    // 3. 获取 provider + api key
    let provider = ai_provider_service::get_default(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("No default AI provider configured".to_string()))?;

    let default_key = ai_api_key_service::get_default_for_provider(pool, &provider.id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("No API key configured for default provider".to_string())
        })?;

    let api_key = ai_api_key_service::get_decrypted(pool, app_data_dir, &default_key.id).await?;
    let ai_provider = provider_factory::create_provider(&provider, &api_key)?;

    // 4. 使用模型
    let model = provider.models_json.clone();

    // 5. 构建请求
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: build_summary_system_prompt(),
        },
        Message {
            role: "user".to_string(),
            content: build_summary_user_prompt(&chapter, &content),
        },
    ];

    let request = CompletionRequest {
        model,
        messages,
        temperature: Some(0.3), // 摘要需要稳定输出
        // 不设置 max_tokens：推理模型（如 DeepSeek V4）默认开启 thinking，
        // 硬 token 限制可能被 thinking 阶段消耗后导致 content 为空。
        // 长度约束已通过 prompt 中的 SUMMARY_TARGET_CHARS 软性引导。
        max_tokens: None,
        stream: false,
    };

    // 6. 调用 AI
    let response = ai_provider.complete(request).await?;
    let summary = response.content.trim().to_string();

    if summary.is_empty() {
        return Err(AppError::Internal("AI returned empty summary".into()));
    }

    // 7. 写入 chapters.summary
    let update = crate::models::chapter::UpdateChapterMeta {
        title: None,
        status: None,
        summary: Some(summary.clone()),
    };
    chapter_repo::update_meta(pool, chapter_id, &update).await?;

    tracing::info!(
        "Generated summary for chapter {} ({} chars)",
        chapter_id,
        summary.chars().count()
    );

    // 8. 触发嵌入生成
    // if let Err(e) = embedding_service::generate_and_store(
    //     pool,
    //     app_data_dir,
    //     &chapter.project_id,
    //     "chapter",
    //     chapter_id,
    //     &summary,
    //     "",
    // )
    // .await
    // {
    //     tracing::warn!(
    //         "Failed to generate embedding for chapter summary {}: {} (summary still saved)",
    //         chapter_id,
    //         e
    //     );
    // }

    Ok(summary)
}

/// 批量为项目内所有无摘要章节生成摘要
///
/// # 返回
/// (成功数, 失败数)
pub async fn generate_all_summaries(
    pool: &SqlitePool,
    app_data_dir: &Path,
    project_id: &str,
) -> Result<(usize, usize), AppError> {
    let chapters: Vec<Chapter> = sqlx::query_as(
        "SELECT * FROM chapters WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    let mut success = 0usize;
    let mut failed = 0usize;

    for chapter in &chapters {
        // 跳过已有摘要的
        if let Some(s) = &chapter.summary {
            if !s.trim().is_empty() {
                continue;
            }
        }

        match generate_chapter_summary(pool, app_data_dir, &chapter.id, false).await {
            Ok(_) => success += 1,
            Err(e) => {
                tracing::warn!(
                    "Failed to generate summary for chapter {}: {}",
                    chapter.id,
                    e
                );
                failed += 1;
            }
        }
    }

    tracing::info!(
        "Batch summary generation: {} succeeded, {} failed (project {})",
        success,
        failed,
        project_id
    );

    Ok((success, failed))
}

/// 获取章节摘要（直接读 DB）
pub async fn get_summary(pool: &SqlitePool, chapter_id: &str) -> Result<Option<String>, AppError> {
    let chapter = chapter_repo::get(pool, chapter_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Chapter '{}' not found", chapter_id)))?;
    Ok(chapter.summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_system_prompt_contains_constraints() {
        let prompt = build_summary_system_prompt();
        assert!(prompt.contains("摘要"));
        assert!(prompt.contains(&SUMMARY_TARGET_CHARS.to_string()));
        assert!(prompt.contains("直接输出"));
    }

    #[test]
    fn test_summary_user_prompt_with_content() {
        let chapter = Chapter {
            id: "test".into(),
            volume_id: "v".into(),
            project_id: "p".into(),
            title: "测试章节".into(),
            order_index: 0,
            status: "draft".into(),
            word_count: 0,
            summary: None,
            created_at: "2026-01-01 00:00:00".into(),
            updated_at: "2026-01-01 00:00:00".into(),
        };
        let prompt = build_summary_user_prompt(&chapter, "这是正文内容");
        assert!(prompt.contains("测试章节"));
        assert!(prompt.contains("这是正文内容"));
    }

    #[test]
    fn test_summary_user_prompt_empty_content() {
        let chapter = Chapter {
            id: "test".into(),
            volume_id: "v".into(),
            project_id: "p".into(),
            title: "空章节".into(),
            order_index: 0,
            status: "draft".into(),
            word_count: 0,
            summary: None,
            created_at: "2026-01-01 00:00:00".into(),
            updated_at: "2026-01-01 00:00:00".into(),
        };
        let prompt = build_summary_user_prompt(&chapter, "");
        assert!(prompt.contains("空章节"));
        assert!(prompt.contains("正文为空"));
    }
}
