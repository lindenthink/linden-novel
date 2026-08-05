mod commands;
mod db;
mod error;
mod models;
mod services;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志：写 app 日志目录
    let log_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,linden_novel=debug".into());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 日志初始化
            let log_dir = app.path().app_log_dir()?.to_path_buf();
            std::fs::create_dir_all(&log_dir).ok();
            let log_file = std::fs::File::create(log_dir.join("linden.log")).ok();
            if let Some(file) = log_file {
                tracing_subscriber::fmt()
                    .with_env_filter(log_filter)
                    .with_writer(file)
                    .with_ansi(false)
                    .init();
            }

            // 初始化数据库
            let app_data_dir = app.path().app_data_dir()?.to_path_buf();
            let pool = tauri::async_runtime::block_on(db::pool::init_pool(&app_data_dir))
                .expect("Failed to initialize database");
            app.manage(pool);

            tracing::info!("Linden Novel started, DB at {:?}", app_data_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
