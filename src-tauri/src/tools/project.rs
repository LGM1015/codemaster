use crate::agent::registry::{Tool, ToolResult};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use walkdir::WalkDir;
use std::path::Path;

pub struct ProjectStructureTool;

impl Tool for ProjectStructureTool {
    fn name(&self) -> &str {
        "project_structure"
    }

    fn description(&self) -> &str {
        "Analyze project structure and identify key files."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project root path"
                },
                "depth": {
                    "type": "integer",
                    "description": "Max depth to traverse (default 2)"
                }
            },
            "required": ["path"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let path_str = args["path"].as_str().ok_or("Missing path parameter")?;
            let max_depth = args["depth"].as_u64().unwrap_or(2) as usize;
            
            let root = Path::new(path_str);
            if !root.exists() {
                return Err("Path does not exist".to_string());
            }

            let mut structure = String::new();
            let mut project_type = "unknown";

            // Identify project type
            if root.join("package.json").exists() {
                project_type = "nodejs";
            } else if root.join("Cargo.toml").exists() {
                project_type = "rust";
            } else if root.join("requirements.txt").exists() || root.join("pyproject.toml").exists() {
                project_type = "python";
            }

            structure.push_str(&format!("Project Type: {}\n", project_type));
            structure.push_str("Structure:\n");

            let walker = WalkDir::new(root).max_depth(max_depth).sort_by_file_name().into_iter();
            
            for entry in walker.filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "node_modules" && name != "target" && name != ".git" && !name.starts_with('.')
            }) {
                match entry {
                    Ok(e) => {
                        let depth = e.depth();
                        if depth == 0 { continue; }
                        
                        let indent = "  ".repeat(depth - 1);
                        let marker = if e.file_type().is_dir() { "/" } else { "" };
                        structure.push_str(&format!("{}├── {}{}\n", indent, e.file_name().to_string_lossy(), marker));
                    },
                    Err(_) => continue,
                }
            }

            Ok(structure)
        })
    }
}
