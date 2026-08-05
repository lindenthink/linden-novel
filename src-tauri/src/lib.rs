mod ai;
mod commands;
mod crypto;
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
            // prompt_template
            commands::prompt_template::list_prompt_templates,
            commands::prompt_template::get_prompt_template,
            commands::prompt_template::list_prompt_templates_by_type,
            commands::prompt_template::create_prompt_template,
            commands::prompt_template::update_prompt_template,
            commands::prompt_template::delete_prompt_template,
            // ai_complete
            commands::ai_complete::ai_complete,
            commands::ai_complete::ai_complete_stream,
            commands::ai_complete::ai_render_template,
            // ai_generation
            commands::ai_generation::ai_generate,
            commands::ai_generation::list_ai_generation_history,
            commands::ai_generation::get_ai_generation_history,
            commands::ai_generation::delete_ai_generation_history,
            commands::ai_generation::delete_ai_generation_history_by_chapter,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
