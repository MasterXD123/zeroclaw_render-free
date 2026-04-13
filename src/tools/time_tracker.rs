//! Time tracker tool — measure duration of each task.
//!
//! Records start/end timestamps and computes elapsed time for tasks.
//! Useful for performance measurement and profiling.

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TimeTrackerTool {
    sessions: Mutex<HashMap<String, TaskSession>>,
}

#[derive(Clone)]
struct TaskSession {
    task_id: String,
    started_at: u64,
    labels: Vec<String>,
}

impl TimeTrackerTool {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn format_duration(ms: u64) -> String {
        let seconds = ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;

        if days > 0 {
            format!(
                "{}d {}h {}m {}s",
                days,
                hours % 24,
                minutes % 60,
                seconds % 60
            )
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds % 60)
        } else {
            format!("{}s", seconds)
        }
    }
}

impl Default for TimeTrackerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TimeTrackerTool {
    fn name(&self) -> &str {
        "time_tracker"
    }

    fn description(&self) -> &str {
        "Track time spent on tasks. Start a timer for a task_id, stop it to get \
         elapsed duration, or query active sessions. Useful for performance \
         profiling and measuring how long operations take.\n\n\
         Actions:\n\
         - start: begin tracking a task\n\
         - stop: end tracking and return duration\n\
         - active: list all currently running timers\n\
         - elapsed: return duration of a running or completed task without stopping"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["start", "stop", "active", "elapsed"],
                    "description": "Action to perform: start (begin timer), stop (end timer and return duration), active (list running timers), elapsed (get duration without stopping)"
                },
                "task_id": {
                    "type": "string",
                    "description": "Unique identifier for the task. Required for start, stop, and elapsed actions."
                },
                "labels": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional labels to attach to the task (e.g. ['performance', 'io'])."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let action = match args.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'action' parameter".to_string()),
                });
            }
        };

        match action {
            "start" => {
                let task_id = match args.get("task_id").and_then(|v| v.as_str()) {
                    Some(id) if !id.trim().is_empty() => id.trim().to_string(),
                    _ => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some("Missing 'task_id' for start action".to_string()),
                        });
                    }
                };

                let labels: Vec<String> = args
                    .get("labels")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                let now = Self::now();
                let mut sessions = self.sessions.lock().unwrap();

                if sessions.contains_key(&task_id) {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "Task '{}' already has an active timer. Stop it first.",
                            task_id
                        )),
                    });
                }

                sessions.insert(
                    task_id.clone(),
                    TaskSession {
                        task_id: task_id.clone(),
                        started_at: now,
                        labels,
                    },
                );

                Ok(ToolResult {
                    success: true,
                    output: format!("Timer started for task '{}' at {}", task_id, now),
                    error: None,
                })
            }

            "stop" => {
                let task_id = match args.get("task_id").and_then(|v| v.as_str()) {
                    Some(id) if !id.trim().is_empty() => id.trim().to_string(),
                    _ => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some("Missing 'task_id' for stop action".to_string()),
                        });
                    }
                };

                let now = Self::now();
                let mut sessions = self.sessions.lock().unwrap();

                let session = match sessions.remove(&task_id) {
                    Some(s) => s,
                    None => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("No active timer found for task '{}'", task_id)),
                        });
                    }
                };

                let elapsed_ms = now - session.started_at;
                let duration_str = Self::format_duration(elapsed_ms);

                Ok(ToolResult {
                    success: true,
                    output: json!({
                        "task_id": task_id,
                        "started_at": session.started_at,
                        "stopped_at": now,
                        "duration_ms": elapsed_ms,
                        "duration_human": duration_str,
                        "labels": session.labels
                    })
                    .to_string(),
                    error: None,
                })
            }

            "active" => {
                let sessions = self.sessions.lock().unwrap();
                let now = Self::now();

                if sessions.is_empty() {
                    return Ok(ToolResult {
                        success: true,
                        output: json!({
                            "active_tasks": [],
                            "count": 0
                        })
                        .to_string(),
                        error: None,
                    });
                }

                let active: Vec<serde_json::Value> = sessions
                    .values()
                    .map(|s| {
                        let elapsed_ms = now - s.started_at;
                        json!({
                            "task_id": s.task_id,
                            "started_at": s.started_at,
                            "elapsed_ms": elapsed_ms,
                            "elapsed_human": Self::format_duration(elapsed_ms),
                            "labels": s.labels
                        })
                    })
                    .collect();

                Ok(ToolResult {
                    success: true,
                    output: json!({
                        "active_tasks": active,
                        "count": active.len()
                    })
                    .to_string(),
                    error: None,
                })
            }

            "elapsed" => {
                let task_id = match args.get("task_id").and_then(|v| v.as_str()) {
                    Some(id) if !id.trim().is_empty() => id.trim().to_string(),
                    _ => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some("Missing 'task_id' for elapsed action".to_string()),
                        });
                    }
                };

                let sessions = self.sessions.lock().unwrap();
                let now = Self::now();

                match sessions.get(&task_id) {
                    Some(s) => {
                        let elapsed_ms = now - s.started_at;
                        Ok(ToolResult {
                            success: true,
                            output: json!({
                                "task_id": task_id,
                                "started_at": s.started_at,
                                "elapsed_ms": elapsed_ms,
                                "elapsed_human": Self::format_duration(elapsed_ms),
                                "labels": s.labels,
                                "running": true
                            })
                            .to_string(),
                            error: None,
                        })
                    }
                    None => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "No timer found for task '{}' (never started or already stopped)",
                            task_id
                        )),
                    }),
                }
            }

            _ => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Unknown action '{}'. Use: start, stop, active, or elapsed.",
                    action
                )),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_tool() -> TimeTrackerTool {
        TimeTrackerTool::new()
    }

    #[tokio::test]
    async fn start_and_stop() {
        let tool = new_tool();

        // Start
        let res = tool
            .execute(json!({"action": "start", "task_id": "test-task-1"}))
            .await
            .unwrap();
        assert!(res.success);

        // Stop
        let res = tool
            .execute(json!({"action": "stop", "task_id": "test-task-1"}))
            .await
            .unwrap();
        assert!(res.success);
        assert!(res.output.contains("duration_ms"));
        assert!(res.output.contains("duration_human"));
    }

    #[tokio::test]
    async fn start_with_labels() {
        let tool = new_tool();
        let res = tool
            .execute(json!({
                "action": "start",
                "task_id": "test-task-2",
                "labels": ["performance", "io"]
            }))
            .await
            .unwrap();
        assert!(res.success);
    }

    #[tokio::test]
    async fn stop_nonexistent() {
        let tool = new_tool();
        let res = tool
            .execute(json!({"action": "stop", "task_id": "does-not-exist"}))
            .await
            .unwrap();
        assert!(!res.success);
        assert!(res.error.is_some());
    }

    #[tokio::test]
    async fn active_empty() {
        let tool = new_tool();
        let res = tool.execute(json!({"action": "active"})).await.unwrap();
        assert!(res.success);
        assert!(res.output.contains("\"count\":0"));
    }

    #[tokio::test]
    async fn elapsed_without_start() {
        let tool = new_tool();
        let res = tool
            .execute(json!({"action": "elapsed", "task_id": "never-started"}))
            .await
            .unwrap();
        assert!(!res.success);
    }

    #[tokio::test]
    async fn duplicate_start_fails() {
        let tool = new_tool();
        tool.execute(json!({"action": "start", "task_id": "dup-task"}))
            .await
            .unwrap();

        let res = tool
            .execute(json!({"action": "start", "task_id": "dup-task"}))
            .await
            .unwrap();
        assert!(!res.success);
    }
}
