use crate::api::deepseek::{DeepSeekClient, Message, ToolCall, FunctionCall};
use super::registry::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use futures::StreamExt;

const SYSTEM_PROMPT: &str = r#"你是 CodeMaster，一个专业的 AI 编码助手，专为中国开发者设计。

## 核心能力
你拥有以下工具来帮助用户完成编码任务：
- **read_file**: 读取文件内容，支持指定行范围
- **write_file**: 创建或覆盖文件
- **edit_file**: 精确替换文件中的字符串
- **grep**: 使用正则表达式搜索代码
- **glob**: 查找匹配模式的文件
- **bash**: 执行 Shell/PowerShell 命令
- **project_structure**: 获取项目目录结构

## 工作原则
1. **先理解后行动**: 在修改代码前，先阅读相关文件了解上下文
2. **最小改动原则**: 只修改必要的代码，不要过度重构
3. **安全第一**: 执行命令前考虑潜在风险，避免破坏性操作
4. **清晰沟通**: 用中文解释你的思路和操作

## 响应风格
- 简洁专业，避免冗余
- 代码修改时展示关键改动
- 遇到错误时分析原因并提供解决方案
- 支持中英文双语交流

当前工作环境: Windows + PowerShell
"#;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum AgentEvent {
    Thinking(String),
    StreamChunk(String),         // New: streaming text chunk
    StreamEnd,                   // New: streaming ended for current message
    ToolCall { name: String, args: String, id: String },
    ToolResult { name: String, result: String, id: String },
    Message(String),
    NewMessage(Message),
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
        // Ensure system prompt is at the beginning
        let has_system = history.first().map(|m| m.role == "system").unwrap_or(false);
        if !has_system {
            history.insert(0, Message {
                role: "system".to_string(),
                content: Some(SYSTEM_PROMPT.to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

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

            let current_tools = tools_option.clone();

            // Retry logic for API calls
            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 3;
            
            let mut stream_result = Err("Initial error".to_string());
            
            while retry_count < MAX_RETRIES {
                match self.client.chat_completion_stream(history.clone(), current_tools.clone()).await {
                    Ok(s) => {
                        stream_result = Ok(s);
                        break;
                    },
                    Err(e) => {
                        retry_count += 1;
                        if retry_count < MAX_RETRIES {
                            let _ = tx.send(AgentEvent::Thinking(format!("Network error, retrying ({}/{})...", retry_count, MAX_RETRIES))).await;
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000 * retry_count as u64)).await;
                        } else {
                            stream_result = Err(e.to_string());
                        }
                    }
                }
            }
            
            let mut stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error(format!("API Error after retries: {}", e))).await;
                    break;
                }
            };

            // Accumulate streaming response
            let mut content_buffer = String::new();
            let mut tool_calls_map: std::collections::HashMap<i32, ToolCall> = std::collections::HashMap::new();
            let mut _finish_reason: Option<String> = None;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            // Handle content delta
                            if let Some(content) = &choice.delta.content {
                                content_buffer.push_str(content);
                                let _ = tx.send(AgentEvent::StreamChunk(content.clone())).await;
                            }
                            
                            // Handle tool calls delta
                            if let Some(tool_calls) = &choice.delta.tool_calls {
                                for tc in tool_calls {
                                    let entry = tool_calls_map.entry(tc.index).or_insert_with(|| ToolCall {
                                        id: String::new(),
                                        r#type: "function".to_string(),
                                        function: FunctionCall {
                                            name: String::new(),
                                            arguments: String::new(),
                                        },
                                    });
                                    
                                    if let Some(id) = &tc.id {
                                        entry.id = id.clone();
                                    }
                                    if let Some(name) = &tc.function.as_ref().and_then(|f| f.name.clone()) {
                                        entry.function.name = name.clone();
                                    }
                                    if let Some(args) = &tc.function.as_ref().and_then(|f| f.arguments.clone()) {
                                        entry.function.arguments.push_str(args);
                                    }
                                }
                            }
                            
                            if choice.finish_reason.is_some() {
                                _finish_reason = choice.finish_reason.clone();
                            }
                        }
                    },
                    Err(e) => {
                        let _ = tx.send(AgentEvent::Error(format!("Stream Error: {}", e))).await;
                        break;
                    }
                }
            }

            let _ = tx.send(AgentEvent::StreamEnd).await;

            // Build complete message
            let tool_calls_vec: Vec<ToolCall> = tool_calls_map.into_values().collect();
            let message = Message {
                role: "assistant".to_string(),
                // Ensure content is never null (use empty string for tool calls)
                content: Some(content_buffer.clone()), 
                tool_calls: if tool_calls_vec.is_empty() { None } else { Some(tool_calls_vec.clone()) },
                tool_call_id: None,
                name: None,
            };
            
            // Sync state with frontend
            let _ = tx.send(AgentEvent::NewMessage(message.clone())).await;
            
            history.push(message.clone());

            // Check for tool calls
            if let Some(tool_calls) = &message.tool_calls {
                if tool_calls.is_empty() {
                    break;
                }

                for tool_call in tool_calls {
                    let _ = tx.send(AgentEvent::ToolCall { 
                        name: tool_call.function.name.clone(), 
                        args: tool_call.function.arguments.clone(),
                        id: tool_call.id.clone()
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
                        result: result_str.clone(),
                        id: tool_call.id.clone()
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
                break;
            }
        }
        
        let _ = tx.send(AgentEvent::Done).await;
    }
}
