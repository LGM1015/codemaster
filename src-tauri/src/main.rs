#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod agent;
mod tools;
mod api;
mod db;

use std::sync::{Mutex, Arc};
use commands::settings::AppState;
use keyring::Entry;
use api::deepseek::DeepSeekClient;
use agent::registry::ToolRegistry;
use tools::file::{ReadFileTool, WriteFileTool, EditFileTool};
use tools::search::{GrepTool, GlobTool};
use tools::bash::BashTool;
use tools::project::ProjectStructureTool;

fn main() {
    let client = match Entry::new("codemaster-app", "deepseek-api-key") {
        Ok(entry) => match entry.get_password() {
            Ok(key) => Some(DeepSeekClient::new(key)),
            Err(_) => None,
        },
        Err(_) => None,
    };

    let mut registry = ToolRegistry::new();
    registry.register(ReadFileTool);
    registry.register(WriteFileTool);
    registry.register(EditFileTool);
    registry.register(GrepTool);
    registry.register(GlobTool);
    registry.register(BashTool);
    registry.register(ProjectStructureTool);

    let app_state = AppState {
        client: Mutex::new(client),
        registry: Arc::new(registry),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::settings::set_api_key,
            commands::settings::get_api_key,
            commands::settings::test_connection,
            commands::chat::send_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
