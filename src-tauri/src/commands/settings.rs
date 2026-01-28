use keyring::Entry;
use tauri::State;
use crate::api::{UnifiedLLMClient, ModelConfig, ModelProvider};
use crate::api::deepseek::DeepSeekClient;
use crate::agent::registry::ToolRegistry;
use std::sync::{Mutex, Arc};
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "codemaster-app";
const DEEPSEEK_KEY: &str = "deepseek-api-key";
const QWEN_KEY: &str = "qwen-api-key";
const MODEL_PROVIDER_KEY: &str = "model-provider";

#[derive(Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub provider: String,
    pub deepseek_key: Option<String>,
    pub qwen_key: Option<String>,
}

pub struct AppState {
    pub client: Mutex<Option<DeepSeekClient>>,
    pub unified_client: Mutex<Option<UnifiedLLMClient>>,
    pub registry: Arc<ToolRegistry>,
    pub current_provider: Mutex<ModelProvider>,
}

fn get_key(name: &str) -> Option<String> {
    Entry::new(SERVICE_NAME, name)
        .ok()
        .and_then(|e| e.get_password().ok())
}

fn set_key(name: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, name).map_err(|e| e.to_string())?;
    entry.set_password(value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_api_key(state: State<'_, AppState>, api_key: String) -> Result<(), String> {
    // Legacy: save as DeepSeek key
    set_key(DEEPSEEK_KEY, &api_key)?;

    // Update the client in state
    let mut client_lock = state.client.lock().map_err(|_| "Failed to lock state")?;
    *client_lock = Some(DeepSeekClient::new(api_key.clone()));

    // Also update unified client
    let mut unified_lock = state.unified_client.lock().map_err(|_| "Failed to lock unified state")?;
    *unified_lock = Some(UnifiedLLMClient::new(ModelConfig::deepseek(api_key)));

    Ok(())
}

#[tauri::command]
pub fn get_api_key() -> Result<String, String> {
    get_key(DEEPSEEK_KEY).ok_or_else(|| "No API key found".to_string())
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

// New multi-model commands
#[tauri::command]
pub fn get_model_settings() -> Result<ModelSettings, String> {
    Ok(ModelSettings {
        provider: get_key(MODEL_PROVIDER_KEY).unwrap_or_else(|| "deepseek".to_string()),
        deepseek_key: get_key(DEEPSEEK_KEY),
        qwen_key: get_key(QWEN_KEY),
    })
}

#[tauri::command]
pub fn set_model_settings(
    state: State<'_, AppState>,
    provider: String,
    deepseek_key: Option<String>,
    qwen_key: Option<String>,
) -> Result<(), String> {
    // Save keys
    if let Some(key) = &deepseek_key {
        set_key(DEEPSEEK_KEY, key)?;
    }
    if let Some(key) = &qwen_key {
        set_key(QWEN_KEY, key)?;
    }
    set_key(MODEL_PROVIDER_KEY, &provider)?;

    // Update provider
    let model_provider = match provider.as_str() {
        "qwen" => ModelProvider::Qwen,
        _ => ModelProvider::DeepSeek,
    };

    // Update current provider
    {
        let mut provider_lock = state.current_provider.lock().map_err(|_| "Failed to lock")?;
        *provider_lock = model_provider.clone();
    }

    // Update unified client based on selected provider
    let config = match model_provider {
        ModelProvider::Qwen => {
            let key = qwen_key.or_else(|| get_key(QWEN_KEY))
                .ok_or("Qwen API key not set")?;
            ModelConfig::qwen(key)
        },
        ModelProvider::DeepSeek => {
            let key = deepseek_key.clone().or_else(|| get_key(DEEPSEEK_KEY))
                .ok_or("DeepSeek API key not set")?;
            ModelConfig::deepseek(key)
        },
    };

    let mut unified_lock = state.unified_client.lock().map_err(|_| "Failed to lock")?;
    *unified_lock = Some(UnifiedLLMClient::new(config));

    // Also update legacy client for backward compatibility
    if let Some(key) = deepseek_key.or_else(|| get_key(DEEPSEEK_KEY)) {
        let mut client_lock = state.client.lock().map_err(|_| "Failed to lock")?;
        *client_lock = Some(DeepSeekClient::new(key));
    }

    Ok(())
}

#[tauri::command]
pub fn get_current_provider(state: State<'_, AppState>) -> Result<String, String> {
    let provider = state.current_provider.lock().map_err(|_| "Failed to lock")?;
    Ok(match *provider {
        ModelProvider::DeepSeek => "deepseek".to_string(),
        ModelProvider::Qwen => "qwen".to_string(),
    })
}

#[tauri::command]
pub async fn test_model_connection(provider: String, api_key: String) -> Result<String, String> {
    let config = match provider.as_str() {
        "qwen" => ModelConfig::qwen(api_key),
        _ => ModelConfig::deepseek(api_key),
    };
    
    let client = UnifiedLLMClient::new(config);
    let messages = vec![crate::api::deepseek::Message {
        role: "user".to_string(),
        content: Some("ping".to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }];
    
    match client.chat_completion(messages, None).await {
        Ok(_) => Ok(format!("{} connection successful", provider)),
        Err(e) => Err(e.to_string()),
    }
}
