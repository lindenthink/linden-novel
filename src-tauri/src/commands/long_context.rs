use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::db::repo::chapter_repo;
use crate::error::AppError;
use crate::models::async_task::NewTaskRequest;
use crate::services::task_manager::TaskManagerState;
use crate::services::{embedding_service, summary_service};

// --- 请求/响应类型 ---

#[derive(Debug, Deserialize)]
pub struct GenerateSummaryRequest {
    pub chapter_id: String,
    /// 是否强制重新生成（即使已有摘要）
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize)]
pub struct GenerateSummaryResponse {
    pub chapter_id: String,
    pub summary: String,
    pub char_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct BatchSummaryRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize)]
pub struct BatchSummaryResponse {
    pub project_id: String,
    pub success_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct SyncEmbeddingsRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize)]
pub struct SyncEmbeddingsResponse {
    pub project_id: String,
    pub embedded_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct RagSearchRequest {
    pub project_id: String,
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_min_score")]
    pub min_score: f32,
    #[serde(default)]
    pub exclude_chapter_ids: Vec<String>,
}

fn default_top_k() -> usize {
    3
}
fn default_min_score() -> f32 {
    0.3
}

#[derive(Debug, Serialize)]
pub struct RagSearchResponse {
    pub results: Vec<RagSearchItem>,
}

#[derive(Debug, Serialize)]
pub struct RagSearchItem {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Deserialize)]
pub struct DeleteEmbeddingsRequest {
    pub project_id: String,
}

// --- 命令实现 ---

/// 为指定章节生成摘要（含嵌入触发）
#[tauri::command]
pub async fn generate_chapter_summary(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    task_manager_state: State<'_, TaskManagerState>,
    request: GenerateSummaryRequest,
) -> Result<GenerateSummaryResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    // trigger_embedding=false：由本 command 统一提交 embed_element + embed_chapter 任务到 TaskManager，
    // 让任务中心展示进度（单章节场景）
    let summary = summary_service::generate_chapter_summary(
        pool.inner(),
        &app_data_dir,
        &request.chapter_id,
        request.force,
        false,
    )
    .await?;

    // 提交摘要级 + 切片级嵌入任务到任务中心
    submit_chapter_embedding_tasks(
        &app,
        &task_manager_state,
        pool.inner(),
        &request.chapter_id,
        &summary,
    )
    .await?;

    let char_count = summary.chars().count();
    Ok(GenerateSummaryResponse {
        chapter_id: request.chapter_id,
        summary,
        char_count,
    })
}

/// 提交章节嵌入任务（摘要级 embed_element + 切片级 embed_chapter）到 TaskManager
async fn submit_chapter_embedding_tasks(
    app: &AppHandle,
    task_manager_state: &State<'_, TaskManagerState>,
    pool: &SqlitePool,
    chapter_id: &str,
    summary: &str,
) -> Result<(), AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;
    let app_data_dir_str = app_data_dir.to_string_lossy().to_string();

    let chapter = chapter_repo::get(pool, chapter_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Chapter '{}' not found", chapter_id)))?;
    let project_id = chapter.project_id.clone();

    let task_manager = task_manager_state
        .read()
        .await
        .clone()
        .ok_or_else(|| AppError::Internal("TaskManager not initialized".into()))?;

    // 1. 摘要级嵌入任务（target_type=chapter）
    let mut hasher = Sha256::new();
    hasher.update(summary.as_bytes());
    let summary_hash = format!("{:x}", hasher.finalize());

    task_manager
        .submit(NewTaskRequest {
            task_type: "embed_element".to_string(),
            project_id: project_id.clone(),
            target_type: Some("chapter".to_string()),
            target_id: Some(chapter_id.to_string()),
            content_hash: Some(summary_hash),
            payload_json: Some(serde_json::json!({
                "text": summary,
                "app_data_dir": app_data_dir_str,
            })),
        })
        .await?;

    // 2. 切片级嵌入任务（重建该章节切片向量）
    // content_hash 基于 chapter_id：同章节 pending/running 时幂等跳过；
    // 已完成的任务可重新提交（用户重新生成摘要时章节内容可能已变化）
    let mut hasher = Sha256::new();
    hasher.update(chapter_id.as_bytes());
    let chapter_hash = format!("{:x}", hasher.finalize());

    task_manager
        .submit(NewTaskRequest {
            task_type: "embed_chapter".to_string(),
            project_id,
            target_type: Some("chapter".to_string()),
            target_id: Some(chapter_id.to_string()),
            content_hash: Some(chapter_hash),
            payload_json: Some(serde_json::json!({
                "app_data_dir": app_data_dir_str,
            })),
        })
        .await?;

    Ok(())
}

/// 批量为项目内所有无摘要章节生成摘要
#[tauri::command]
pub async fn batch_generate_summaries(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: BatchSummaryRequest,
) -> Result<BatchSummaryResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let (success_count, failed_count) =
        summary_service::generate_all_summaries_silent(pool.inner(), &app_data_dir, &request.project_id).await?;

    Ok(BatchSummaryResponse {
        project_id: request.project_id,
        success_count,
        failed_count,
    })
}

/// 为项目所有元素（章节摘要 + 角色/故事线/世界观描述）同步嵌入
#[tauri::command]
pub async fn sync_project_embeddings(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: SyncEmbeddingsRequest,
) -> Result<SyncEmbeddingsResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let embedded_count =
        embedding_service::sync_project_embeddings_silent(pool.inner(), &app_data_dir, &request.project_id).await?;

    Ok(SyncEmbeddingsResponse {
        project_id: request.project_id,
        embedded_count,
    })
}

/// 执行 RAG 检索（语义搜索）
#[tauri::command]
pub async fn rag_search(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: RagSearchRequest,
) -> Result<RagSearchResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let config = crate::ai::rag::RagConfig {
        top_k: request.top_k,
        min_score: request.min_score,
        exclude_chapter_ids: request.exclude_chapter_ids,
        exclude_element_ids: Vec::new(),
    };

    let ctx = crate::ai::rag::retrieve(
        pool.inner(),
        &app_data_dir,
        &request.project_id,
        &request.query,
        &config,
    )
    .await?;

    let mut results: Vec<RagSearchItem> = Vec::new();

    for s in ctx.related_chapter_summaries {
        results.push(RagSearchItem {
            source_type: "chapter".into(),
            source_id: s.chapter_id,
            title: s.title,
            content: s.summary,
            score: s.score,
        });
    }
    for c in ctx.related_characters {
        results.push(RagSearchItem {
            source_type: "character".into(),
            source_id: c.id,
            title: c.name,
            content: c.description.unwrap_or_default(),
            score: c.score,
        });
    }
    for s in ctx.related_storylines {
        results.push(RagSearchItem {
            source_type: "storyline".into(),
            source_id: s.id,
            title: s.name,
            content: s.description.unwrap_or_default(),
            score: s.score,
        });
    }
    for w in ctx.related_worldviews {
        results.push(RagSearchItem {
            source_type: "worldview".into(),
            source_id: w.id,
            title: w.name,
            content: w.description.unwrap_or_default(),
            score: w.score,
        });
    }

    // 按分数降序排列
    results.sort_by(|a, b| {
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(RagSearchResponse { results })
}

/// 删除项目的全部嵌入
#[tauri::command]
pub async fn delete_project_embeddings(
    pool: State<'_, SqlitePool>,
    request: DeleteEmbeddingsRequest,
) -> Result<(), AppError> {
    embedding_service::remove_by_project(pool.inner(), &request.project_id).await?;
    Ok(())
}

/// 获取章节摘要
#[tauri::command]
pub async fn get_chapter_summary(
    pool: State<'_, SqlitePool>,
    chapter_id: String,
) -> Result<Option<String>, AppError> {
    summary_service::get_summary(pool.inner(), &chapter_id).await
}
