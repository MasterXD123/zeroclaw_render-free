//! Safe Executor - Dry-run, validate, execute, rollback
//!
//! Provides safe execution modes for tool operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Simulate execution without changes
    DryRun,
    /// Verify parameters and permissions only
    Validate,
    /// Execute the operation
    Execute,
    /// Rollback a previous execution
    Rollback,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::DryRun
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::DryRun => write!(f, "dry-run"),
            ExecutionMode::Validate => write!(f, "validate"),
            ExecutionMode::Execute => write!(f, "execute"),
            ExecutionMode::Rollback => write!(f, "rollback"),
        }
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub mode: ExecutionMode,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub dry_run_placeholder: Option<String>,
}

impl ExecutionResult {
    pub fn dry_run(msg: &str) -> Self {
        Self {
            mode: ExecutionMode::DryRun,
            success: true,
            output: String::new(),
            error: None,
            dry_run_placeholder: Some(msg.to_string()),
        }
    }

    pub fn validated() -> Self {
        Self {
            mode: ExecutionMode::Validate,
            success: true,
            output: String::new(),
            error: None,
            dry_run_placeholder: None,
        }
    }

    pub fn executed(output: &str) -> Self {
        Self {
            mode: ExecutionMode::Execute,
            success: true,
            output: output.to_string(),
            error: None,
            dry_run_placeholder: None,
        }
    }

    pub fn rolled_back() -> Self {
        Self {
            mode: ExecutionMode::Rollback,
            success: true,
            output: String::new(),
            error: None,
            dry_run_placeholder: None,
        }
    }

    pub fn failed(error: &str) -> Self {
        Self {
            mode: ExecutionMode::Execute,
            success: false,
            output: String::new(),
            error: Some(error.to_string()),
            dry_run_placeholder: None,
        }
    }
}

/// Safe Executor with execution modes
pub struct SafeExecutor {
    mode: ExecutionMode,
    verify_timeout_ms: u64,
}

impl SafeExecutor {
    /// Create new executor with mode
    pub fn new(mode: ExecutionMode) -> Self {
        Self {
            mode,
            verify_timeout_ms: 5000,
        }
    }

    /// Create with default mode (dry-run)
    pub fn default() -> Self {
        Self::new(ExecutionMode::DryRun)
    }

    /// Set execution mode
    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set verify timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.verify_timeout_ms = ms;
        self
    }

    /// Get current mode
    pub fn mode(&self) -> ExecutionMode {
        self.mode
    }

    /// Execute with given mode
    pub async fn execute<T: Serialize>(&self, tool: &str, input: &T) -> ExecutionResult {
        let input_str = serde_json::to_string(input).unwrap_or_default();

        match self.mode {
            ExecutionMode::DryRun => {
                tracing::info!("[DRY-RUN] Would execute: {} with {}", tool, input_str);
                ExecutionResult::dry_run(&format!("Would execute {} with {}", tool, input_str))
            }
            ExecutionMode::Validate => {
                tracing::info!("[VALIDATE] Checking: {} with {}", tool, input_str);
                // In validate mode, just verify the input is well-formed
                if input_str.contains("error") || input_str.contains("null") {
                    ExecutionResult::failed("Validation failed: malformed input")
                } else {
                    ExecutionResult::validated()
                }
            }
            ExecutionMode::Execute => {
                tracing::info!("[EXECUTE] Running: {} with {}", tool, input_str);
                // Real execution would happen here
                ExecutionResult::executed(&format!("Executed {} successfully", tool))
            }
            ExecutionMode::Rollback => {
                tracing::info!("[ROLLBACK] Would rollback: {}", tool);
                ExecutionResult::rolled_back()
            }
        }
    }

    /// Check if execution is allowed in current mode
    pub fn can_execute(&self) -> bool {
        matches!(self.mode, ExecutionMode::Execute)
    }

    /// Check if dry-run is active
    pub fn is_dry_run(&self) -> bool {
        matches!(self.mode, ExecutionMode::DryRun)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dry_run() {
        let executor = SafeExecutor::new(ExecutionMode::DryRun);
        let result = executor.execute("test_tool", &serde_json::json!({"key": "value"})).await;
        assert!(result.success);
        assert!(result.dry_run_placeholder.is_some());
    }

    #[tokio::test]
    async fn test_validate() {
        let executor = SafeExecutor::new(ExecutionMode::Validate);
        let result = executor.execute("test_tool", &serde_json::json!({"key": "value"})).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_execute() {
        let executor = SafeExecutor::new(ExecutionMode::Execute);
        let result = executor.execute("test_tool", &serde_json::json!({"key": "value"})).await;
        assert!(result.success);
        assert!(result.output.contains("test_tool"));
    }
}