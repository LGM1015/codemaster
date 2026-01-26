use crate::agent::registry::{Tool, ToolResult};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::fs;
use std::path::Path;

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read content from a file. Can verify file existence."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file"
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (0-based)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of lines to read"
                }
            },
            "required": ["path"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let path_str = args["path"].as_str().ok_or("Missing path parameter")?;
            let offset = args["offset"].as_u64().unwrap_or(0);
            let limit = args["limit"].as_u64(); // None means read all

            let metadata = fs::metadata(path_str).map_err(|e| format!("File not found or inaccessible: {}", e))?;
            if metadata.len() > 5 * 1024 * 1024 {
                return Err("File too large (>5MB). Use search or read specific lines.".to_string());
            }

            let content = fs::read_to_string(path_str).map_err(|e| format!("Failed to read file: {}", e))?;
            
            let lines: Vec<&str> = content.lines().collect();
            let start = offset as usize;
            
            if start >= lines.len() && !lines.is_empty() {
                 return Err(format!("Offset {} is out of bounds (file has {} lines)", start, lines.len()));
            }
            
            let end = match limit {
                Some(l) => std::cmp::min(start + l as usize, lines.len()),
                None => lines.len(),
            };
            
            let result_lines = &lines[start..end];
            
            // Add line numbers for better context
            let result = result_lines.iter().enumerate()
                .map(|(i, line)| format!("{:4} | {}", start + i + 1, line))
                .collect::<Vec<String>>()
                .join("\n");
                
            Ok(result)
        })
    }
}

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Overwrites existing files. Creates directories if needed."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let path_str = args["path"].as_str().ok_or("Missing path parameter")?;
            let content = args["content"].as_str().ok_or("Missing content parameter")?;
            
            let path = Path::new(path_str);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            
            fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;
            
            Ok(format!("Successfully wrote to {}", path_str))
        })
    }
}

pub struct EditFileTool;

impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Replace a string in a file with a new string. Use this for small edits."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file"
                },
                "old_string": {
                    "type": "string",
                    "description": "The string to replace (must match exactly)"
                },
                "new_string": {
                    "type": "string",
                    "description": "The new string"
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let path_str = args["path"].as_str().ok_or("Missing path parameter")?;
            let old_string = args["old_string"].as_str().ok_or("Missing old_string parameter")?;
            let new_string = args["new_string"].as_str().ok_or("Missing new_string parameter")?;
            
            let content = fs::read_to_string(path_str).map_err(|e| format!("Failed to read file: {}", e))?;
            
            if !content.contains(old_string) {
                return Err("old_string not found in file".to_string());
            }
            
            let new_content = content.replacen(old_string, new_string, 1);
            
            fs::write(path_str, new_content).map_err(|e| format!("Failed to write file: {}", e))?;
            
            Ok(format!("Successfully edited {}", path_str))
        })
    }
}
