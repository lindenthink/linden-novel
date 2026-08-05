use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri::State;

use crate::error::AppError;
use crate::services::io_service::{self, ExportFormat};

/// 导出项目
#[tauri::command]
pub async fn export_project(
    pool: State<'_, SqlitePool>,
    project_id: String,
    format: String,
    path: String,
) -> Result<(), AppError> {
    let fmt: ExportFormat = format.parse().map_err(|e: anyhow::Error| {
        AppError::Validation(e.to_string())
    })?;

    let path_buf = PathBuf::from(&path);
    io_service::export_project(&pool, &project_id, fmt, &path_buf)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

/// 导入项目（JSON）
#[tauri::command]
pub async fn import_project(
    pool: State<'_, SqlitePool>,
    path: String,
) -> Result<String, AppError> {
    let path_buf = PathBuf::from(&path);
    io_service::import_project_json(&pool, &path_buf)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
