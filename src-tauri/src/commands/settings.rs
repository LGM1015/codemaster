use keyring::Entry;
use tauri::State;
use crate::api::deepseek::DeepSeekClient;
use crate::agent::registry::ToolRegistry;
use std::sync::{Mutex, Arc};

const SERVICE_NAME: &str = "codemaster-app";
const USER_NAME: &str = "deepseek-api-key";

pub struct AppState {
    pub client: Mutex<Option<DeepSeekClient>>,
    pub registry: Arc<ToolRegistry>,
}

#[tauri::command]
pub fn set_api_key(state: State<'_, AppState>, api_key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, USER_NAME).map_err(|e| e.to_string())?;
    entry.set_password(&api_key).map_err(|e| e.to_string())?;

    // Update the client in state
    let mut client_lock = state.client.lock().map_err(|_| "Failed to lock state")?;
    *client_lock = Some(DeepSeekClient::new(api_key));

    Ok(())
}

#[tauri::command]
pub fn get_api_key() -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, USER_NAME).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(api_key: String) -> Result<String, String> {
    let client = DeepSeekClient::new(api_key);
    let messages = vec![crate::api::deepseek::Message {
        role: "user".to_string(),
        content: Some("ping".to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];
    
    match client.chat_completion(messages, None).await {
        Ok(_) => Ok("Connection successful".to_string()),
        Err(e) => Err(e.to_string()),
    }
}
