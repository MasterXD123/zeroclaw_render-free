//! Decision tool — conditional branching and routing.
//!
//! Evaluates conditions and routes execution based on comparisons,
//! logical operators, and branching logic.

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

pub struct DecisionTool;

impl DecisionTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecisionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

impl Value {
    fn from_json(v: &serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => {
                Value::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            _ => Value::String(v.to_string()),
        }
    }

    fn compare(&self, op: &str, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => {
                match op {
                    "==" => Ok(matches!((self, other), (Value::Null, Value::Null))),
                    "!=" => Ok(!matches!((self, other), (Value::Null, Value::Null))),
                    "is_null" => Ok(matches!(self, Value::Null)),
                    "is_not_null" => Ok(!matches!(self, Value::Null)),
                    _ => Err(format!("Cannot compare null with '{}'", op)),
                }
            }
            (Value::Number(a), Value::Number(b)) => {
                match op {
                    "==" => Ok((a - b).abs() < 1e-10),
                    "!=" => Ok((a - b).abs() >= 1e-10),
                    "<" => Ok(a < b),
                    "<=" => Ok(a <= b),
                    ">" => Ok(a > b),
                    ">=" => Ok(a >= b),
                    _ => Err(format!("Unknown operator '{}' for numbers", op)),
                }
            }
            (Value::String(a), Value::String(b)) => {
                match op {
                    "==" => Ok(a == b),
                    "!=" => Ok(a != b),
                    "<" => Ok(a < b),
                    "<=" => Ok(a <= b),
                    ">" => Ok(a > b),
                    ">=" => Ok(a >= b),
                    "contains" => Ok(a.contains(b)),
                    "starts_with" => Ok(a.starts_with(b)),
                    "ends_with" => Ok(a.ends_with(b)),
                    "matches" => {
                        match regex::Regex::new(b) {
                            Ok(re) => Ok(re.is_match(a)),
                            Err(e) => Err(format!("Invalid regex: {}", e)),
                        }
                    }
                    "in" => Ok(b.contains(a)),
                    _ => Err(format!("Unknown operator '{}' for strings", op)),
                }
            }
            (Value::Bool(a), Value::Bool(b)) => {
                match op {
                    "==" => Ok(a == b),
                    "!=" => Ok(a != b),
                    "and" => Ok(*a && *b),
                    "or" => Ok(*a || *b),
                    _ => Err(format!("Operator '{}' not supported for booleans", op)),
                }
            }
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                // Try numeric comparison if string is numeric
                if let (Some(num), n_val) = (s.parse::<f64>().ok(), *n) {
                    match op {
                        "==" => Ok((n_val - num).abs() < 1e-10),
                        "!=" => Ok((n_val - num).abs() >= 1e-10),
                        "<" => Ok(n_val < num),
                        "<=" => Ok(n_val <= num),
                        ">" => Ok(n_val > num),
                        ">=" => Ok(n_val >= num),
                        _ => Err(format!("Operator '{}' not supported for string-number", op)),
                    }
                } else {
                    Err(format!("Cannot compare {} with {}", self.type_name(), other.type_name()))
                }
            }
            _ => Err(format!("Cannot compare {} with {}", self.type_name(), other.type_name())),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
        }
    }
}

fn eval_condition(cond: &serde_json::Value) -> Result<bool, String> {
    // Simple condition: { "left": <value>, "op": <operator>, "right": <value> }
    // Compound: { "and": [...conditions] } or { "or": [...conditions] }
    // Negation: { "not": <condition> }
    // Direct value: <boolean>

    if let Some(obj) = cond.as_object() {
        if let Some(left) = obj.get("left") {
            let left_val = Value::from_json(left);
            let op = obj.get("op")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'op' in condition")?;
            let right = obj.get("right")
                .ok_or("Missing 'right' in condition")?;
            let right_val = Value::from_json(right);
            return left_val.compare(op, &right_val);
        }

        if let Some(conditions) = obj.get("and") {
            let arr = conditions.as_array()
                .ok_or("'and' must be an array")?;
            for c in arr {
                if !eval_condition(c)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        if let Some(conditions) = obj.get("or") {
            let arr = conditions.as_array()
                .ok_or("'or' must be an array")?;
            for c in arr {
                if eval_condition(c)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        if let Some(c) = obj.get("not") {
            return Ok(!eval_condition(c)?);
        }

        if let Some(v) = obj.get("is_null") {
            return Ok(Value::from_json(v).compare("is_null", &Value::Null).is_ok_and(|r| r));
        }

        if let Some(v) = obj.get("is_not_null") {
            return Ok(Value::from_json(v).compare("is_not_null", &Value::Null).is_ok_and(|r| r));
        }
    }

    if let Some(b) = cond.as_bool() {
        return Ok(b);
    }

    Err(format!("Invalid condition format: {}", cond))
}

fn eval_branch(branch: &serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(branch.clone())
}

#[async_trait]
impl Tool for DecisionTool {
    fn name(&self) -> &str {
        "decision"
    }

    fn description(&self) -> &str {
        "Evaluate conditions and route execution based on branching logic. \
         Supports comparisons (==, !=, <, <=, >, >=), string operations \
         (contains, starts_with, ends_with, matches, in), logical operators \
         (and, or, not), and branching (if/elif/else). Use for conditional \
         routing, validation, and dynamic decision-making.\n\n\
         Condition structure:\n\
         - Simple: { \"left\": value, \"op\": \"operator\", \"right\": value }\n\
         - Compound: { \"and\": [cond1, cond2] } or { \"or\": [cond1, cond2] }\n\
         - Negation: { \"not\": condition }\n\
         - Direct bool: true/false\n\n\
         Operators:\n\
         - Comparison: ==, !=, <, <=, >, >=\n\
         - String: contains, starts_with, ends_with, matches (regex), in (substring)\n\
         - Null: is_null, is_not_null"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "condition": {
                    "oneOf": [
                        {"type": "boolean"},
                        {
                            "type": "object",
                            "description": "Condition object with left/op/right structure"
                        }
                    ],
                    "description": "Condition to evaluate. Can be a boolean or a condition object."
                },
                "if_true": {
                    "type": "object",
                    "description": "Value or object to return if condition is true"
                },
                "if_false": {
                    "type": "object",
                    "description": "Value or object to return if condition is false"
                },
                "branches": {
                    "type": "array",
                    "description": "Array of { condition, value } pairs for if/elif/else branching. First matching condition wins.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "condition": {"description": "Condition to test"},
                            "value": {"description": "Value to return if condition is true"}
                        }
                    }
                },
                "default": {
                    "description": "Default value if no conditions match (for branches)"
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // Evaluate a single condition
        let condition_match = |cond: &serde_json::Value| -> Result<bool, String> {
            eval_condition(cond)
        };

        // Handle if_true/if_false pattern
        if let Some(cond) = args.get("condition") {
            match condition_match(cond) {
                Ok(true) => {
                    if let Some(if_true) = args.get("if_true") {
                        return Ok(ToolResult {
                            success: true,
                            output: eval_branch(if_true).unwrap().to_string(),
                            error: None,
                        });
                    }
                }
                Ok(false) => {
                    if let Some(if_false) = args.get("if_false") {
                        return Ok(ToolResult {
                            success: true,
                            output: eval_branch(if_false).unwrap().to_string(),
                            error: None,
                        });
                    }
                }
                Err(e) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    });
                }
            }
        }

        // Handle branches pattern
        if let Some(branches) = args.get("branches").and_then(|v| v.as_array()) {
            for branch in branches {
                if let (Some(cond), Some(value)) = (branch.get("condition"), branch.get("value")) {
                    match condition_match(cond) {
                        Ok(true) => {
                            return Ok(ToolResult {
                                success: true,
                                output: eval_branch(value).unwrap().to_string(),
                                error: None,
                            });
                        }
                        Ok(false) => continue,
                        Err(e) => {
                            return Ok(ToolResult {
                                success: false,
                                output: String::new(),
                                error: Some(e),
                            });
                        }
                    }
                }
            }

            // Default branch
            if let Some(default) = args.get("default") {
                return Ok(ToolResult {
                    success: true,
                    output: eval_branch(default).unwrap().to_string(),
                    error: None,
                });
            }
        }

        // Just evaluate condition and return result
        if let Some(cond) = args.get("condition") {
            match condition_match(cond) {
                Ok(result) => {
                    return Ok(ToolResult {
                        success: true,
                        output: json!({ "result": result }).to_string(),
                        error: None,
                    });
                }
                Err(e) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    });
                }
            }
        }

        Ok(ToolResult {
            success: false,
            output: String::new(),
            error: Some("No condition or branches provided".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_tool() -> DecisionTool {
        DecisionTool::new()
    }

    #[tokio::test]
    async fn simple_equality() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": { "left": 5, "op": ">", "right": 3 },
                "if_true": { "route": "A" },
                "if_false": { "route": "B" }
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("A"));
    }

    #[tokio::test]
    async fn string_contains() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": { "left": "hello world", "op": "contains", "right": "world" },
                "if_true": "matched",
                "if_false": "not matched"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("matched"));
    }

    #[tokio::test]
    async fn logical_and() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": {
                    "and": [
                        { "left": 10, "op": ">", "right": 5 },
                        { "left": "test", "op": "contains", "right": "es" }
                    ]
                },
                "if_true": "both true",
                "if_false": "failed"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("both true"));
    }

    #[tokio::test]
    async fn logical_or() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": {
                    "or": [
                        { "left": 1, "op": "==", "right": 2 },
                        { "left": "hello", "op": "starts_with", "right": "he" }
                    ]
                },
                "if_true": "one true",
                "if_false": "both false"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("one true"));
    }

    #[tokio::test]
    async fn negation() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": { "not": { "left": 5, "op": ">", "right": 10 } },
                "if_true": "5 <= 10",
                "if_false": "impossible"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("5 <= 10"));
    }

    #[tokio::test]
    async fn branches_pattern() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "branches": [
                    { "condition": { "left": "high", "op": "==", "right": "high" }, "value": "priority 1" },
                    { "condition": { "left": "medium", "op": "==", "right": "medium" }, "value": "priority 2" }
                ],
                "default": "priority 3"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("priority 1"));
    }

    #[tokio::test]
    async fn branches_default() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "branches": [
                    { "condition": false, "value": "never" }
                ],
                "default": "default_value"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("default_value"));
    }

    #[tokio::test]
    async fn regex_matches() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": { "left": "file-123.txt", "op": "matches", "right": r"^\w+-\d+\.\w+$" },
                "if_true": "valid filename",
                "if_false": "invalid"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("valid"));
    }

    #[tokio::test]
    async fn null_handling() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": { "left": null, "op": "is_null", "right": null },
                "if_true": "is null",
                "if_false": "not null"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("is null"));
    }

    #[tokio::test]
    async fn in_operator() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "condition": { "left": "cat", "op": "in", "right": "catalog" },
                "if_true": "found",
                "if_false": "not found"
            }))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("found"));
    }
}
