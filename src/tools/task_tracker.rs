//! Task tracker tool — create, update, list, and manage tasks for workflow execution.
//!
//! Used by SuperAgent to track progress of tasks in a plan.
//! Tasks have: id, description, status, dependencies, result, created_at, updated_at.

use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TaskTrackerTool {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub plan_id: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub task_type: TaskType,
    pub dependencies: Vec<String>,
    pub result: Option<String>,
    pub output: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Skipped => write!(f, "skipped"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P0 => write!(f, "p0"),
            Priority::P1 => write!(f, "p1"),
            Priority::P2 => write!(f, "p2"),
            Priority::P3 => write!(f, "p3"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Research,
    Action,
    Validation,
    Coordination,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::Research => write!(f, "research"),
            TaskType::Action => write!(f, "action"),
            TaskType::Validation => write!(f, "validation"),
            TaskType::Coordination => write!(f, "coordination"),
        }
    }
}

impl Task {
    pub fn new(
        id: String,
        plan_id: String,
        description: String,
        priority: Priority,
        task_type: TaskType,
        dependencies: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            plan_id,
            description,
            status: TaskStatus::Pending,
            priority,
            task_type,
            dependencies,
            result: None,
            output: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        }
    }
}

impl TaskTrackerTool {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn create_task(&self, args: &serde_json::Value) -> Result<Task, String> {
        let task_id = args
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'task_id'")?
            .to_string();

        let plan_id = args
            .get("plan_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let priority = args
            .get("priority")
            .and_then(|v| v.as_str())
            .map(|p| match p.to_lowercase().as_str() {
                "p0" => Priority::P0,
                "p1" => Priority::P1,
                "p2" => Priority::P2,
                "p3" => Priority::P3,
                _ => Priority::P2,
            })
            .unwrap_or(Priority::P2);

        let task_type = args
            .get("task_type")
            .and_then(|v| v.as_str())
            .map(|t| match t.to_lowercase().as_str() {
                "research" => TaskType::Research,
                "action" => TaskType::Action,
                "validation" => TaskType::Validation,
                "coordination" => TaskType::Coordination,
                _ => TaskType::Action,
            })
            .unwrap_or(TaskType::Action);

        let dependencies = args
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let full_id = if task_id.contains('/') {
            task_id.clone()
        } else {
            format!("{}/{}", plan_id, task_id)
        };

        let task = Task::new(full_id.clone(), plan_id, description, priority, task_type, dependencies);

        let mut tasks = self.tasks.lock().await;
        tasks.insert(full_id.clone(), task);

        Ok(tasks.get(&full_id).cloned().unwrap())
    }

    async fn update_task(&self, args: &serde_json::Value) -> Result<Task, String> {
        let task_id = args
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'task_id'")?
            .to_string();

        // Normalize task_id: if it doesn't contain '/', prepend "default/"
        let full_id = if task_id.contains('/') {
            task_id.clone()
        } else {
            format!("default/{}", task_id)
        };

        let mut tasks = self.tasks.lock().await;

        let task = tasks
            .get_mut(&full_id)
            .ok_or(format!("Task not found: {}", task_id))?;

        if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
            task.status = match status.to_lowercase().as_str() {
                "pending" => TaskStatus::Pending,
                "in_progress" => {
                    task.started_at = Some(Utc::now());
                    TaskStatus::InProgress
                }
                "completed" => {
                    task.completed_at = Some(Utc::now());
                    TaskStatus::Completed
                }
                "failed" => {
                    task.completed_at = Some(Utc::now());
                    TaskStatus::Failed
                }
                "skipped" => {
                    task.completed_at = Some(Utc::now());
                    TaskStatus::Skipped
                }
                _ => return Err(format!("Invalid status: {}", status)),
            };
            task.updated_at = Utc::now();
        }

        if let Some(result) = args.get("result").and_then(|v| v.as_str()) {
            task.result = Some(result.to_string());
            task.updated_at = Utc::now();
        }

        if let Some(output) = args.get("output") {
            task.output = Some(output.clone());
            task.updated_at = Utc::now();
        }

        Ok(task.clone())
    }

    async fn list_tasks(&self, args: &serde_json::Value) -> Result<Vec<Task>, String> {
        let plan_id = args.get("plan_id").and_then(|v| v.as_str());
        let status = args.get("status").and_then(|v| v.as_str());
        let priority = args.get("priority").and_then(|v| v.as_str());

        let tasks = self.tasks.lock().await;

        let mut filtered: Vec<Task> = tasks.values().cloned().collect();

        if let Some(plan) = plan_id {
            filtered.retain(|t| t.plan_id == plan);
        }

        if let Some(s) = status {
            let status_lower = s.to_lowercase();
            filtered.retain(|t| t.status.to_string() == status_lower);
        }

        if let Some(p) = priority {
            let priority_lower = p.to_lowercase();
            filtered.retain(|t| t.priority.to_string() == priority_lower);
        }

        // Sort by plan_id then task_id
        filtered.sort_by(|a, b| {
            let plan_cmp = a.plan_id.cmp(&b.plan_id);
            if plan_cmp == std::cmp::Ordering::Equal {
                a.id.cmp(&b.id)
            } else {
                plan_cmp
            }
        });

        Ok(filtered)
    }

    async fn get_task(&self, task_id: &str) -> Result<Task, String> {
        let full_id = if task_id.contains('/') {
            task_id.to_string()
        } else {
            format!("default/{}", task_id)
        };
        let tasks = self.tasks.lock().await;
        tasks
            .get(&full_id)
            .cloned()
            .ok_or(format!("Task not found: {}", task_id))
    }

    async fn delete_task(&self, task_id: &str) -> Result<(), String> {
        let full_id = if task_id.contains('/') {
            task_id.to_string()
        } else {
            format!("default/{}", task_id)
        };
        let mut tasks = self.tasks.lock().await;
        tasks
            .remove(&full_id)
            .ok_or(format!("Task not found: {}", task_id))?;
        Ok(())
    }

    async fn reset_tasks(&self, plan_id: Option<&str>) -> Result<usize, String> {
        let mut tasks = self.tasks.lock().await;
        let mut count = 0;

        if let Some(plan) = plan_id {
            tasks.retain(|_, t| {
                if t.plan_id == plan {
                    count += 1;
                    false
                } else {
                    true
                }
            });
        } else {
            count = tasks.len();
            tasks.clear();
        }

        Ok(count)
    }
}

impl Default for TaskTrackerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TaskTrackerTool {
    fn name(&self) -> &str {
        "task_tracker"
    }

    fn description(&self) -> &str {
        "Create and track tasks for complex workflows. Each task has: id, description, status, \
         priority (p0-p3), type (research/action/validation/coordination), dependencies, result. \
         Use to track progress of multi-step plans. \
         Actions: create (new task), update (change status/result), list (filter tasks), \
         get (single task), delete (remove task), reset (clear tasks by plan)"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "update", "list", "get", "delete", "reset"],
                    "description": "Action to perform: create (new task), update (change status/result), list (filter tasks), get (single task), delete (remove task), reset (clear all or by plan)"
                },
                "task_id": {
                    "type": "string",
                    "description": "Unique task identifier (e.g., 'plan-123/1.1' or just '1.1' with plan_id)"
                },
                "plan_id": {
                    "type": "string",
                    "description": "Plan identifier for grouping tasks (used with create and list)"
                },
                "description": {
                    "type": "string",
                    "description": "Task description (used with create)"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "failed", "skipped"],
                    "description": "Task status (used with update)"
                },
                "priority": {
                    "type": "string",
                    "enum": ["p0", "p1", "p2", "p3"],
                    "description": "Priority: p0 (critical), p1 (high), p2 (medium), p3 (low). Default: p2"
                },
                "task_type": {
                    "type": "string",
                    "enum": ["research", "action", "validation", "coordination"],
                    "description": "Task type: research (gather info), action (do something), validation (verify), coordination (synchronize). Default: action"
                },
                "dependencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of task IDs this task depends on (used with create)"
                },
                "result": {
                    "type": "string",
                    "description": "Result or output summary (used with update)"
                },
                "output": {
                    "type": "object",
                    "description": "Structured output data (used with update)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let action = match args.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'action' parameter".to_string()),
                });
            }
        };

        match action {
            "create" => match self.create_task(&args).await {
                Ok(task) => Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&task)?,
                    error: None,
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e),
                }),
            },

            "update" => match self.update_task(&args).await {
                Ok(task) => Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&task)?,
                    error: None,
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e),
                }),
            },

            "list" => match self.list_tasks(&args).await {
                Ok(tasks) => Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&tasks)?,
                    error: None,
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e),
                }),
            },

            "get" => {
                let task_id = args.get("task_id").and_then(|v| v.as_str());
                match task_id {
                    Some(id) => match self.get_task(id).await {
                        Ok(task) => Ok(ToolResult {
                            success: true,
                            output: serde_json::to_string_pretty(&task)?,
                            error: None,
                        }),
                        Err(e) => Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(e),
                        }),
                    },
                    None => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some("Missing 'task_id' for 'get' action".to_string()),
                    }),
                }
            }

            "delete" => {
                let task_id = args.get("task_id").and_then(|v| v.as_str());
                match task_id {
                    Some(id) => match self.delete_task(id).await {
                        Ok(()) => Ok(ToolResult {
                            success: true,
                            output: format!("Task '{}' deleted", id),
                            error: None,
                        }),
                        Err(e) => Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(e),
                        }),
                    },
                    None => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some("Missing 'task_id' for 'delete' action".to_string()),
                    }),
                }
            }

            "reset" => {
                let plan_id = args.get("plan_id").and_then(|v| v.as_str());
                match self.reset_tasks(plan_id).await {
                    Ok(count) => Ok(ToolResult {
                        success: true,
                        output: format!("Reset {} task(s)", count),
                        error: None,
                    }),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    }),
                }
            }

            _ => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown action: '{}'", action)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tool() -> TaskTrackerTool {
        TaskTrackerTool::new()
    }

    #[tokio::test]
    async fn create_and_get_task() {
        let tool = test_tool();
        let result = tool
            .execute(json!({
                "action": "create",
                "task_id": "plan-1/1.1",
                "plan_id": "plan-1",
                "description": "Test task",
                "priority": "p0",
                "task_type": "action"
            }))
            .await
            .unwrap();

        assert!(result.success, "create failed: {:?}", result.error);
        let task: Task = serde_json::from_str(&result.output).expect("valid JSON");
        assert_eq!(task.id, "plan-1/1.1");
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn update_task_status() {
        let tool = test_tool();

        // Create
        let create_result = tool.execute(json!({
            "action": "create",
            "task_id": "t1",
            "description": "Test"
        }))
        .await
        .unwrap();
        assert!(create_result.success, "create failed: {:?}", create_result.error);

        // Update
        let result = tool
            .execute(json!({
                "action": "update",
                "task_id": "t1",
                "status": "in_progress",
                "result": "Working on it"
            }))
            .await
            .unwrap();

        assert!(result.success, "update failed: {:?}", result.error);
        let task: Task = serde_json::from_str(&result.output).expect("valid JSON");
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.result.as_deref(), Some("Working on it"));
    }

    #[tokio::test]
    async fn list_tasks_filtered() {
        let tool = test_tool();

        // Create multiple
        tool.execute(json!({ "action": "create", "task_id": "p1/1", "plan_id": "p1", "priority": "p0" }))
            .await
            .unwrap();
        tool.execute(json!({ "action": "create", "task_id": "p1/2", "plan_id": "p1", "priority": "p2" }))
            .await
            .unwrap();
        tool.execute(json!({ "action": "create", "task_id": "p2/1", "plan_id": "p2", "priority": "p1" }))
            .await
            .unwrap();

        // List all
        let all = tool.execute(json!({ "action": "list" })).await.unwrap();
        assert!(all.success);
        let tasks: Vec<Task> = serde_json::from_str(&all.output).unwrap();
        assert_eq!(tasks.len(), 3);

        // Filter by plan
        let p1 = tool
            .execute(json!({ "action": "list", "plan_id": "p1" }))
            .await
            .unwrap();
        let tasks: Vec<Task> = serde_json::from_str(&p1.output).unwrap();
        assert_eq!(tasks.len(), 2);

        // Filter by priority
        let p0 = tool
            .execute(json!({ "action": "list", "priority": "p0" }))
            .await
            .unwrap();
        let tasks: Vec<Task> = serde_json::from_str(&p0.output).unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn delete_task() {
        let tool = test_tool();

        tool.execute(json!({ "action": "create", "task_id": "todelete", "description": "To be deleted" }))
            .await
            .unwrap();

        let result = tool
            .execute(json!({ "action": "delete", "task_id": "todelete" }))
            .await
            .unwrap();
        assert!(result.success);

        // Verify deleted
        let get_result = tool
            .execute(json!({ "action": "get", "task_id": "todelete" }))
            .await
            .unwrap();
        assert!(!get_result.success);
    }

    #[tokio::test]
    async fn reset_tasks() {
        let tool = test_tool();

        tool.execute(json!({ "action": "create", "task_id": "p1/1", "plan_id": "p1" }))
            .await
            .unwrap();
        tool.execute(json!({ "action": "create", "task_id": "p1/2", "plan_id": "p1" }))
            .await
            .unwrap();
        tool.execute(json!({ "action": "create", "task_id": "p2/1", "plan_id": "p2" }))
            .await
            .unwrap();

        // Reset only plan p1
        let result = tool
            .execute(json!({ "action": "reset", "plan_id": "p1" }))
            .await
            .unwrap();
        assert!(result.success);

        let remaining = tool.execute(json!({ "action": "list" })).await.unwrap();
        let tasks: Vec<Task> = serde_json::from_str(&remaining.output).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "p2/1");
    }

    #[tokio::test]
    async fn dependencies_tracking() {
        let tool = test_tool();

        let r1 = tool.execute(json!({
            "action": "create",
            "task_id": "1",
            "description": "First",
            "dependencies": []
        }))
        .await
        .unwrap();
        assert!(r1.success, "create 1 failed: {:?}", r1.error);

        let r2 = tool.execute(json!({
            "action": "create",
            "task_id": "2",
            "description": "Second",
            "dependencies": ["1"]
        }))
        .await
        .unwrap();
        assert!(r2.success, "create 2 failed: {:?}", r2.error);

        let task2 = tool
            .execute(json!({ "action": "get", "task_id": "2" }))
            .await
            .unwrap();
        assert!(task2.success, "get 2 failed: {:?}", task2.error);
        let task: Task = serde_json::from_str(&task2.output).expect("valid JSON");
        assert_eq!(task.dependencies, vec!["1"]);
    }
}
