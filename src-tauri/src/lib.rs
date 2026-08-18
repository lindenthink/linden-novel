mod ai;
mod commands;
mod crypto;
mod db;
mod error;
mod models;
mod services;

use std::sync::Arc;
use services::task_manager::{TaskManager, TaskManagerState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志：写 app 日志目录，按天轮转
    let log_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,linden_novel=debug".into());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 日志初始化：文件（按天轮转）+ 控制台
            let log_dir = app.path().app_log_dir()?.to_path_buf();
            std::fs::create_dir_all(&log_dir).ok();

            let file_appender = tracing_appender::rolling::daily(&log_dir, "linden.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            // 将 _guard 保存到静态变量，防止被提前 drop
            // 使用 Box::leak 确保 guard 在应用生命周期内一直存活
            let guard = Box::new(_guard);
            let _static_guard: &'static tracing_appender::non_blocking::WorkerGuard =
                Box::leak(guard);

            tracing_subscriber::fmt()
                .with_env_filter(log_filter)
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                    "%Y-%m-%d %H:%M:%S".to_string(),
                ))
                .init();

            // 清理过期日志文件（按文件修改时间判断）
            cleanup_old_logs(&log_dir);

            // 初始化数据库（sqlite-vec 扩展静态链接，自动加载）
            let app_data_dir = app.path().app_data_dir()?.to_path_buf();
            let pool = tauri::async_runtime::block_on(db::pool::init_pool(&app_data_dir))
                .expect("Failed to initialize database");
            app.manage(pool.clone());

            // 初始化 TaskManager 并启动 Worker
            let (tx, rx) = tokio::sync::mpsc::channel(100);
            let task_manager = TaskManager::new_with_sender(pool.clone(), app.handle().clone(), tx);
            let task_manager_state = Arc::new(tokio::sync::RwLock::new(Some(task_manager.clone())));
            
            // 崩溃恢复：将所有 running 状态的任务标记为 failed
            if let Err(e) = tauri::async_runtime::block_on(
                crate::db::repo::async_task_repo::reset_running_to_failed(&pool)
            ) {
                tracing::error!("Failed to reset running tasks during startup: {}", e);
            }

            // 清理 3 天前的已完成任务，控制任务中心数据量
            match tauri::async_runtime::block_on(
                crate::db::repo::async_task_repo::delete_old_completed(&pool, 3)
            ) {
                Ok(n) => {
                    if n > 0 {
                        tracing::info!("Cleaned up {} old completed tasks (>3 days)", n);
                    }
                }
                Err(e) => tracing::warn!("Failed to clean up old tasks: {}", e),
            }
            
            // 存储 TaskManager 到 State
            app.manage(task_manager_state);
            
            // 启动后台 Worker 任务处理循环
            let app_handle_clone = app.handle().clone();
            let pool_clone = pool.clone();
            tauri::async_runtime::spawn(async move {
                TaskManager::run_worker(pool_clone, app_handle_clone, rx).await;
            });

            // 后台下载嵌入模型（不阻塞应用启动），下载完成后立即预加载 + warm-up，
            // 避免首次 RAG 查询时同步触发 OnceCell 初始化（约 19s 阻塞）
            let app_data_dir_for_spawn = app_data_dir.clone();
            tauri::async_runtime::spawn(async move {
                let embedder_dir = app_data_dir_for_spawn.join("embedder_model");
                if !ai::model_downloader::is_model_ready(&embedder_dir) {
                    tracing::info!("开始后台下载嵌入模型...");
                    match ai::model_downloader::ensure_model(&embedder_dir).await {
                        Ok(_) => tracing::info!("嵌入模型下载完成"),
                        Err(e) => {
                            tracing::error!("嵌入模型下载失败: {}", e);
                            return;
                        }
                    }
                } else {
                    tracing::info!("嵌入模型已就绪，跳过下载");
                }

                // 预加载 embedder（触发 OnceCell 初始化，Embedder::load 在 spawn_blocking 中执行避免阻塞 runtime）
                let dir_for_init = app_data_dir_for_spawn.clone();
                let init_result = tokio::task::spawn_blocking(move || {
                    ai::provider_factory::get_local_embedder(&dir_for_init)
                })
                .await;

                match init_result {
                    Ok(Ok(provider)) => {
                        // warm-up：跑一次 embed 触发推理图 JIT 优化，避免首次 RAG 查询慢
                        match provider
                            .embed(ai::provider::EmbeddingRequest {
                                model: String::new(),
                                input: "预热嵌入模型".to_string(),
                            })
                            .await
                        {
                            Ok(_) => tracing::info!("嵌入模型预加载 + warm-up 完成"),
                            Err(e) => tracing::warn!("嵌入模型 warm-up 失败: {}", e),
                        }
                    }
                    Ok(Err(e)) => tracing::warn!("嵌入模型预加载失败: {}", e),
                    Err(e) => tracing::warn!("嵌入模型预加载 spawn_blocking 失败: {}", e),
                }
            });

            tracing::info!("Linden Novel started, DB at {:?}", app_data_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // project
            commands::project::list_projects,
            commands::project::get_project,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::delete_project,
            // volume
            commands::volume::list_volumes,
            commands::volume::create_volume,
            commands::volume::update_volume,
            commands::volume::delete_volume,
            commands::volume::reorder_volumes,
            // chapter + content
            commands::chapter::list_chapters,
            commands::chapter::get_chapter,
            commands::chapter::create_chapter,
            commands::chapter::update_chapter_meta,
            commands::chapter::delete_chapter,
            commands::chapter::reorder_chapters,
            commands::chapter::get_chapter_content,
            commands::chapter::save_chapter_content,
            // settings
            commands::settings::get_setting,
            commands::settings::set_setting,
            // io (import/export)
            commands::io::export_project,
            commands::io::import_project,
            // character
            commands::character::list_characters,
            commands::character::get_character,
            commands::character::create_character,
            commands::character::update_character,
            commands::character::delete_character,
            // storyline
            commands::storyline::list_storylines,
            commands::storyline::get_storyline,
            commands::storyline::create_storyline,
            commands::storyline::update_storyline,
            commands::storyline::delete_storyline,
            // worldview
            commands::worldview::list_worldview,
            commands::worldview::get_worldview,
            commands::worldview::create_worldview,
            commands::worldview::update_worldview,
            commands::worldview::delete_worldview,
            // chapter_element
            commands::chapter_element::list_chapter_elements,
            commands::chapter_element::add_chapter_element,
            commands::chapter_element::remove_chapter_element,
            commands::chapter_element::remove_chapter_element_by_ref,
            // ai_provider
            commands::ai_provider::list_ai_providers,
            commands::ai_provider::get_ai_provider,
            commands::ai_provider::create_ai_provider,
            commands::ai_provider::update_ai_provider,
            commands::ai_provider::delete_ai_provider,
            commands::ai_provider::get_default_ai_provider,
            // ai_api_key
            commands::ai_api_key::list_ai_api_keys,
            commands::ai_api_key::create_ai_api_key,
            commands::ai_api_key::delete_ai_api_key,
            commands::ai_api_key::set_default_ai_api_key,
            // ai_complete
            commands::ai_complete::ai_complete,
            commands::ai_complete::ai_complete_stream,
            // ai_generation
            commands::ai_generation::ai_generate,
            commands::ai_generation::ai_generate_stream,
            commands::ai_generation::list_ai_generation_history,
            commands::ai_generation::get_ai_generation_history,
            commands::ai_generation::delete_ai_generation_history,
            commands::ai_generation::delete_ai_generation_history_by_chapter,
            // long_context (SP4)
            commands::long_context::generate_chapter_summary,
            commands::long_context::batch_generate_summaries,
            commands::long_context::sync_project_embeddings,
            commands::long_context::rag_search,
            commands::long_context::delete_project_embeddings,
            commands::long_context::get_chapter_summary,
            // entity_snapshot (SP4.5)
            commands::entity_snapshot::generate_chapter_snapshots,
            commands::entity_snapshot::batch_generate_snapshots,
            commands::entity_snapshot::get_entity_evolution,
            commands::entity_snapshot::list_chapter_snapshots,
            commands::entity_snapshot::delete_project_snapshots,
            commands::entity_snapshot::list_project_entities,
            // async tasks
            commands::tasks::submit_task,
            commands::tasks::list_tasks,
            commands::tasks::get_task,
            commands::tasks::cancel_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 清理过期日志文件
///
/// 保留天数通过环境变量 `LINDEN_LOG_RETENTION_DAYS` 配置，默认 7 天。
/// 按文件修改时间判断，仅删除 `linden.log` 前缀的轮转文件。
fn cleanup_old_logs(log_dir: &std::path::Path) {
    let retention_days = std::env::var("LINDEN_LOG_RETENTION_DAYS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(7);

    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(retention_days * 24 * 3600);

    let entries = match std::fs::read_dir(log_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut removed = 0u32;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.starts_with("linden.log") {
            continue;
        }
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    if std::fs::remove_file(&path).is_ok() {
                        removed += 1;
                    }
                }
            }
        }
    }

    if removed > 0 {
        tracing::info!(
            "Cleaned up {} log files older than {} days",
            removed,
            retention_days
        );
    }
}
