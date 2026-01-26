use crate::agent::registry::{Tool, ToolResult};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use tokio::process::Command;
use std::time::Duration;

pub struct BashTool;

impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a shell command. On Windows uses PowerShell."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command to execute"
                },
                "workdir": {
                    "type": "string",
                    "description": "Working directory"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds (default 120)"
                }
            },
            "required": ["command"]
        })
    }

    fn call(&self, args: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> {
        Box::pin(async move {
            let command_str = args["command"].as_str().ok_or("Missing command parameter")?;
            let workdir = args["workdir"].as_str();
            let timeout_secs = args["timeout"].as_u64().unwrap_or(120);

            let dangerous = ["rm -rf /", "format c:", "rd /s /q c:\\"];
            for d in dangerous {
                if command_str.to_lowercase().contains(d) {
                    return Err(format!("Command blocked for security: {}", d));
                }
            }

            let mut cmd = if cfg!(target_os = "windows") {
                let mut c = Command::new("powershell");
                c.args(&["-Command", command_str]);
                c
            } else {
                let mut c = Command::new("bash");
                c.args(&["-c", command_str]);
                c
            };

            if let Some(wd) = workdir {
                cmd.current_dir(wd);
            }

            let child = cmd.stdout(std::process::Stdio::piped())
                           .stderr(std::process::Stdio::piped())
                           .spawn()
                           .map_err(|e| format!("Failed to spawn command: {}", e))?;

            let output = match tokio::time::timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await {
                Ok(result) => result.map_err(|e| format!("Failed to wait for command: {}", e))?,
                Err(_) => return Err(format!("Command timed out after {} seconds", timeout_secs)),
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            if output.status.success() {
                // If there's stderr even on success (warnings), append it
                if !stderr.trim().is_empty() {
                    Ok(format!("{}\nWARNINGS:\n{}", stdout, stderr))
                } else {
                    Ok(stdout.to_string())
                }
            } else {
                Ok(format!("Exit Code: {}\nStderr: {}\nStdout: {}", exit_code, stderr, stdout))
            }
        })
    }
}
