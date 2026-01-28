#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod agent;
mod tools;
mod api;
mod db;

use std::sync::{Mutex, Arc};
use commands::settings::AppState;
use commands::session::DbState;
use keyring::Entry;
use api::deepseek::DeepSeekClient;
use api::{UnifiedLLMClient, ModelConfig, ModelProvider};
use agent::registry::ToolRegistry;
use tools::file::{ReadFileTool, WriteFileTool, EditFileTool};
use tools::search::{GrepTool, GlobTool};
use tools::bash::BashTool;
use tools::project::ProjectStructureTool;
use db::Database;

fn main() {
    // Load saved provider preference
    let saved_provider = Entry::new("codemaster-app", "model-provider")
        .ok()
        .and_then(|e| e.get_password().ok())
        .unwrap_or_else(|| "deepseek".to_string());
    
    let current_provider = match saved_provider.as_str() {
        "qwen" => ModelProvider::Qwen,
        _ => ModelProvider::DeepSeek,
    };

    // Initialize DeepSeek client (legacy)
    let client = match Entry::new("codemaster-app", "deepseek-api-key") {
        Ok(entry) => match entry.get_password() {
            Ok(key) => Some(DeepSeekClient::new(key)),
            Err(_) => None,
        },
        Err(_) => None,
    };

    // Initialize unified client based on saved provider
    let unified_client = match &current_provider {
        ModelProvider::DeepSeek => {
            Entry::new("codemaster-app", "deepseek-api-key")
                .ok()
                .and_then(|e| e.get_password().ok())
                .map(|key| UnifiedLLMClient::new(ModelConfig::deepseek(key)))
        },
        ModelProvider::Qwen => {
            Entry::new("codemaster-app", "qwen-api-key")
                .ok()
                .and_then(|e| e.get_password().ok())
                .map(|key| UnifiedLLMClient::new(ModelConfig::qwen(key)))
        },
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
        unified_client: Mutex::new(unified_client),
        registry: Arc::new(registry),
        current_provider: Mutex::new(current_provider),
    };

    // Initialize database
    let database = match Database::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };
    
    let db_state = DbState {
        db: Arc::new(database),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .manage(db_state)
        .invoke_handler(tauri::generate_handler![
            commands::settings::set_api_key,
            commands::settings::get_api_key,
            commands::settings::test_connection,
            commands::settings::get_model_settings,
            commands::settings::set_model_settings,
            commands::settings::get_current_provider,
            commands::settings::test_model_connection,
            commands::chat::send_message,
            commands::session::create_session,
            commands::session::list_sessions,
            commands::session::get_session,
            commands::session::update_session_title,
            commands::session::delete_session,
            commands::session::get_session_messages,
            commands::session::save_message,
            commands::session::clear_session_messages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
