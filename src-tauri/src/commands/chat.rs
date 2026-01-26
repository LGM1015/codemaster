use tauri::{State, Window, Emitter};
use crate::commands::settings::AppState;
use crate::agent::r#loop::{Agent, AgentEvent};
use crate::api::deepseek::Message;
use tokio::sync::mpsc;

#[tauri::command]
pub async fn send_message(
    window: Window,
    state: State<'_, AppState>,
    message: String,
    history: Vec<Message>,
) -> Result<(), String> {
    let client_guard = state.client.lock().map_err(|_| "Failed to lock state")?;
    let client = client_guard.as_ref().ok_or("API Key not set")?.clone();
    drop(client_guard); // Release lock

    let registry = state.registry.clone();
    let agent = Agent::new(client, registry);

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn agent task
    tokio::spawn(async move {
        agent.run_task(message, history, tx).await;
    });

    // Forward events to frontend
    while let Some(event) = rx.recv().await {
        // Tauri v2 uses .emit() on Window (renamed from emit_all/emit in v1? v2 uses Emitter trait)
        // Window implements Emitter.
        if let Err(e) = window.emit("agent-event", event) {
            eprintln!("Failed to emit event: {}", e);
        }
    }

    Ok(())
}
