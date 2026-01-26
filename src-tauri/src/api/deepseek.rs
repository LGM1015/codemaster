use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;

const BASE_URL: &str = "https://api.deepseek.com/v1";

#[derive(Clone)]
pub struct DeepSeekClient {
    api_key: String,
    client: Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
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
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl DeepSeekClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse, Box<dyn Error + Send + Sync>> {
        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages,
            tools,
            stream: false,
        };

        let response = self.client
            .post(format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            model: "deepseek-chat".to_string(),
            messages,
            tools,
            stream: true,
        };

        let response = self.client
            .post(format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
