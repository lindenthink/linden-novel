use std::path::Path;

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::project::{CreateProject, Project, UpdateProject};
use crate::services::project_service;

#[tauri::command]
pub async fn list_projects(pool: State<'_, SqlitePool>) -> Result<Vec<Project>, AppError> {
    project_service::list(&pool).await
}

#[tauri::command]
pub async fn get_project(pool: State<'_, SqlitePool>, id: String) -> Result<Project, AppError> {
    project_service::get(&pool, &id).await
}

#[tauri::command]
pub async fn create_project(
    pool: State<'_, SqlitePool>,
    input: CreateProject,
) -> Result<Project, AppError> {
    project_service::create(&pool, &input).await
}

#[tauri::command]
pub async fn update_project(
    pool: State<'_, SqlitePool>,
    app: AppHandle,
    id: String,
    input: UpdateProject,
) -> Result<Project, AppError> {
    // 更新前查询旧 cover_path，用于后续清理磁盘文件
    let old_cover = project_service::get(&pool, &id)
        .await
        .ok()
        .and_then(|p| p.cover_path);

    let updated = project_service::update(&pool, &id, &input).await?;

    // 旧封面非空且更新后路径变了（清空 or 换新图）→ 删除旧文件
    if let Some(old_path) = old_cover {
        if updated.cover_path.as_deref() != Some(old_path.as_str()) {
            try_delete_cover_file(&app, &old_path);
        }
    }

    Ok(updated)
}

#[tauri::command]
pub async fn delete_project(
    pool: State<'_, SqlitePool>,
    app: AppHandle,
    id: String,
) -> Result<(), AppError> {
    // 删除前查询 cover_path，用于后续清理磁盘文件
    let old_cover = project_service::get(&pool, &id)
        .await
        .ok()
        .and_then(|p| p.cover_path);

    project_service::delete(&pool, &id).await?;

    if let Some(old_path) = old_cover {
        try_delete_cover_file(&app, &old_path);
    }

    Ok(())
}

const ALLOWED_COVER_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];

/// 将用户选择的本地图片复制到 app_data_dir/covers 下，返回相对路径（如 `covers/{uuid}.png`）。
///
/// DB 仅存相对路径；前端展示时拼接 appDataDir 后通过 `convertFileSrc` 渲染。
#[tauri::command]
pub async fn save_project_cover(app: AppHandle, file_path: String) -> Result<String, AppError> {
    let src = Path::new(&file_path);
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .filter(|e| ALLOWED_COVER_EXTS.contains(&e.as_str()))
        .ok_or_else(|| AppError::Validation("不支持的图片格式（仅允许 png/jpg/jpeg/gif/webp/bmp）".into()))?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("获取 app_data_dir 失败: {}", e)))?;
    let covers_dir = app_data_dir.join("covers");
    std::fs::create_dir_all(&covers_dir)
        .map_err(|e| AppError::Internal(format!("创建封面目录失败: {}", e)))?;

    let uuid = uuid::Uuid::new_v4().to_string();
    let filename = format!("{}.{}", uuid, ext);
    let dst = covers_dir.join(&filename);

    tokio::fs::copy(&src, &dst)
        .await
        .map_err(|e| AppError::Internal(format!("复制封面文件失败: {}", e)))?;

    Ok(format!("covers/{}", filename))
}

/// 删除 app_data_dir 下的封面文件（相对路径如 `covers/xxx.png`）。
///
/// 失败仅告警不阻塞业务操作，避免文件系统问题影响 DB 写入。
fn try_delete_cover_file(app: &AppHandle, cover_path: &str) {
    let abs = match app.path().app_data_dir() {
        Ok(dir) => dir.join(cover_path),
        Err(e) => {
            tracing::warn!("获取 app_data_dir 失败，跳过封面文件清理: {}", e);
            return;
        }
    };
    match std::fs::remove_file(&abs) {
        Ok(_) => tracing::info!("已删除封面文件: {:?}", abs),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!("封面文件不存在，跳过删除: {:?}", abs);
        }
        Err(e) => tracing::warn!("删除封面文件失败 {:?}: {}", abs, e),
    }
}
