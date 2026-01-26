use crate::api::deepseek::{DeepSeekClient, Message, ToolCall};
use super::registry::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum AgentEvent {
    Thinking(String),
    ToolCall { name: String, args: String },
    ToolResult { name: String, result: String },
    Message(String),
    Error(String),
    Done,
}

pub struct Agent {
    client: DeepSeekClient,
    registry: Arc<ToolRegistry>,
    max_steps: u32,
}

impl Agent {
    pub fn new(client: DeepSeekClient, registry: Arc<ToolRegistry>) -> Self {
        Self {
            client,
            registry,
            max_steps: 20,
        }
    }

    pub async fn run_task(&self, task: String, mut history: Vec<Message>, tx: mpsc::Sender<AgentEvent>) {
        if history.is_empty() || history.last().map(|m| m.role.as_str()) != Some("user") {
             history.push(Message {
                role: "user".to_string(),
                content: Some(task),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        let mut steps = 0;
        let api_tools = self.registry.to_api_tools();
        let tools_option = if api_tools.is_empty() { None } else { Some(api_tools) };

        loop {
            if steps >= self.max_steps {
                let _ = tx.send(AgentEvent::Error("Max steps reached".to_string())).await;
                break;
            }
            steps += 1;
            
            let _ = tx.send(AgentEvent::Thinking("Thinking...".to_string())).await;

            // Clone tools for each request because chat_completion takes ownership
            let current_tools = tools_option.clone();

            let response = match self.client.chat_completion(history.clone(), current_tools).await {
                 Ok(res) => res,
                 Err(e) => {
                     let _ = tx.send(AgentEvent::Error(format!("API Error: {}", e))).await;
                     break;
                 }
            };

            let choice = match response.choices.first() {
                Some(c) => c,
                None => {
                    let _ = tx.send(AgentEvent::Error("No response from API".to_string())).await;
                    break;
                }
            };

            let message = choice.message.clone();
            history.push(message.clone());

            // Check for tool calls
            if let Some(tool_calls) = &message.tool_calls {
                if tool_calls.is_empty() {
                    if let Some(content) = message.content {
                        let _ = tx.send(AgentEvent::Message(content)).await;
                    }
                    break;
                }

                for tool_call in tool_calls {
                    let _ = tx.send(AgentEvent::ToolCall { 
                        name: tool_call.function.name.clone(), 
                        args: tool_call.function.arguments.clone() 
                    }).await;

                    let tool_name = &tool_call.function.name;
                    let args_str = &tool_call.function.arguments;
                    
                    let result = match self.registry.get(tool_name) {
                        Some(tool) => {
                            match serde_json::from_str::<serde_json::Value>(args_str) {
                                Ok(args) => tool.call(args).await,
                                Err(e) => Err(format!("Invalid JSON args: {}", e)),
                            }
                        },
                        None => Err(format!("Tool not found: {}", tool_name)),
                    };

                    let result_str = match result {
                        Ok(s) => s,
                        Err(e) => format!("Error: {}", e),
                    };

                    let _ = tx.send(AgentEvent::ToolResult { 
                        name: tool_name.clone(), 
                        result: result_str.clone() 
                    }).await;

                    history.push(Message {
                        role: "tool".to_string(),
                        content: Some(result_str),
                        tool_calls: None,
                        tool_call_id: Some(tool_call.id.clone()),
                        name: Some(tool_name.clone()),
                    });
                }
            } else {
                if let Some(content) = message.content {
                    let _ = tx.send(AgentEvent::Message(content)).await;
                }
                break;
            }
        }
        
        let _ = tx.send(AgentEvent::Done).await;
    }
}
