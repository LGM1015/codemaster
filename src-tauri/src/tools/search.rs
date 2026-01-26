use crate::agent::registry::{Tool, ToolResult};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use walkdir::WalkDir;
use regex::Regex;
use glob::glob;

pub struct GrepTool;

impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for a pattern in files within a directory."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory path to search in (defaults to current dir)"
                },
                "include": {
                    "type": "string",
                    "description": "File pattern to include (e.g., *.rs)"
                }
            },
            "required": ["pattern"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let pattern_str = args["pattern"].as_str().ok_or("Missing pattern parameter")?;
            let path_str = args["path"].as_str().unwrap_or(".");
            let include_pattern = args["include"].as_str();

            let regex = Regex::new(pattern_str).map_err(|e| format!("Invalid regex: {}", e))?;
            
            let mut matches = Vec::new();
            let mut count = 0;
            
            for entry in WalkDir::new(path_str).into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() {
                    continue;
                }
                
                let file_path = entry.path();
                let path_string = file_path.to_string_lossy();
                
                // Skip hidden files/dirs and common ignore dirs
                if path_string.contains("/.") || path_string.contains("\\.") || 
                   path_string.contains("node_modules") || path_string.contains("target") || 
                   path_string.contains(".git") {
                    continue;
                }

                if let Some(inc) = include_pattern {
                     if let Ok(pattern) = glob::Pattern::new(inc) {
                         if !pattern.matches_path(file_path) {
                             continue;
                         }
                     }
                }

                if let Ok(content) = std::fs::read_to_string(file_path) {
                    for (i, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            matches.push(format!("{}:{}: {}", path_string, i + 1, line.trim()));
                            count += 1;
                            if count >= 100 {
                                break;
                            }
                        }
                    }
                }
                
                if count >= 100 {
                    break;
                }
            }
            
            if matches.is_empty() {
                Ok("No matches found.".to_string())
            } else {
                Ok(matches.join("\n"))
            }
        })
    }
}

pub struct GlobTool;

impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern (e.g., **/*.rs)"
                },
                "path": {
                    "type": "string",
                    "description": "Base path (optional)"
                }
            },
            "required": ["pattern"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let pattern_str = args["pattern"].as_str().ok_or("Missing pattern parameter")?;
            let base_path = args["path"].as_str().unwrap_or(".");
            
            let full_pattern = if base_path == "." {
                pattern_str.to_string()
            } else {
                format!("{}/{}", base_path, pattern_str)
            };

            let mut files = Vec::new();
            let mut count = 0;
            
            match glob(&full_pattern) {
                Ok(paths) => {
                    for entry in paths {
                        if let Ok(path) = entry {
                            files.push(path.display().to_string());
                            count += 1;
                            if count >= 100 {
                                break;
                            }
                        }
                    }
                },
                Err(e) => return Err(format!("Invalid glob pattern: {}", e)),
            }
            
            if files.is_empty() {
                Ok("No files found.".to_string())
            } else {
                Ok(files.join("\n"))
            }
        })
    }
}
