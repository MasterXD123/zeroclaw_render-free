//! Integration tests for tools working together.
//!
//! Tests the interaction between multiple tools to verify they work
//! correctly as a system.

use zeroclaw::tools::{CodeRunnerTool, ToolResult, ValidatorTool};
use zeroclaw::tools::CalculatorTool;

use serde_json::json;

/// Test that code_runner + validator work together as a pipeline.
/// The agent generates JS to process data, then validates the result.
#[tokio::test]
async fn tools_code_runner_and_validator_pipeline() {
    let code_runner = CodeRunnerTool::new();
    let validator = ValidatorTool::new();

    // Step 1: Code runner transforms data (filter repos with >100 stars)
    let code_result = code_runner
        .execute(json!({
            "code": "input.filter(x => x.stars > 100).map(x => ({name: x.name, stars: x.stars}))",
            "data": [
                {"name": "repo1", "stars": 50},
                {"name": "repo2", "stars": 200},
                {"name": "repo3", "stars": 150}
            ]
        }))
        .await
        .unwrap();

    assert!(code_result.success, "code_runner failed: {:?}", code_result.error);
    assert!(code_result.output.contains("repo2"), "Expected repo2 in output: {}", code_result.output);

    // Step 2: Parse the output and validate with schema
    let validated_data: serde_json::Value = serde_json::from_str(&code_result.output)
        .expect("code_runner should return valid JSON");

    let schema = json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "stars": {"type": "number"}
            },
            "required": ["name", "stars"]
        }
    });

    let validation_result = validator
        .execute(json!({
            "data": validated_data,
            "schema": schema,
            "schema_name": "RepoArray"
        }))
        .await
        .unwrap();

    assert!(validation_result.success, "validation failed: {:?}", validation_result.error);
    assert!(validation_result.output.contains("RepoArray"));
}

/// Test calculator + code_runner: compute then process result
#[tokio::test]
async fn tools_calculator_and_code_runner() {
    let calculator = CalculatorTool::new();
    let code_runner = CodeRunnerTool::new();

    // Calculator evaluates expression
    let calc_result = calculator
        .execute(json!({"expression": "2 + 2"}))
        .await
        .unwrap();

    assert!(calc_result.success, "calculator failed: {:?}", calc_result.error);
    assert!(calc_result.output.contains("4"));

    // Code runner processes the number
    let code_result = code_runner
        .execute(json!({
            "code": "input * 10",
            "data": 4
        }))
        .await
        .unwrap();

    assert!(code_result.success, "code_runner failed: {:?}", code_result.error);
    assert!(code_result.output.contains("40"));
}

/// Test code_runner output validation with complex schema
#[tokio::test]
async fn tools_code_runner_validates_complex_output() {
    let code_runner = CodeRunnerTool::new();
    let validator = ValidatorTool::new();

    // Code runner generates structured data
    let code_result = code_runner
        .execute(json!({
            "code": "({\"users\": [{\"id\": 1, \"name\": \"Alice\", \"active\": true}, {\"id\": 2, \"name\": \"Bob\", \"active\": false}], \"count\": 2})"
        }))
        .await
        .unwrap();

    assert!(code_result.success, "code_runner failed: {:?}", code_result.error);

    let data: serde_json::Value = serde_json::from_str(&code_result.output)
        .expect("Should be valid JSON");

    let schema = json!({
        "type": "object",
        "properties": {
            "users": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "number"},
                        "name": {"type": "string"},
                        "active": {"type": "boolean"}
                    },
                    "required": ["id", "name", "active"]
                }
            },
            "count": {"type": "number"}
        },
        "required": ["users", "count"]
    });

    let validation_result = validator
        .execute(json!({
            "data": data,
            "schema": schema
        }))
        .await
        .unwrap();

    assert!(validation_result.success, "validation failed: {:?}", validation_result.error);
}

/// Test that invalid code output fails validation
#[tokio::test]
async fn tools_code_runner_invalid_output_fails_validation() {
    let code_runner = CodeRunnerTool::new();
    let validator = ValidatorTool::new();

    // Code runner produces valid JSON but wrong schema
    let code_result = code_runner
        .execute(json!({
            "code": "({name: 'test', value: 'not a number'})"
        }))
        .await
        .unwrap();

    assert!(code_result.success, "code_runner should succeed");

    let data: serde_json::Value = serde_json::from_str(&code_result.output)
        .expect("Should be valid JSON");

    // Schema expects value to be a number
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "value": {"type": "number"}
        },
        "required": ["name", "value"]
    });

    let validation_result = validator
        .execute(json!({
            "data": data,
            "schema": schema
        }))
        .await
        .unwrap();

    // Should fail: value is string not number
    assert!(!validation_result.success, "validation should fail for wrong type");
    assert!(validation_result.error.is_some());
}

/// Test code_runner with regex pattern validation
#[tokio::test]
async fn tools_code_runner_and_validator_with_regex() {
    let code_runner = CodeRunnerTool::new();
    let validator = ValidatorTool::new();

    // Extract emails using regex
    let code_result = code_runner
        .execute(json!({
            "code": "'user1@test.com and user2@example.org'.match(/[\\w.-]+@[\\w.-]+\\.\\w+/g)"
        }))
        .await
        .unwrap();

    assert!(code_result.success);

    let data: serde_json::Value = serde_json::from_str(&code_result.output)
        .expect("Should be valid JSON");

    let schema = json!({
        "type": "array",
        "items": {
            "type": "string",
            "pattern": "^[^@]+@[^@]+$"
        }
    });

    let validation_result = validator
        .execute(json!({
            "data": data,
            "schema": schema
        }))
        .await
        .unwrap();

    assert!(validation_result.success, "emails should match email pattern");
}
