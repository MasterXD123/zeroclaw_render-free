//! Validator tool — validate JSON data against JSON Schema.
//!
//! Use cases:
//! - Validate API responses against expected schema
//! - Validate tool outputs match expected format
//! - Ensure data compliance with formal specifications
//!
//! Supports JSON Schema draft-07.

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use jsonschema::JSONSchema;
use serde_json::json;

pub struct ValidatorTool;

impl ValidatorTool {
    pub fn new() -> Self {
        Self
    }

    fn validate_json(
        &self,
        data: &serde_json::Value,
        schema: &serde_json::Value,
        schema_name: Option<&str>,
    ) -> ToolResult {
        let validator = match JSONSchema::compile(schema) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Invalid schema: {}", e)),
                };
            }
        };

        let validation_result = validator.validate(data);

        match validation_result {
            Ok(()) => ToolResult {
                success: true,
                output: format!(
                    "Valid: data matches schema{}",
                    schema_name.map(|n| format!(" '{}'", n)).unwrap_or_default()
                ),
                error: None,
            },
            Err(errors) => {
                // Collect errors into a Vec to count them
                let error_vec: Vec<_> = errors.collect();
                let error_count = error_vec.len();
                let error_messages: Vec<String> = error_vec
                    .into_iter()
                    .map(|e| {
                        let path = e.instance_path.to_string();
                        if path.is_empty() {
                            format!("- {}", e)
                        } else {
                            format!("- {}: {}", path, e)
                        }
                    })
                    .collect();

                ToolResult {
                    success: false,
                    output: format!("{} validation error(s):", error_count),
                    error: Some(error_messages.join("\n")),
                }
            }
        }
    }
}

impl Default for ValidatorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ValidatorTool {
    fn name(&self) -> &str {
        "validator"
    }

    fn description(&self) -> &str {
        "Validate JSON data against a JSON Schema. \
         Use to verify that data (from HTTP responses, file reads, or other tools) \
         matches an expected structure. \
         Supports JSON Schema draft-07. \
         Returns success=true if data is valid, success=false with error details if invalid."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "any",
                    "description": "JSON data to validate (object, array, string, number, etc.)"
                },
                "schema": {
                    "type": "object",
                    "description": "JSON Schema to validate against (draft-07). Example: {\"type\": \"object\", \"properties\": {\"name\": {\"type\": \"string\"}}}"
                },
                "schema_name": {
                    "type": "string",
                    "description": "Optional name for the schema (used in success message for clarity)"
                }
            },
            "required": ["data", "schema"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let data = match args.get("data") {
            Some(d) => d.clone(),
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'data' parameter".to_string()),
                });
            }
        };

        let schema = match args.get("schema") {
            Some(s) if s.is_object() => s.clone(),
            Some(_) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("'schema' must be a JSON object (schema)".to_string()),
                });
            }
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'schema' parameter".to_string()),
                });
            }
        };

        let schema_name = args.get("schema_name").and_then(|v| v.as_str());

        Ok(self.validate_json(&data, &schema, schema_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tool() -> ValidatorTool {
        ValidatorTool::new()
    }

    #[tokio::test]
    async fn valid_object() {
        let result = test_tool()
            .execute(json!({
                "data": {"name": "test", "value": 42},
                "schema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "value": {"type": "number"}
                    },
                    "required": ["name", "value"]
                }
            }))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
        assert!(result.output.contains("Valid"));
    }

    #[tokio::test]
    async fn valid_array() {
        let result = test_tool()
            .execute(json!({
                "data": [1, 2, 3, 4, 5],
                "schema": {
                    "type": "array",
                    "items": {"type": "number"}
                }
            }))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
    }

    #[tokio::test]
    async fn invalid_type() {
        let result = test_tool()
            .execute(json!({
                "data": {"name": "test", "value": "not a number"},
                "schema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "value": {"type": "number"}
                    }
                }
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("number"));
    }

    #[tokio::test]
    async fn missing_required_field() {
        let result = test_tool()
            .execute(json!({
                "data": {"name": "test"},
                "schema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "value": {"type": "number"}
                    },
                    "required": ["name", "value"]
                }
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("value"));
    }

    #[tokio::test]
    async fn invalid_schema() {
        let result = test_tool()
            .execute(json!({
                "data": {"name": "test"},
                "schema": {"type": "invalid_type"}
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid schema"));
    }

    #[tokio::test]
    async fn missing_data() {
        let result = test_tool()
            .execute(json!({
                "schema": {"type": "object"}
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Missing 'data'"));
    }

    #[tokio::test]
    async fn missing_schema() {
        let result = test_tool()
            .execute(json!({
                "data": {"name": "test"}
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Missing 'schema'"));
    }

    #[tokio::test]
    async fn with_schema_name() {
        let result = test_tool()
            .execute(json!({
                "data": {"name": "test"},
                "schema": {"type": "object"},
                "schema_name": "UserObject"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("UserObject"));
    }

    #[tokio::test]
    async fn nested_object_validation() {
        let result = test_tool()
            .execute(json!({
                "data": {
                    "user": {
                        "name": "John",
                        "address": {
                            "city": "NYC"
                        }
                    }
                },
                "schema": {
                    "type": "object",
                    "properties": {
                        "user": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "address": {
                                    "type": "object",
                                    "properties": {
                                        "city": {"type": "string"}
                                    }
                                }
                            }
                        }
                    }
                }
            }))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
    }

    #[tokio::test]
    async fn array_with_min_items() {
        let result = test_tool()
            .execute(json!({
                "data": [1, 2],
                "schema": {
                    "type": "array",
                    "minItems": 3
                }
            }))
            .await
            .unwrap();
        assert!(!result.success);
        // Error message may contain "item" or path, just verify it's invalid
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn string_pattern_validation() {
        let result = test_tool()
            .execute(json!({
                "data": "user@domain.com",
                "schema": {
                    "type": "string",
                    "pattern": "^[^@]+@[^@]+$"
                }
            }))
            .await
            .unwrap();
        assert!(result.success, "error: {:?}", result.error);
    }

    #[tokio::test]
    async fn number_range_validation() {
        let result = test_tool()
            .execute(json!({
                "data": 150,
                "schema": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100
                }
            }))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("100"));
    }
}
