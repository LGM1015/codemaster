use crate::api::deepseek::Message;
use crate::db::{Database, Session};
use std::sync::Arc;
use tauri::State;

pub struct DbState {
    pub db: Arc<Database>,
}

#[tauri::command]
pub fn create_session(state: State<'_, DbState>, title: String) -> Result<Session, String> {
    state.db.create_session(&title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sessions(state: State<'_, DbState>) -> Result<Vec<Session>, String> {
    state.db.list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(state: State<'_, DbState>, id: String) -> Result<Option<Session>, String> {
    state.db.get_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_session_title(
    state: State<'_, DbState>,
    id: String,
    title: String,
) -> Result<(), String> {
    state
        .db
        .update_session_title(&id, &title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_session(state: State<'_, DbState>, id: String) -> Result<(), String> {
    state.db.delete_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session_messages(
    state: State<'_, DbState>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    let db_messages = state
        .db
        .get_messages(&session_id)
        .map_err(|e| e.to_string())?;

    // Convert SessionMessage to API Message format
    let messages: Vec<Message> = db_messages
        .into_iter()
        .map(|m| {
            Message {
                role: m.role,
                // Ensure content is never null (use empty string)
                content: m.content.or(Some("".to_string())),
                tool_calls: m.tool_calls.and_then(|tc| serde_json::from_str(&tc).ok()),
                tool_call_id: m.tool_call_id,
                name: m.name,
            }
        })
        .collect();

    Ok(messages)
}

#[tauri::command]
pub fn save_message(
    state: State<'_, DbState>,
    session_id: String,
    role: String,
    content: Option<String>,
    tool_calls: Option<String>,
    tool_call_id: Option<String>,
    name: Option<String>,
) -> Result<i64, String> {
    // Also update session timestamp
    state.db.touch_session(&session_id).ok();

    state
        .db
        .add_message(
            &session_id,
            &role,
            content.as_deref(),
            tool_calls.as_deref(),
            tool_call_id.as_deref(),
            name.as_deref(),
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_session_messages(state: State<'_, DbState>, session_id: String) -> Result<(), String> {
    state
        .db
        .clear_messages(&session_id)
        .map_err(|e| e.to_string())
}
