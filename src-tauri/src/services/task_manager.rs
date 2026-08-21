use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::db::pool;
use crate::db::repo::async_task_repo;
use crate::error::AppError;
use crate::models::async_task::{AsyncTask, NewTaskRequest, TaskStatus};
use crate::services::embedding_service;

struct TaskManagerInner {
    pool: SqlitePool,
    app_handle: AppHandle,
    sender: mpsc::Sender<String>, // 发送 task_id
}

#[derive(Clone)]
pub struct TaskManager {
    inner: Arc<TaskManagerInner>,
}

impl TaskManager {
    pub fn new_with_sender(pool: SqlitePool, app_handle: AppHandle, sender: mpsc::Sender<String>) -> Self {
        Self {
            inner: Arc::new(TaskManagerInner { pool, app_handle, sender }),
        }
    }

    /// 提交异步任务（支持幂等）
    pub async fn submit(&self, req: NewTaskRequest) -> Result<String, AppError> {
        // 1. 幂等检查：如果有相同 hash 的 pending/running 任务，直接返回该任务 ID
        if let Some(content_hash) = &req.content_hash {
            if let Some(existing) = async_task_repo::find_existing_pending_or_running(
                &self.inner.pool,
                &req.task_type,
                req.target_id.as_deref(),
                Some(content_hash.as_str()),
            )
            .await?
            {
                tracing::info!(
                    "Task already exists (idempotent): id={}, type={}",
                    existing.id,
                    req.task_type
                );
                return Ok(existing.id);
            }
        }

        // 2. 创建新任务记录
        let now = pool::now();
        let task_id = Uuid::new_v4().to_string();
        let task = AsyncTask {
            id: task_id.clone(),
            task_type: req.task_type.clone(),
            project_id: req.project_id,
            target_type: req.target_type,
            target_id: req.target_id,
            content_hash: req.content_hash,
            payload_json: req
                .payload_json
                .map(|v| serde_json::to_string(&v).unwrap_or_default()),
            status: TaskStatus::Pending.to_string(),
            progress_current: 0,
            progress_total: 0,
            result_json: None,
            error_message: None,
            created_at: now.clone(),
            started_at: None,
            completed_at: None,
        };

        async_task_repo::create(&self.inner.pool, &task).await?;
        tracing::info!("Task created: id={}, type={}", task_id, task.task_type);

        // 3. 发送新任务创建事件
        let _ = self.inner.app_handle.emit(
            "task-created",
            serde_json::json!({
                "task_id": task_id,
                "task_type": task.task_type,
                "project_id": task.project_id,
                "status": "pending",
                "created_at": task.created_at,
            }),
        );

        // 4. 发送到 worker 队列
        self.inner.sender
            .send(task_id.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send task to queue: {}", e)))?;

        Ok(task_id)
    }

    /// 运行 worker 循环
    pub async fn run_worker(pool: SqlitePool, app_handle: AppHandle, mut receiver: mpsc::Receiver<String>) {
        tracing::info!("Task worker started");

        while let Some(task_id) = receiver.recv().await {
            if let Err(e) = process_task(&pool, &app_handle, &task_id).await {
                tracing::error!("Task {} failed: {}", task_id, e);
                // 更新任务状态为失败
                if let Err(update_err) = async_task_repo::update_result(
                    &pool,
                    &task_id,
                    TaskStatus::Failed,
                    None,
                    Some(&format!("{:?}", e)),
                )
                .await
                {
                    tracing::error!("Failed to update task status: {}", update_err);
                }
                // 发送失败事件
                let _ = app_handle.emit(
                    "task-completed",
                    serde_json::json!({
                        "task_id": task_id,
                        "status": "failed",
                        "error": format!("{:?}", e)
                    }),
                );
            }
        }

        tracing::info!("Task worker stopped");
    }
}

/// 处理单个任务的函数
async fn process_task(
    pool: &SqlitePool,
    app_handle: &AppHandle,
    task_id: &str,
) -> Result<(), AppError> {
    // 1. 获取任务
    let task = async_task_repo::get_by_id(pool, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task '{}' not found", task_id)))?;

    // 2. 标记为 running
    async_task_repo::mark_running(pool, task_id, Some(1)).await?; // total steps = 1
    let _ = app_handle.emit(
        "task-progress",
        serde_json::json!({
            "task_id": task_id,
            "status": "running",
            "progress_current": 0,
            "progress_total": 1
        }),
    );

    // 3. 根据 task_type 分发处理
    let result = match task.task_type.as_str() {
        "embed_element" => process_embed_element(pool, &task, app_handle).await,
        "embed_chapter" => process_embed_chapter(pool, &task, app_handle).await,
        "sync_embeddings" => process_sync_embeddings(pool, &task, app_handle).await,
        "generate_summary" => process_generate_summary(pool, &task, app_handle).await,
        "generate_snapshots" => process_generate_snapshots(pool, &task, app_handle).await,
        _ => Err(AppError::Validation(format!(
            "Unknown task type: {}",
            task.task_type
        ))),
    };

    // 4. 更新最终状态
    match result {
        Ok(()) => {
            async_task_repo::update_progress(pool, task_id, 1).await?;
            async_task_repo::update_result(
                pool,
                task_id,
                TaskStatus::Completed,
                Some(r#"{"success": true}"#),
                None,
            )
            .await?;
            let _ = app_handle.emit(
                "task-completed",
                serde_json::json!({
                    "task_id": task_id,
                    "status": "completed",
                    "result": {"success": true}
                }),
            );
            tracing::info!("Task {} completed successfully", task_id);
        }
        Err(e) => {
            async_task_repo::update_result(
                pool,
                task_id,
                TaskStatus::Failed,
                None,
                Some(&format!("{:?}", e)),
            )
            .await?;
            let _ = app_handle.emit(
                "task-completed",
                serde_json::json!({
                    "task_id": task_id,
                    "status": "failed",
                    "error": format!("{:?}", e)
                }),
            );
            tracing::error!("Task {} failed: {}", task_id, e);
        }
    }

    Ok(())
}

/// 处理 embed_element 类型的任务
async fn process_embed_element(
    pool: &SqlitePool,
    task: &AsyncTask,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let task_id = &task.id;
    let target_id = task.target_id.as_deref()
        .ok_or_else(|| AppError::Validation("No target_id for embed_element task".into()))?;
    
    let payload_str = task.payload_json.as_deref()
        .ok_or_else(|| AppError::Validation("No payload for embed_element task".into()))?;
    
    let payload: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse payload: {}", e)))?;

    // 从 payload 提取参数
    let text = payload.get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("No 'text' in payload".into()))?;
    
    let source_type = task.target_type.as_deref()
        .ok_or_else(|| AppError::Validation("No target_type for embed_element task".into()))?;
    
    let app_data_dir_str = payload.get("app_data_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("No 'app_data_dir' in payload".into()))?;
    let app_data_dir = PathBuf::from(app_data_dir_str);

    tracing::info!(
        "Processing embed_element task: id={}, target_id={}, type={}",
        task_id, target_id, source_type
    );

    // 检查 hash 是否匹配（如果是重复提交但状态变了，幂等逻辑已处理，但这里再保险一下）
    if let Some(expected_hash) = &task.content_hash {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let actual_hash = format!("{:x}", hasher.finalize());
        
        if actual_hash != *expected_hash {
            tracing::info!(
                "Content hash changed for task {}, updating hash and re-embedding",
                task_id
            );
            // 更新任务的 hash
            sqlx::query("UPDATE async_tasks SET content_hash = ? WHERE id = ?")
                .bind(&actual_hash)
                .bind(task_id)
                .execute(pool)
                .await
                .map_err(AppError::from)?;
        }
    }

    // 更新进度：0 -> 1 (完成)
    async_task_repo::update_progress(pool, task_id, 0).await?;
    let _ = app_handle.emit(
        "task-progress",
        serde_json::json!({
            "task_id": task_id,
            "progress_current": 0,
            "progress_total": 1
        }),
    );

    // 调用实际的嵌入服务
    let was_embedded = embedding_service::generate_and_store(
        pool,
        &app_data_dir,
        &task.project_id,
        source_type,
        target_id,
        text,
        "", // default model
    ).await?;

    // 更新进度：1 -> 1 (完成)
    async_task_repo::update_progress(pool, task_id, 1).await?;
    let _ = app_handle.emit(
        "task-progress",
        serde_json::json!({
            "task_id": task_id,
            "progress_current": 1,
            "progress_total": 1
        }),
    );

    if was_embedded {
        tracing::info!("Successfully embedded element for task {}", task_id);
    } else {
        tracing::info!("Embedding skipped (hash unchanged or empty) for task {}", task_id);
    }

    Ok(())
}

/// 处理 embed_chapter 类型的任务：章节切片级嵌入（重建该章节所有切片向量）
async fn process_embed_chapter(
    pool: &SqlitePool,
    task: &AsyncTask,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let task_id = &task.id;
    let chapter_id = task.target_id.as_deref()
        .ok_or_else(|| AppError::Validation("No target_id for embed_chapter task".into()))?;

    let payload_str = task.payload_json.as_deref()
        .ok_or_else(|| AppError::Validation("No payload for embed_chapter task".into()))?;
    let payload: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse payload: {}", e)))?;

    let app_data_dir_str = payload.get("app_data_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("No 'app_data_dir' in payload".into()))?;
    let app_data_dir = PathBuf::from(app_data_dir_str);

    tracing::info!(
        "Processing embed_chapter task: id={}, chapter_id={}",
        task_id, chapter_id
    );

    // 读取章节正文
    let content = crate::services::summary_service::get_chapter_text(pool, chapter_id).await?;
    if content.trim().is_empty() {
        tracing::info!("Chapter {} content is empty, skipping chunk embedding", chapter_id);
        return Ok(());
    }

    // 更新进度：0/1
    async_task_repo::update_progress(pool, task_id, 0).await?;
    let _ = app_handle.emit(
        "task-progress",
        serde_json::json!({
            "task_id": task_id,
            "progress_current": 0,
            "progress_total": 1
        }),
    );

    // 执行切片嵌入
    crate::services::chunk_embedding_service::embed_chapter(
        pool,
        &app_data_dir,
        chapter_id,
        &task.project_id,
        &content,
        &crate::ai::chunker::ChunkConfig::default(),
    )
    .await?;

    // 更新进度：1/1
    async_task_repo::update_progress(pool, task_id, 1).await?;
    let _ = app_handle.emit(
        "task-progress",
        serde_json::json!({
            "task_id": task_id,
            "progress_current": 1,
            "progress_total": 1
        }),
    );

    tracing::info!("Successfully embedded chapter chunks for task {}", task_id);
    Ok(())
}

/// 处理 sync_embeddings 类型的任务：全量项目嵌入同步（5 步进度）
async fn process_sync_embeddings(
    pool: &SqlitePool,
    task: &AsyncTask,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let task_id = &task.id;

    let payload_str = task.payload_json.as_deref()
        .ok_or_else(|| AppError::Validation("No payload for sync_embeddings task".into()))?;
    let payload: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse payload: {}", e)))?;

    let app_data_dir_str = payload.get("app_data_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("No 'app_data_dir' in payload".into()))?;
    let app_data_dir = PathBuf::from(app_data_dir_str);
    let project_id = &task.project_id;

    tracing::info!(
        "Processing sync_embeddings task: id={}, project_id={}",
        task_id, project_id
    );

    // 声明进度回调闭包
    let task_id_for_progress = task_id.clone();
    let app_handle_for_progress = app_handle.clone();
    let pool_for_progress = pool.clone();
    let progress_callback = move |current: usize, total: usize| {
        // 在同步上下文中我们不直接 await，而是通过相对轻量的 update_progress 先写入 DB
        // 注意：这里调用的是 sync API，但 sqlx::query::execute 是纯 SQL 执行不 await，
        // 我们在 Worker 线程内部，安全做法是用 spawn_blocking 或直接调用 DB 更新。
        // 由于 execute() 本身不阻塞 await（同步执行 SQL），这里直接 spawn 一个 tokio task。
        let task_id = task_id_for_progress.clone();
        let ah = app_handle_for_progress.clone();
        let pool = pool_for_progress.clone();
        tauri::async_runtime::spawn(async move {
            let _ = async_task_repo::update_progress(&pool, &task_id, current as i64).await;
            let _ = async_task_repo::update_status(
                &pool,
                &task_id,
                TaskStatus::Running,
                Some(current as i64),
                Some(total as i64),
                None,
                None,
            )
            .await;
            let _ = ah.emit(
                "task-progress",
                serde_json::json!({
                    "task_id": task_id,
                    "status": "running",
                    "progress_current": current,
                    "progress_total": total
                }),
            );
        });
    };

    // 执行核心同步逻辑
    let result = embedding_service::sync_project_embeddings(
        pool,
        &app_data_dir,
        project_id,
        progress_callback,
    )
    .await?;

    tracing::info!(
        "sync_embeddings task {} completed: summary={}, character={}, storyline={}, worldview={}, chunks={}, total={}",
        task_id,
        result.chapter_summary_embedded,
        result.character_embedded,
        result.storyline_embedded,
        result.worldview_embedded,
        result.chunk_embedded,
        result.total_embedded,
    );

    Ok(())
}

/// 处理 generate_summary 类型的任务：批量生成项目章节摘要（逐章进度）
async fn process_generate_summary(
    pool: &SqlitePool,
    task: &AsyncTask,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let task_id = &task.id;

    let payload_str = task.payload_json.as_deref()
        .ok_or_else(|| AppError::Validation("No payload for generate_summary task".into()))?;
    let payload: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse payload: {}", e)))?;

    let app_data_dir_str = payload.get("app_data_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("No 'app_data_dir' in payload".into()))?;
    let app_data_dir = PathBuf::from(app_data_dir_str);
    let project_id = &task.project_id;

    tracing::info!(
        "Processing generate_summary task: id={}, project_id={}",
        task_id, project_id
    );

    // 进度回调闭包（与 sync_embeddings 模式一致）
    let task_id_for_progress = task_id.clone();
    let app_handle_for_progress = app_handle.clone();
    let pool_for_progress = pool.clone();
    let progress_callback = move |current: usize, total: usize| {
        let task_id = task_id_for_progress.clone();
        let ah = app_handle_for_progress.clone();
        let pool = pool_for_progress.clone();
        tauri::async_runtime::spawn(async move {
            let _ = async_task_repo::update_status(
                &pool,
                &task_id,
                TaskStatus::Running,
                Some(current as i64),
                Some(total as i64),
                None,
                None,
            )
            .await;
            let _ = ah.emit(
                "task-progress",
                serde_json::json!({
                    "task_id": task_id,
                    "status": "running",
                    "progress_current": current,
                    "progress_total": total
                }),
            );
        });
    };

    let result = crate::services::summary_service::generate_all_summaries(
        pool,
        &app_data_dir,
        project_id,
        progress_callback,
    )
    .await?;

    tracing::info!(
        "generate_summary task {} completed: success={}, failed={}, skipped={}, total={}",
        task_id,
        result.success_count,
        result.failed_count,
        result.skipped_count,
        result.total,
    );

    Ok(())
}

/// 处理 generate_snapshots 类型的任务：批量生成项目实体快照（逐章进度）
async fn process_generate_snapshots(
    pool: &SqlitePool,
    task: &AsyncTask,
    app_handle: &AppHandle,
) -> Result<(), AppError> {
    let task_id = &task.id;

    let payload_str = task.payload_json.as_deref()
        .ok_or_else(|| AppError::Validation("No payload for generate_snapshots task".into()))?;
    let payload: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| AppError::Internal(format!("Failed to parse payload: {}", e)))?;

    let app_data_dir_str = payload.get("app_data_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("No 'app_data_dir' in payload".into()))?;
    let app_data_dir = PathBuf::from(app_data_dir_str);
    let project_id = &task.project_id;

    tracing::info!(
        "Processing generate_snapshots task: id={}, project_id={}",
        task_id, project_id
    );

    // 进度回调闭包（与其他批量任务模式一致）
    let task_id_for_progress = task_id.clone();
    let app_handle_for_progress = app_handle.clone();
    let pool_for_progress = pool.clone();
    let progress_callback = move |current: usize, total: usize| {
        let task_id = task_id_for_progress.clone();
        let ah = app_handle_for_progress.clone();
        let pool = pool_for_progress.clone();
        tauri::async_runtime::spawn(async move {
            let _ = async_task_repo::update_status(
                &pool,
                &task_id,
                TaskStatus::Running,
                Some(current as i64),
                Some(total as i64),
                None,
                None,
            )
            .await;
            let _ = ah.emit(
                "task-progress",
                serde_json::json!({
                    "task_id": task_id,
                    "status": "running",
                    "progress_current": current,
                    "progress_total": total
                }),
            );
        });
    };

    let result = crate::services::entity_snapshot_service::generate_all_snapshots(
        pool,
        &app_data_dir,
        project_id,
        progress_callback,
    )
    .await?;

    tracing::info!(
        "generate_snapshots task {} completed: success={}, failed={}, skipped={}, total_chapters={}",
        task_id,
        result.success_count,
        result.failed_count,
        result.skipped_count,
        result.total_chapters,
    );

    Ok(())
}

// 定义 Tauri State 类型，使用 Arc 包装以便在多线程/异步环境中安全共享
pub type TaskManagerState = Arc<tokio::sync::RwLock<Option<TaskManager>>>;
