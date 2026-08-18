use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::path::Path;

use crate::ai::provider::{CompletionRequest, Message};
use crate::ai::provider_factory;
use crate::db::repo::entity_snapshot_repo;
use crate::error::AppError;
use crate::models::character::Character;
use crate::models::chapter::Chapter;
use crate::models::entity_snapshot::{EntitySnapshot, EntityType, UpsertEntitySnapshot};
use crate::models::storyline::Storyline;
use crate::services::{ai_api_key_service, ai_provider_service};

/// 构建角色状态提取的系统提示
fn build_character_system_prompt() -> String {
    "你是一位专业的小说分析专家。你的任务是从给定的小说章节中提取指定角色的当前状态。\n\n\
     要求：\n\
     1. 仅提取该角色在本章中出现的状态信息\n\
     2. 按以下 JSON 格式输出，不要添加额外解释：\n\
     ```json\n\
     {\n\
       \"status\": \"alive\" | \"dead\" | \"missing\" | \"unknown\",\n\
       \"location\": \"当前所在地点（如有）\",\n\
       \"role_change\": \"角色身份/立场的重大变化（如有）\",\n\
       \"relationships\": { \"对方角色名\": \"关系变化描述\" },\n\
       \"key_events\": [\"本章发生的关键事件\"],\n\
       \"emotional_state\": \"当前情感状态（如有）\"\n\
     }\n\
     ```\n\
     3. 如某项信息无法从文本中提取，留空字符串或省略\n\
     4. 直接输出 JSON，不要包含任何前缀或标记".to_string()
}

/// 构建故事线状态提取的系统提示
fn build_storyline_system_prompt() -> String {
    "你是一位专业的小说分析专家。你的任务是从给定的小说章节中提取指定情节线索的当前进展。\n\n\
     要求：\n\
     1. 仅提取该情节线索在本章中的进展信息\n\
     2. 按以下 JSON 格式输出，不要添加额外解释：\n\
     ```json\n\
     {\n\
       \"progress\": \"resolved\" | \"advancing\" | \"stalled\" | \"introduced\" | \"unknown\",\n\
       \"key_developments\": [\"本章中的关键进展\"],\n\
       \"involved_characters\": [\"涉及的角色\"],\n\
       \"foreshadowing\": \"伏笔/铺垫信息（如有）\",\n\
       \"tension_level\": \"low\" | \"medium\" | \"high\" | \"climax\"\n\
     }\n\
     ```\n\
     3. 如某项信息无法从文本中提取，留空字符串或省略\n\
     4. 直接输出 JSON，不要包含任何前缀或标记".to_string()
}

/// 构建用户消息
fn build_user_prompt(
    entity_name: &str,
    entity_desc: Option<&str>,
    chapter_title: &str,
    chapter_content: &str,
) -> String {
    let mut prompt = format!("目标实体：{}\n", entity_name);
    if let Some(desc) = entity_desc {
        if !desc.trim().is_empty() {
            prompt.push_str(&format!("实体描述：{}\n", desc));
        }
    }
    prompt.push_str(&format!("章节标题：{}\n", chapter_title));
    prompt.push_str("章节正文：\n");
    prompt.push_str(chapter_content);
    prompt
}

/// 清理 AI 返回的 JSON（去除可能的 markdown 包裹）
fn extract_json(raw: &str) -> String {
    let trimmed = raw.trim();
    // Remove markdown code block wrapping
    if trimmed.starts_with("```") {
        let start = trimmed.find('{').unwrap_or(0);
        let end = trimmed.rfind('}').map(|p| p + 1).unwrap_or(trimmed.len());
        if start < end {
            return trimmed[start..end].to_string();
        }
    }
    trimmed.to_string()
}

/// 验证并规范化 JSON
fn normalize_state_json(raw: &str, entity_type: EntityType) -> Value {
    let parsed: Value = serde_json::from_str(&extract_json(raw))
        .unwrap_or_else(|_| {
            // 如果解析失败，将原始文本放入 summary 字段
            json!({ "raw_text": raw.trim() })
        });

    // 确保 status 字段存在
    let mut obj = parsed.as_object().cloned().unwrap_or_default();

    match entity_type {
        EntityType::Character => {
            if !obj.contains_key("status") {
                obj.insert("status".into(), json!("unknown"));
            }
        }
        EntityType::Storyline => {
            if !obj.contains_key("progress") {
                obj.insert("progress".into(), json!("unknown"));
            }
        }
    }

    Value::Object(obj)
}

/// 获取默认 AI provider
async fn get_default_provider(
    pool: &SqlitePool,
    app_data_dir: &Path,
) -> Result<Box<dyn crate::ai::provider::AiProvider>, AppError> {
    let provider = ai_provider_service::get_default(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("No default AI provider configured".to_string()))?;

    let default_key = ai_api_key_service::get_default_for_provider(pool, &provider.id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("No API key configured for default provider".to_string())
        })?;

    let api_key = ai_api_key_service::get_decrypted(pool, app_data_dir, &default_key.id).await?;
    provider_factory::create_provider(&provider, &api_key)
}

/// 获取章节正文
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

/// 为指定章节中的所有关联实体生成快照
///
/// # 流程
/// 1. 读取章节及正文
/// 2. 查询章节关联的角色和故事线
/// 3. 逐一调用 AI 提取状态
/// 4. 与上一快照对比变化（如果有）
/// 5. 存入 entity_snapshots
///
/// # 返回
/// (生成的快照数, 失败数)
pub async fn generate_chapter_snapshots(
    pool: &SqlitePool,
    app_data_dir: &Path,
    chapter_id: &str,
) -> Result<(usize, usize), AppError> {
    let chapter = crate::db::repo::chapter_repo::get(pool, chapter_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Chapter not found".to_string()))?;

    let content = get_chapter_text(pool, chapter_id).await?;
    if content.trim().is_empty() {
        return Ok((0, 0));
    }

    // 获取关联元素
    let elements = crate::db::repo::chapter_element_repo::list_by_chapter(pool, chapter_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to list elements: {}", e)))?;

    if elements.is_empty() {
        return Ok((0, 0));
    }

    let provider = get_default_provider(pool, app_data_dir).await?;
    let model = provider.models().first().cloned().unwrap_or_else(|| "gpt-3.5-turbo".to_string());

    let mut success = 0usize;
    let mut failed = 0usize;

    for element in &elements {
        let entity_type_str = element.element_type.as_str();
        let entity_type: EntityType = entity_type_str
            .parse()
            .map_err(|e: String| AppError::Validation(e))?;

        let (entity_name, entity_desc) = match entity_type {
            EntityType::Character => {
                let c: Option<Character> = crate::db::repo::character_repo::get(
                    pool,
                    &element.element_id,
                )
                .await?;
                match c {
                    Some(c) => (c.name, c.description),
                    None => continue,
                }
            }
            EntityType::Storyline => {
                let s: Option<Storyline> = crate::db::repo::storyline_repo::get(
                    pool,
                    &element.element_id,
                )
                .await?;
                match s {
                    Some(s) => (s.name, s.description),
                    None => continue,
                }
            }
        };

        // 构建 prompt
        let system_prompt = match entity_type {
            EntityType::Character => build_character_system_prompt(),
            EntityType::Storyline => build_storyline_system_prompt(),
        };

        let user_prompt = build_user_prompt(
            &entity_name,
            entity_desc.as_deref(),
            &chapter.title,
            &content,
        );

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        let request = CompletionRequest {
            model: model.clone(),
            messages,
            temperature: Some(0.2),
            // 不设置 max_tokens：推理模型（如 DeepSeek V4）默认开启 thinking，
            // 硬 token 限制会被 thinking 消耗后导致 content 为空。
            max_tokens: None,
            stream: false,
        };

        match provider.complete(request).await {
            Ok(response) => {
                let normalized = normalize_state_json(&response.content, entity_type);
                let state_json = serde_json::to_string(&normalized).unwrap_or_default();

                // 生成自然语言摘要
                let summary = generate_summary_from_state(&normalized, &entity_name, entity_type);

                // 计算变化（与上一快照对比）
                let changes = compute_changes(
                    pool,
                    entity_type,
                    &element.element_id,
                    &chapter.project_id,
                    &state_json,
                )
                .await;

                let input = UpsertEntitySnapshot {
                    project_id: chapter.project_id.clone(),
                    entity_type,
                    entity_id: element.element_id.clone(),
                    chapter_id: chapter_id.to_string(),
                    state_json,
                    summary,
                    changes,
                };

                match entity_snapshot_repo::upsert(pool, &input).await {
                    Ok(_) => success += 1,
                    Err(e) => {
                        tracing::warn!("Failed to save snapshot: {}", e);
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to extract state for {}:{}: {}",
                    entity_type_str,
                    element.element_id,
                    e
                );
                failed += 1;
            }
        }
    }

    tracing::info!(
        "Entity snapshots for chapter {}: {} succeeded, {} failed",
        chapter_id,
        success,
        failed
    );

    Ok((success, failed))
}

/// 从状态 JSON 生成自然语言摘要
fn generate_summary_from_state(state: &Value, entity_name: &str, entity_type: EntityType) -> String {
    match entity_type {
        EntityType::Character => {
            let status = state.get("status").and_then(|v| v.as_str()).unwrap_or("未知");
            let location = state.get("location").and_then(|v| v.as_str()).unwrap_or("");
            let key_events = state
                .get("key_events")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("、")
                })
                .unwrap_or_default();

            let mut summary = format!("{}（状态：{}）", entity_name, status);
            if !location.is_empty() {
                summary.push_str(&format!("，位于：{}", location));
            }
            if !key_events.is_empty() {
                summary.push_str(&format!("。关键事件：{}", key_events));
            }
            summary
        }
        EntityType::Storyline => {
            let progress = state.get("progress").and_then(|v| v.as_str()).unwrap_or("未知");
            let key_devs = state
                .get("key_developments")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("、")
                })
                .unwrap_or_default();
            let tension = state
                .get("tension_level")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let mut summary = format!("{}（进展：{}）", entity_name, progress);
            if !key_devs.is_empty() {
                summary.push_str(&format!("。关键进展：{}", key_devs));
            }
            if !tension.is_empty() {
                summary.push_str(&format!("。紧张度：{}", tension));
            }
            summary
        }
    }
}

/// 计算与上一快照的差异
async fn compute_changes(
    pool: &SqlitePool,
    entity_type: EntityType,
    entity_id: &str,
    _project_id: &str,
    current_state_json: &str,
) -> Option<String> {
    // 找最近的一个非本章快照
    let prev: Option<EntitySnapshot> = sqlx::query_as::<_, EntitySnapshot>(
        "SELECT * FROM entity_snapshots
         WHERE entity_type = ? AND entity_id = ?
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(entity_type.as_str())
    .bind(entity_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let prev = prev?;

    let current: Value = serde_json::from_str(current_state_json).ok()?;
    let previous: Value = serde_json::from_str(&prev.state_json).ok()?;

    let mut changes = Vec::new();

    // 对比关键字段
    for field in &["status", "progress", "location"] {
        let curr_val = current.get(field).and_then(|v| v.as_str());
        let prev_val = previous.get(field).and_then(|v| v.as_str());
        if curr_val != prev_val && curr_val.is_some() {
            changes.push(format!(
                "{}: {} → {}",
                field,
                prev_val.unwrap_or("无"),
                curr_val.unwrap()
            ));
        }
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes.join("; "))
    }
}

/// 批量快照生成统计结果
#[derive(Debug, Default, serde::Serialize)]
pub struct BatchSnapshotResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_chapters: usize,
    pub skipped_count: usize,
}

/// 批量为项目内所有章节生成实体快照
///
/// 已有快照的章节会被跳过（与 `generate_all_summaries` 跳过已有摘要的章节保持一致），
/// 单章生成时由前端弹窗确认覆盖。
///
/// # 参数
/// - `progress`: 进度回调 `(current, total)`，current 为已处理的章节数，total 为待生成章节数
pub async fn generate_all_snapshots<F>(
    pool: &SqlitePool,
    app_data_dir: &Path,
    project_id: &str,
    mut progress: F,
) -> Result<BatchSnapshotResult, AppError>
where
    F: FnMut(usize, usize) + Send,
{
    let chapters: Vec<Chapter> = sqlx::query_as(
        "SELECT * FROM chapters WHERE project_id = ? ORDER BY order_index",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    // 筛出需要生成快照的章节：跳过已存在快照的章节
    let mut need_snapshots: Vec<&Chapter> = Vec::new();
    for c in &chapters {
        if entity_snapshot_repo::exists_by_chapter_id(pool, &c.id).await? {
            continue;
        }
        need_snapshots.push(c);
    }

    let total = need_snapshots.len();
    let skipped = chapters.len() - total;
    let mut result = BatchSnapshotResult {
        total_chapters: chapters.len(),
        skipped_count: skipped,
        ..Default::default()
    };

    tracing::info!(
        "Batch snapshot generation: {} chapters total, {} to generate, {} skipped (project {})",
        chapters.len(),
        total,
        skipped,
        project_id
    );

    progress(0, total);

    for (i, chapter) in need_snapshots.iter().enumerate() {
        match generate_chapter_snapshots(pool, app_data_dir, &chapter.id).await {
            Ok((s, f)) => {
                result.success_count += s;
                result.failed_count += f;
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to generate snapshots for chapter {}: {}",
                    chapter.id,
                    e
                );
                result.failed_count += 1;
            }
        }
        progress(i + 1, total);
    }

    Ok(result)
}

/// 向后兼容的简化封装（无进度回调场景）
pub async fn generate_all_snapshots_silent(
    pool: &SqlitePool,
    app_data_dir: &Path,
    project_id: &str,
) -> Result<(usize, usize), AppError> {
    let r = generate_all_snapshots(pool, app_data_dir, project_id, |_, _| {}).await?;
    Ok((r.success_count, r.failed_count))
}

/// 获取实体的完整演变历史
pub async fn get_entity_evolution(
    pool: &SqlitePool,
    entity_type: EntityType,
    entity_id: &str,
) -> Result<crate::models::entity_snapshot::EntityEvolution, AppError> {
    entity_snapshot_repo::get_evolution(pool, entity_type, entity_id)
        .await
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_clean() {
        let raw = r#"{"status":"alive","location":"北京"}"#;
        let result = extract_json(raw);
        assert!(result.contains("status"));
    }

    #[test]
    fn test_extract_json_wrapped() {
        let raw = "```json\n{\"status\":\"alive\"}\n```";
        let result = extract_json(raw);
        assert!(result.contains("status"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn test_normalize_state_missing_status() {
        let raw = r#"{"location":"北京"}"#;
        let normalized = normalize_state_json(raw, EntityType::Character);
        assert!(normalized.get("status").is_some());
    }

    #[test]
    fn test_normalize_state_storyline_progress() {
        let raw = r#"{"key_developments": ["事件A"]}"#;
        let normalized = normalize_state_json(raw, EntityType::Storyline);
        assert!(normalized.get("progress").is_some());
    }

    #[test]
    fn test_generate_summary_character() {
        let state = json!({
            "status": "alive",
            "location": "北京",
            "key_events": ["遇到主角", "揭示身份"]
        });
        let summary = generate_summary_from_state(&state, "张三", EntityType::Character);
        assert!(summary.contains("张三"));
        assert!(summary.contains("alive"));
        assert!(summary.contains("北京"));
        assert!(summary.contains("遇到主角"));
    }

    #[test]
    fn test_generate_summary_storyline() {
        let state = json!({
            "progress": "advancing",
            "key_developments": ["线索发现"],
            "tension_level": "high"
        });
        let summary = generate_summary_from_state(&state, "复仇线", EntityType::Storyline);
        assert!(summary.contains("复仇线"));
        assert!(summary.contains("advancing"));
        assert!(summary.contains("high"));
    }
}
