//! Reasoning Loop - Iterative validation with JSON Schema
//!
//! Provides structured reasoning with validation at each iteration.

use serde_json::Value;

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Reasoning step
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub iteration: usize,
    pub input: Value,
    pub output: Option<Value>,
    pub validation: ValidationResult,
}

/// Configuration for reasoning loop
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Maximum iterations before giving up
    pub max_iterations: usize,
    /// Require strict validation
    pub strict: bool,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            strict: false,
        }
    }
}

/// Reasoning Loop with JSON Schema validation
pub struct ReasoningLoop {
    config: ReasoningConfig,
    steps: Vec<ReasoningStep>,
}

impl ReasoningLoop {
    /// Create new reasoning loop with config
    pub fn new(config: ReasoningConfig) -> Self {
        Self {
            config,
            steps: Vec::new(),
        }
    }

    /// Create with default config (10 iterations, non-strict)
    pub fn default() -> Self {
        Self::new(ReasoningConfig::default())
    }

    /// Validate output against expected structure
    pub fn validate_output(&self, output: &Value, schema: &Value) -> ValidationResult {
        // Simple JSON Schema validation (key presence check)
        let mut errors = Vec::new();

        if let Some(obj) = output.as_object() {
            if let Some(required) = schema.get("required") {
                if let Some(arr) = required.as_array() {
                    for req in arr {
                        if let Some(key) = req.as_str() {
                            if !obj.contains_key(key) {
                                errors.push(format!("Missing required field: {}", key));
                            }
                        }
                    }
                }
            }
        } else if let Some(arr) = output.as_array() {
            // For array outputs, just check it's not empty if minItems specified
            if let Some(min) = schema.get("minItems") {
                if let Some(min_val) = min.as_i64() {
                    if arr.len() as i64 < min_val {
                        errors.push(format!("Array too short: expected at least {}", min_val));
                    }
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    /// Run one iteration of reasoning
    pub fn step<F>(&mut self, input: Value, schema: &Value, mut transform: F) -> ValidationResult
    where
        F: FnMut(Value) -> Value,
    {
        let iteration = self.steps.len();
        let output = transform(input.clone());

        let validation = self.validate_output(&output, schema);

        self.steps.push(ReasoningStep {
            iteration,
            input,
            output: Some(output),
            validation: validation.clone(),
        });

        validation
    }

    /// Run loop until valid or max iterations
    pub fn run<F>(&mut self, input: Value, schema: &Value, mut transform: F) -> Result<Value, String>
    where
        F: FnMut(Value) -> Value,
    {
        for _ in 0..self.config.max_iterations {
            let iteration = self.steps.len();
            let output = transform(input.clone());
            let validation = self.validate_output(&output, schema);

            self.steps.push(ReasoningStep {
                iteration,
                input,
                output: Some(output.clone()),
                validation: validation.clone(),
            });

            if validation.is_valid {
                return Ok(output);
            }

            input = output; // Continue with transformed output
        }

        Err(format!(
            "Max iterations ({}) reached without valid output",
            self.config.max_iterations
        ))
    }

    /// Get all reasoning steps
    pub fn steps(&self) -> &[ReasoningStep] {
        &self.steps
    }

    /// Get iteration count
    pub fn iterations(&self) -> usize {
        self.steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_success() {
        let rl = ReasoningLoop::default();
        let output = json!({"name": "test", "value": 42});
        let schema = json!({
            "type": "object",
            "required": ["name", "value"]
        });

        let result = rl.validate_output(&output, &schema);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validation_missing_field() {
        let rl = ReasoningLoop::default();
        let output = json!({"name": "test"});
        let schema = json!({
            "type": "object",
            "required": ["name", "value"]
        });

        let result = rl.validate_output(&output, &schema);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
}