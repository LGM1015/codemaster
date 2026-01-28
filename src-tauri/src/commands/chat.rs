use tauri::{State, Window, Emitter};
use crate::commands::settings::AppState;
use crate::agent::r#loop::Agent;
use crate::api::deepseek::Message;
use tokio::sync::mpsc;

#[tauri::command]
pub async fn send_message(
    window: Window,
    state: State<'_, AppState>,
    message: String,
    history: Vec<Message>,
) -> Result<(), String> {
    let client = {
        let guard = state.client.lock().map_err(|_| "Failed to lock state")?;
        guard.as_ref().ok_or("API Key not set")?.clone()
    };

    let registry = state.registry.clone();
    let agent = Agent::new(client, registry);

    let (tx, mut rx) = mpsc::channel(100);

    // Spawn agent task
    tokio::spawn(async move {
        agent.run_task(message, history, tx).await;
    });

    // Forward events to frontend
    while let Some(event) = rx.recv().await {
        // Tauri v2 uses .emit() on Window
        if let Err(e) = window.emit("agent-event", event) {
            eprintln!("Failed to emit event: {}", e);
        }
    }

    Ok(())
}
