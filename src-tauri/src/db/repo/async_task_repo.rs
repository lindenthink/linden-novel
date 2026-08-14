use sqlx::SqlitePool;

use crate::db::pool;
use crate::error::AppError;
use crate::models::async_task::{AsyncTask, TaskStatus};

pub async fn create(
    pool: &SqlitePool,
    task: &AsyncTask,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO async_tasks (
            id, task_type, project_id, target_type, target_id, content_hash, payload_json,
            status, progress_current, progress_total,
            result_json, error_message,
            created_at, started_at, completed_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&task.id)
    .bind(&task.task_type)
    .bind(&task.project_id)
    .bind(&task.target_type)
    .bind(&task.target_id)
    .bind(&task.content_hash)
    .bind(&task.payload_json)
    .bind(&task.status)
    .bind(task.progress_current)
    .bind(task.progress_total)
    .bind(&task.result_json)
    .bind(&task.error_message)
    .bind(&task.created_at)
    .bind(&task.started_at)
    .bind(&task.completed_at)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<AsyncTask>, AppError> {
    sqlx::query_as::<_, AsyncTask>("SELECT * FROM async_tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
}

pub async fn list_by_project(
    pool: &SqlitePool,
    project_id: &str,
    status_filter: Option<&str>,
) -> Result<Vec<AsyncTask>, AppError> {
    let tasks = if let Some(status) = status_filter {
        sqlx::query_as::<_, AsyncTask>(
            "SELECT * FROM async_tasks WHERE project_id = ? AND status = ? ORDER BY created_at DESC"
        )
        .bind(project_id)
        .bind(status)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?
    } else {
        sqlx::query_as::<_, AsyncTask>(
            "SELECT * FROM async_tasks WHERE project_id = ? ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?
    };
    Ok(tasks)
}

/// 根据唯一索引检查是否已存在同类待处理/运行中任务（用于幂等）
pub async fn find_existing_pending_or_running(
    pool: &SqlitePool,
    task_type: &str,
    target_id: Option<&str>,
    content_hash: Option<&str>,
) -> Result<Option<AsyncTask>, AppError> {
    if target_id.is_none() || content_hash.is_none() {
        return Ok(None);
    }
    
    let target_id_unwrapped = target_id.unwrap();
    let content_hash_unwrapped = content_hash.unwrap();

    sqlx::query_as::<_, AsyncTask>(
        "SELECT * FROM async_tasks 
         WHERE task_type = ? 
         AND target_id = ? 
         AND content_hash = ?
         AND status IN ('pending', 'running')
         LIMIT 1"
    )
    .bind(task_type)
    .bind(target_id_unwrapped)
    .bind(content_hash_unwrapped)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

pub async fn update_status(
    pool: &SqlitePool,
    id: &str,
    status: TaskStatus,
    progress_current: Option<i64>,
    progress_total: Option<i64>,
    result_json: Option<&str>,
    error_message: Option<&str>,
) -> Result<(), AppError> {
    let now = pool::now();
    let status_str = status.to_string();
    
    sqlx::query(
        "UPDATE async_tasks SET 
            status = ?,
            progress_current = COALESCE(?, progress_current),
            progress_total = COALESCE(?, progress_total),
            result_json = COALESCE(?, result_json),
            error_message = COALESCE(?, error_message),
            completed_at = CASE WHEN ? IN ('completed', 'failed', 'cancelled') THEN ? ELSE completed_at END
         WHERE id = ?"
    )
    .bind(&status_str)
    .bind(progress_current)
    .bind(progress_total)
    .bind(result_json)
    .bind(error_message)
    .bind(&status_str)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn mark_running(
    pool: &SqlitePool,
    id: &str,
    progress_total: Option<i64>,
) -> Result<(), AppError> {
    let now = pool::now();
    sqlx::query(
        "UPDATE async_tasks SET 
            status = 'running',
            started_at = ?,
            progress_total = COALESCE(?, progress_total)
         WHERE id = ?"
    )
    .bind(&now)
    .bind(progress_total)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn update_progress(
    pool: &SqlitePool,
    id: &str,
    progress_current: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE async_tasks SET progress_current = ? WHERE id = ?"
    )
    .bind(progress_current)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

pub async fn update_result(
    pool: &SqlitePool,
    id: &str,
    status: TaskStatus,
    result_json: Option<&str>,
    error_message: Option<&str>,
) -> Result<(), AppError> {
    let now = pool::now();
    let status_str = status.to_string();
    
    sqlx::query(
        "UPDATE async_tasks SET 
            status = ?,
            result_json = ?,
            error_message = ?,
            completed_at = ?
         WHERE id = ?"
    )
    .bind(&status_str)
    .bind(result_json)
    .bind(error_message)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}

/// 标记所有 running 状态的任务为 failed（用于启动时恢复）
pub async fn reset_running_to_failed(pool: &SqlitePool) -> Result<(), AppError> {
    let now = pool::now();
    sqlx::query(
        "UPDATE async_tasks SET 
            status = 'failed',
            error_message = '应用异常退出',
            completed_at = ?
         WHERE status = 'running'"
    )
    .bind(&now)
    .execute(pool)
    .await
    .map_err(AppError::from)?;
    Ok(())
}
