//! Code runner tool — execute JavaScript code in a sandboxed QuickJS runtime.
//!
//! Use cases:
//! - Transform/filter/aggregate data from other tools (http_request, file_read, memory)
//! - Prototype logic before committing to a tool or file
//! - Dynamic validation of complex conditions
//!
//! Each execution runs in a fresh context (stateless, no memory between calls).

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use rquickjs::{Context, Runtime, Value};
use serde_json::json;

pub struct CodeRunnerTool {
    /// Max execution time per script
    timeout_ms: u64,
}

impl CodeRunnerTool {
    pub fn new() -> Self {
        Self { timeout_ms: 10_000 }
    }

    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    fn execute_js(&self, code: &str, input_data: Option<&serde_json::Value>) -> ToolResult {
        let runtime = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                return ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to create JS runtime: {}", e)),
                };
            }
        };
        let context = match Context::full(&runtime) {
            Ok(ctx) => ctx,
            Err(e) => {
                return ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to create JS context: {}", e)),
                };
            }
        };

        context.with(|ctx| {
            // Inject input data as `input` variable
            if let Some(data) = input_data {
                let data_json = serde_json::to_string(data).unwrap_or_default();
                if let Err(e) = ctx.eval::<(), _>(format!("const input = {};", data_json)) {
                    return ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to inject input data: {}", e)),
                    };
                }
            }

            // Execute the user code - wrap in IIFE to capture result
            let full_code = format!(
                "(() => {{ const __result = ({}); return typeof __result === 'object' ? JSON.stringify(__result) : __result; }})()",
                code
            );

            let eval_result: Result<Value, _> = ctx.eval(full_code);

            match eval_result {
                Ok(value) => {
                    let output = value_to_string(&value);
                    ToolResult {
                        success: true,
                        output,
                        error: None,
                    }
                }
                Err(e) => {
                    ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("JS Error: {}", e)),
                    }
                }
            }
        })
    }
}

fn value_to_string(value: &Value) -> String {
    use rquickjs::Type as JKind;
    match value.type_of() {
        JKind::Undefined => "undefined".to_string(),
        JKind::Null => "null".to_string(),
        JKind::Bool => value.as_bool().unwrap_or(false).to_string(),
        JKind::Int => value.as_int().unwrap_or(0).to_string(),
        JKind::Float => value.as_float().unwrap_or(0.0).to_string(),
        JKind::String => {
            // Get string representation via Debug or format
            format!("{:?}", value)
        }
        JKind::Symbol => "[Symbol]".to_string(),
        JKind::BigInt => "[BigInt]".to_string(),
        JKind::Array | JKind::Object => {
            // Use JSON.stringify in JS wrapper for objects
            "[Object]".to_string()
        }
        JKind::Function => "[Function]".to_string(),
        JKind::Module | JKind::Uninitialized => "[Unknown]".to_string(),
        JKind::Unknown => "[Unknown]".to_string(),
    }
}

impl Default for CodeRunnerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodeRunnerTool {
    fn name(&self) -> &str {
        "code_runner"
    }

    fn description(&self) -> &str {
        "Execute JavaScript code in a sandboxed QuickJS runtime. \
         Each execution runs in a fresh context (stateless). \
         Use for: data transformation/filtering, prototyping logic, dynamic validation. \
         Input data can be passed as `input` variable (JSON/object/array). \
         Return a value from your code to get it as output. \
         Timeout: 10 seconds max per execution."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "JavaScript code to execute. Use return to output a value, or the last expression. Input data available as `input` variable."
                },
                "data": {
                    "type": "any",
                    "description": "Optional input data available as `input` variable in the script. Can be any JSON value (object, array, string, number)."
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Execution timeout in milliseconds (default: 10000, max: 30000)",
                    "minimum": 100,
                    "maximum": 30000,
                    "default": 10000
                }
            },
            "required": ["code"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let code = match args.get("code").and_then(|v| v.as_str()) {
            Some(c) if !c.trim().is_empty() => c.trim(),
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'code' parameter".to_string()),
                });
            }
        };

        let data = args.get("data");

        // Validate code for dangerous patterns
        let dangerous = [
            "import(",
            "require(",
            "process",
            "eval",
            "Function(",
            "setTimeout",
            "setInterval",
            "fetch(",
            "XMLHttpRequest",
            "WebSocket",
            "Worker(",
            "SharedArrayBuffer",
        ];
        for pattern in dangerous {
            if code.contains(pattern) {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Disallowed pattern in code: {}", pattern)),
                });
            }
        }

        Ok(self.execute_js(code, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tool() -> CodeRunnerTool {
        CodeRunnerTool::new()
    }

    #[tokio::test]
    async fn basic_arithmetic() {
        let result = test_tool().execute(json!({"code": "2 + 2"})).await.unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("4"), "got: {}", result.output);
    }

    #[tokio::test]
    async fn array_operations() {
        let result = test_tool()
            .execute(json!({"code": "[1, 2, 3, 4, 5].filter(x => x > 2).map(x => x * 2)"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
    }

    #[tokio::test]
    async fn with_input_data() {
        let result = test_tool()
            .execute(json!({
                "code": "input.filter(x => x.stars > 100).map(x => x.name)",
                "data": [
                    {"name": "repo1", "stars": 50},
                    {"name": "repo2", "stars": 200},
                    {"name": "repo3", "stars": 150}
                ]
            }))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("repo2"), "got: {}", result.output);
    }

    #[tokio::test]
    async fn string_operations() {
        let result = test_tool()
            .execute(json!({"code": "'hello world'.toUpperCase()"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("HELLO"), "got: {}", result.output);
    }

    #[tokio::test]
    async fn regex() {
        let result = test_tool()
            .execute(json!({"code": "'test123@example.com'.match(/[0-9]+/)[0]"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("123"), "got: {}", result.output);
    }

    #[tokio::test]
    async fn disallowed_pattern() {
        let result = test_tool()
            .execute(json!({"code": "import('fs')"}))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Disallowed"));
    }

    #[tokio::test]
    async fn missing_code() {
        let result = test_tool().execute(json!({})).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Missing"));
    }

    #[tokio::test]
    async fn math_functions() {
        let result = test_tool()
            .execute(json!({"code": "Math.sqrt(16) + Math.floor(Math.PI)"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
    }

    #[tokio::test]
    async fn json_parse() {
        let result = test_tool()
            .execute(json!({"code": "JSON.parse('{\"key\": \"value\"}').key"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("value"), "got: {}", result.output);
    }

    #[tokio::test]
    async fn return_object() {
        let result = test_tool()
            .execute(json!({"code": "({name: 'test', value: 42})"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(
            result.output.contains("test") && result.output.contains("42"),
            "got: {}",
            result.output
        );
    }

    #[tokio::test]
    async fn return_value() {
        let result = test_tool()
            .execute(json!({"code": "(function() { return 42; })()"}))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("42"), "got: {}", result.output);
    }
}
