#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod state;
mod commands;

use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::Manager;
use state::{AppState, AppSettings};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("stateforge=debug".parse().unwrap()))
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&data_dir).unwrap();
            let db_path = data_dir.join("stateforge.db");

            let rt = tokio::runtime::Handle::current();
            let pool = rt.block_on(sf_core::db::open(&db_path)).unwrap();

            let settings_json = rt.block_on(sf_core::db::queries::get_setting(&pool, "app_settings"))
                .unwrap_or(None);
            let settings: AppSettings = settings_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            app.manage(AppState {
                pool,
                settings: Arc::new(RwLock::new(settings)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Parser
            commands::parse_code,
            commands::parse_log,
            commands::detect_language,
            // Diagram
            commands::render_diagram,
            commands::render_diagram_from_machine,
            commands::get_diagram_graph,
            commands::update_machine_from_graph,
            // Generator
            commands::generate_code,
            commands::supported_languages,
            // Machines
            commands::list_machines,
            commands::get_machine,
            commands::save_machine,
            commands::delete_machine,
            commands::add_state,
            commands::add_transition,
            commands::new_machine,
            commands::analyze_machine,
            // Settings
            commands::get_settings,
            commands::save_settings,
            commands::save_api_key,
            commands::has_api_key,
            // AI
            commands::ai_enhance_machine,
            commands::ai_from_description,
            commands::check_ai_available,
        ])
        .run(tauri::generate_context!())
        .expect("error running StateForge");
}
