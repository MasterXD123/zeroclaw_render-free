//! Google Workspace CLI integration tool.
//! Wraps the `gws` CLI for use as a ZeroClaw tool.

use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::Arc;

pub struct GoogleWorkspaceTool {
    #[allow(dead_code)]
    security: Arc<SecurityPolicy>,
    gws_path: String,
}

impl GoogleWorkspaceTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        let gws_path = std::env::var("GWS_PATH").unwrap_or_else(|_| {
            if cfg!(windows) {
                "C:\\Users\\user\\gws.exe".to_string()
            } else {
                "gws".to_string()
            }
        });
        Self { security, gws_path }
    }

    fn execute_gws(&self, args: &[String]) -> Result<String, String> {
        let output = Command::new(&self.gws_path)
            .args(args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .output()
            .map_err(|e| format!("Failed to execute gws: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("gws error: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_output(&self, output: &str) -> Result<Value, String> {
        serde_json::from_str(output).map_err(|e| format!("Failed to parse JSON: {}", e))
    }
}

#[async_trait]
impl Tool for GoogleWorkspaceTool {
    fn name(&self) -> &str {
        "google_workspace"
    }

    fn description(&self) -> &str {
        "Google Workspace CLI: Gmail, Drive, Calendar, Docs, Sheets integration"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "enum": ["gmail", "drive", "calendar", "docs", "sheets", "chat"],
                    "description": "Google Workspace service to use"
                },
                "action": {
                    "type": "string",
                    "enum": ["list", "create", "get", "delete", "send"],
                    "description": "Action to perform"
                },
                "params": { "type": "object", "description": "Parameters (JSON)" }
            },
            "required": ["service", "action"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let service = args
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("drive");
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        let params = args.get("params").cloned();

        let mut cmd_args: Vec<String> = vec![service.to_string()];

        match action {
            "list" => cmd_args.push("list".to_string()),
            "create" => cmd_args.push("create".to_string()),
            "get" => cmd_args.push("get".to_string()),
            "delete" => cmd_args.push("delete".to_string()),
            "send" => cmd_args.push("send".to_string()),
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unknown action: {}", action)),
                })
            }
        }

        if let Some(p) = params {
            if let Some(obj) = p.as_object() {
                if !obj.is_empty() {
                    cmd_args.push("--params".to_string());
                    let json_str =
                        serde_json::to_string(obj).map_err(|e| anyhow::anyhow!("{}", e))?;
                    cmd_args.push(json_str);
                }
            }
        }

        cmd_args.push("--json".to_string());

        let result = self.execute_gws(&cmd_args);

        match result {
            Ok(output) => match self.parse_output(&output) {
                Ok(json_val) => Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string(&json_val).unwrap_or(output),
                    error: None,
                }),
                Err(_) => Ok(ToolResult {
                    success: true,
                    output,
                    error: None,
                }),
            },
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e),
            }),
        }
    }
}
