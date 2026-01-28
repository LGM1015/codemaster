// Unified LLM client interface supporting multiple providers
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;

// Re-export types that are used across the codebase
pub use super::deepseek::{Message, Tool, StreamToolCall};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ModelProvider {
    DeepSeek,
    Qwen,
    // Can add more providers here
}

impl Default for ModelProvider {
    fn default() -> Self {
        ModelProvider::DeepSeek
    }
}

#[derive(Clone, Debug)]
pub struct ModelConfig {
    pub provider: ModelProvider,
    pub model_name: String,
    pub api_key: String,
    pub base_url: String,
}

impl ModelConfig {
    pub fn deepseek(api_key: String) -> Self {
        Self {
            provider: ModelProvider::DeepSeek,
            model_name: "deepseek-chat".to_string(),
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
        }
    }
    
    pub fn qwen(api_key: String) -> Self {
        Self {
            provider: ModelProvider::Qwen,
            model_name: "qwen-max".to_string(),
            api_key,
            // Qwen uses DashScope API
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct StreamChunk {
    pub id: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Deserialize, Debug)]
pub struct StreamChoice {
    pub index: i32,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Serialize, Debug)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    stream: bool,
}

#[derive(Clone)]
pub struct UnifiedLLMClient {
    config: ModelConfig,
    client: Client,
}

impl UnifiedLLMClient {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
    
    pub fn provider(&self) -> &ModelProvider {
        &self.config.provider
    }
    
    pub fn model_name(&self) -> &str {
        &self.config.model_name
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse, Box<dyn Error + Send + Sync>> {
        let request = ChatRequest {
            model: self.config.model_name.clone(),
            messages,
            tools,
            stream: false,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API Error: {}", error_text).into());
        }

        let chat_response = response.json::<ChatResponse>().await?;
        Ok(chat_response)
    }

    pub async fn chat_completion_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, Box<dyn Error + Send + Sync>>> + Send>>, Box<dyn Error + Send + Sync>> {
        let request = ChatRequest {
            model: self.config.model_name.clone(),
            messages,
            tools,
            stream: true,
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API Error: {}", error_text).into());
        }

        let stream = response.bytes_stream()
            .map(|result| {
                match result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        let mut chunks = Vec::new();
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    continue;
                                }
                                match serde_json::from_str::<StreamChunk>(data) {
                                    Ok(chunk) => chunks.push(Ok(chunk)),
                                    Err(e) => chunks.push(Err(format!("JSON Parse Error: {}", e).into())),
                                }
                            }
                        }
                        futures::stream::iter(chunks)
                    },
                    Err(e) => futures::stream::iter(vec![Err(e.into())]),
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }
}
