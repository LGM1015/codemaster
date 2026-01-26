use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use crate::api::deepseek::Tool as ApiTool;
use crate::api::deepseek::ToolFunction;

pub type ToolResult = Result<String, String>;

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value; // JSON Schema
    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn to_api_tools(&self) -> Vec<ApiTool> {
        self.tools.values().map(|t| ApiTool {
            r#type: "function".to_string(),
            function: ToolFunction {
                name: t.name().to_string(),
                description: t.description().to_string(),
                parameters: t.parameters(),
            },
        }).collect()
    }
}
