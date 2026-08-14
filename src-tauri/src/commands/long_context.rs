use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
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
    pub exclude_chapter_id: Option<String>,
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
    request: GenerateSummaryRequest,
) -> Result<GenerateSummaryResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let summary =
        summary_service::generate_chapter_summary(pool.inner(), &app_data_dir, &request.chapter_id, request.force).await?;

    let char_count = summary.chars().count();
    Ok(GenerateSummaryResponse {
        chapter_id: request.chapter_id,
        summary,
        char_count,
    })
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
        exclude_chapter_id: request.exclude_chapter_id,
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
