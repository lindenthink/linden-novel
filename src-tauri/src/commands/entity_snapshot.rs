use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::entity_snapshot::{EntityType, ProjectEntity};
use crate::services::entity_snapshot_service;

// --- 请求/响应 ---

#[derive(Debug, Deserialize)]
pub struct GenerateSnapshotsRequest {
    pub chapter_id: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateSnapshotsResponse {
    pub chapter_id: String,
    pub success_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct BatchSnapshotsRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize)]
pub struct BatchSnapshotsResponse {
    pub project_id: String,
    pub success_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetEvolutionRequest {
    pub entity_type: String,
    pub entity_id: String,
}

#[derive(Debug, Serialize)]
pub struct EvolutionResponse {
    pub entity_id: String,
    pub entity_type: String,
    pub name: String,
    pub snapshots: Vec<SnapshotWithChapter>,
}

#[derive(Debug, Serialize)]
pub struct SnapshotWithChapter {
    pub id: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub order_index: i32,
    pub state_json: String,
    pub summary: String,
    pub changes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListChapterSnapshotsRequest {
    pub chapter_id: String,
}

#[derive(Debug, Serialize)]
pub struct ListChapterSnapshotsResponse {
    pub snapshots: Vec<SnapshotItem>,
}

#[derive(Debug, Serialize)]
pub struct SnapshotItem {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub summary: String,
    pub state_json: String,
    pub changes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSnapshotsRequest {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectEntitiesRequest {
    pub project_id: String,
}

#[derive(Debug, Serialize)]
pub struct ListProjectEntitiesResponse {
    pub entities: Vec<ProjectEntity>,
}

// --- 命令 ---

/// 为指定章节生成实体快照
#[tauri::command]
pub async fn generate_chapter_snapshots(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: GenerateSnapshotsRequest,
) -> Result<GenerateSnapshotsResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let (success_count, failed_count) =
        entity_snapshot_service::generate_chapter_snapshots(
            pool.inner(),
            &app_data_dir,
            &request.chapter_id,
        )
        .await?;

    Ok(GenerateSnapshotsResponse {
        chapter_id: request.chapter_id,
        success_count,
        failed_count,
    })
}

/// 批量生成项目内所有章节的实体快照
#[tauri::command]
pub async fn batch_generate_snapshots(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    request: BatchSnapshotsRequest,
) -> Result<BatchSnapshotsResponse, AppError> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Internal(format!("Failed to get app data dir: {}", e))
    })?;

    let (success_count, failed_count) =
        entity_snapshot_service::generate_all_snapshots(
            pool.inner(),
            &app_data_dir,
            &request.project_id,
        )
        .await?;

    Ok(BatchSnapshotsResponse {
        project_id: request.project_id,
        success_count,
        failed_count,
    })
}

/// 获取实体的演变历史
#[tauri::command]
pub async fn get_entity_evolution(
    pool: State<'_, SqlitePool>,
    request: GetEvolutionRequest,
) -> Result<EvolutionResponse, AppError> {
    let entity_type: EntityType = request
        .entity_type
        .parse()
        .map_err(|e: String| AppError::Validation(e))?;

    let evo =
        entity_snapshot_service::get_entity_evolution(pool.inner(), entity_type, &request.entity_id)
            .await?;

    let snapshots = evo
        .snapshots
        .into_iter()
        .map(|swc| SnapshotWithChapter {
            id: swc.snapshot.id,
            chapter_id: swc.snapshot.chapter_id,
            chapter_title: swc.chapter_title,
            order_index: swc.order_index,
            state_json: swc.snapshot.state_json,
            summary: swc.snapshot.summary,
            changes: swc.snapshot.changes,
            created_at: swc.snapshot.created_at,
        })
        .collect();

    Ok(EvolutionResponse {
        entity_id: evo.entity_id,
        entity_type: evo.entity_type,
        name: evo.name,
        snapshots,
    })
}

/// 获取指定章节的全部实体快照
#[tauri::command]
pub async fn list_chapter_snapshots(
    pool: State<'_, SqlitePool>,
    request: ListChapterSnapshotsRequest,
) -> Result<ListChapterSnapshotsResponse, AppError> {
    let snapshots = crate::db::repo::entity_snapshot_repo::list_by_chapter(
        pool.inner(),
        &request.chapter_id,
    )
    .await
    .map_err(AppError::from)?;

    let items: Vec<SnapshotItem> = snapshots
        .into_iter()
        .map(|s| SnapshotItem {
            id: s.id,
            entity_type: s.entity_type,
            entity_id: s.entity_id,
            summary: s.summary,
            state_json: s.state_json,
            changes: s.changes,
        })
        .collect();

    Ok(ListChapterSnapshotsResponse { snapshots: items })
}

/// 删除项目的全部实体快照
#[tauri::command]
pub async fn delete_project_snapshots(
    pool: State<'_, SqlitePool>,
    request: DeleteSnapshotsRequest,
) -> Result<(), AppError> {
    crate::db::repo::entity_snapshot_repo::delete_by_project(pool.inner(), &request.project_id)
        .await
        .map_err(AppError::from)?;
    Ok(())
}

/// 列出项目内所有有快照的实体（去重）
#[tauri::command]
pub async fn list_project_entities(
    pool: State<'_, SqlitePool>,
    request: ListProjectEntitiesRequest,
) -> Result<ListProjectEntitiesResponse, AppError> {
    let entities = crate::db::repo::entity_snapshot_repo::list_project_entities(
        pool.inner(),
        &request.project_id,
    )
    .await
    .map_err(AppError::from)?;

    Ok(ListProjectEntitiesResponse { entities })
}
