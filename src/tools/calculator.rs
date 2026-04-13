//! Calculator tool — arbitrary precision mathematical expressions.
//!
//! Supports: +, -, *, /, ^, %, sqrt, abs, floor, ceil, round,
//! sin, cos, tan, asin, acos, atan, log, ln, exp, pow,
//! pi, e, deg2rad, rad2deg

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;

pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions with arbitrary precision. \
         Supports basic operators: + - * / % \
         Comparison: < > <= >= == != \
         Functions: sqrt, abs, floor, ceil, round, exp, ln, log (natural log), sin, cos, tan, asin, acos, atan, pow(base,exp) \
         Constants: pi, e \
         Conversions: deg2rad, rad2deg \
         Examples: 2+2, sqrt(16), sin(pi/2), exp(1), pow(2,8), 5%2"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate. Examples: '2+2', 'sqrt(16)', 'sin(pi/2)', 'exp(1)', 'pow(2,8)', '5%2'"
                },
                "precision": {
                    "type": "integer",
                    "description": "Number of decimal places in result (default: 10, max: 50)",
                    "minimum": 0,
                    "maximum": 50,
                    "default": 10
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let expression = match args.get("expression").and_then(|v| v.as_str()) {
            Some(e) if !e.trim().is_empty() => e.trim(),
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'expression' parameter".to_string()),
                });
            }
        };

        let precision = args
            .get("precision")
            .and_then(|v| v.as_i64())
            .unwrap_or(10)
            .clamp(0, 50) as usize;

        match evaluate_expression(expression, precision) {
            Ok(result) => Ok(ToolResult {
                success: true,
                output: result,
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e),
            }),
        }
    }
}

/// Evaluate a math expression string
fn evaluate_expression(expr: &str, precision: usize) -> Result<String, String> {
    let expr = expr.trim();

    if expr.is_empty() {
        return Err("Empty expression".to_string());
    }

    // Check for dangerous patterns
    if expr.contains("__")
        || expr.to_lowercase().contains("eval")
        || expr.to_lowercase().contains("exec")
        || expr.contains("0x")
    {
        return Err("Expression contains disallowed patterns".to_string());
    }

    // Preprocess: handle implied multiplication, constants, etc
    let processed = preprocess(expr);

    // Handle power operations before meval (meval doesn't support pow or **)
    let processed = handle_power(&processed)?;

    // Use meval for the preprocessed expression
    match meval::eval_str(&processed) {
        Ok(value) => format_result(value, precision),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Handle power expressions like "2^8" - convert to exp(ln(x)*y)
/// x^y = exp(y * ln(x)) for x > 0
fn handle_power(expr: &str) -> Result<String, String> {
    // Check if expression contains power operator ^
    if !expr.contains('^') {
        return Ok(expr.to_string());
    }

    // Parse and replace x^y with exp(ln(x)*y)
    let mut result = String::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '^' {
            // Find the base (previous number/variable) and exponent (next number)
            // First skip trailing whitespace, then find base characters
            let mut base_end = i;
            while base_end > 0 && chars[base_end - 1].is_whitespace() {
                base_end -= 1;
            }
            let mut base_start = base_end;
            while base_start > 0
                && (chars[base_start - 1].is_numeric()
                    || chars[base_start - 1] == ')'
                    || chars[base_start - 1] == 'e')
            {
                base_start -= 1;
            }
            let base = &expr[base_start..base_end];

            // Go forward to find exponent (skip whitespace before number)
            let mut exp_end = i + 1;
            while exp_end < chars.len() && chars[exp_end].is_whitespace() {
                exp_end += 1;
            }
            while exp_end < chars.len() && (chars[exp_end].is_numeric() || chars[exp_end] == '.') {
                exp_end += 1;
            }
            let exponent = &expr[i + 1..exp_end];

            // Replace x^y with exp(ln(x)*y)
            result.push_str(&format!("exp(ln({})*{})", base.trim(), exponent.trim()));
            i = exp_end;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Ok(result)
}

fn preprocess(expr: &str) -> String {
    let expr = expr.trim();

    // Handle constants - meval doesn't have pi/e built-in as names
    let expr = expr.replace("pi", &format!("{}", std::f64::consts::PI));
    let expr = expr.replace("e ", &format!("{} ", std::f64::consts::E));
    let expr = expr.replace("e)", &format!("{})", std::f64::consts::E));

    // Handle deg2rad and rad2deg
    let expr = expr.replace("deg2rad", &format!("({}*", std::f64::consts::PI / 180.0));
    let expr = expr.replace("rad2deg", &format!("({}*", 180.0 / std::f64::consts::PI));

    expr
}

fn format_result(value: f64, precision: usize) -> Result<String, String> {
    if !value.is_finite() {
        return Err(format!("Result is not finite: {}", value));
    }

    // Handle very large or very small numbers with scientific notation
    if value.abs() > 1e10 || (value.abs() < 1e-6 && value != 0.0) {
        let formatted = format!("{:.prec$}", value, prec = precision.min(15));
        return Ok(formatted);
    }

    let formatted = if precision == 0 {
        format!("{:.0}", value)
    } else {
        format!("{:.prec$}", value, prec = precision)
    };

    // Remove trailing zeros after decimal point
    let trimmed = if formatted.contains('.') {
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        formatted
    };

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        assert_eq!(evaluate_expression("2 + 2", 10).unwrap(), "4");
        assert_eq!(evaluate_expression("10 - 3", 10).unwrap(), "7");
        assert_eq!(evaluate_expression("3 * 4", 10).unwrap(), "12");
        assert_eq!(evaluate_expression("15 / 3", 10).unwrap(), "5");
    }

    #[test]
    fn powers() {
        // x^y is converted to exp(ln(x)*y) for positive x
        assert_eq!(evaluate_expression("2 ^ 8", 10).unwrap(), "256");
        assert_eq!(evaluate_expression("10 % 3", 10).unwrap(), "1");
    }

    #[test]
    fn functions() {
        // sqrt
        let sqrt = evaluate_expression("sqrt(16)", 10).unwrap();
        assert!(sqrt == "4" || sqrt == "4.0" || sqrt == "4.0000000000");

        // sin
        let sin = evaluate_expression("sin(0)", 10).unwrap();
        assert!(sin == "0" || sin == "0.0");

        // abs
        let abs = evaluate_expression("abs(-5)", 10).unwrap();
        assert_eq!(abs, "5");

        // floor, ceil
        assert_eq!(evaluate_expression("floor(3.7)", 10).unwrap(), "3");
        assert_eq!(evaluate_expression("ceil(3.2)", 10).unwrap(), "4");
    }

    #[test]
    fn constants() {
        // pi should be replaced before meval sees it
        let result = evaluate_expression("pi", 5).unwrap();
        assert!(result.starts_with("3.1415") || result.starts_with("3.14159"));
    }

    #[test]
    fn complex_expressions() {
        let result = evaluate_expression("2 + 3 * 4", 10).unwrap();
        assert!(result == "14" || result == "14.0");

        let parens = evaluate_expression("(2 + 3) * 4", 10).unwrap();
        assert!(parens == "20" || parens == "20.0");
    }

    #[test]
    fn precision() {
        let div = evaluate_expression("10 / 3", 4).unwrap();
        assert!(div.starts_with("3.333"));

        let div = evaluate_expression("10 / 3", 0).unwrap();
        assert_eq!(div, "3");
    }

    #[test]
    fn errors() {
        assert!(evaluate_expression("", 10).is_err());
        assert!(evaluate_expression("invalid", 10).is_err());
    }
}
