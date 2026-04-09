//! GitHub API integration tool.
//! Provides access to GitHub issues, PRs, repos, and workflows.

use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct GitHubTool {
    #[allow(dead_code)]
    security: Arc<SecurityPolicy>,
    client: Client,
    api_key: Option<String>,
}

impl GitHubTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        let api_key = std::env::var("GITHUB_TOKEN").ok();
        Self {
            security,
            client: Client::new(),
            api_key,
        }
    }

    fn get_api_key(&self) -> Result<String, String> {
        self.api_key
            .clone()
            .ok_or_else(|| "GITHUB_TOKEN not configured".to_string())
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());
        h.insert("User-Agent", "ZeroClaw".parse().unwrap());
        h
    }

    pub async fn list_issues(&self, owner: &str, repo: &str, state: &str) -> Result<Value, String> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues?state={}",
            owner, repo, state
        );
        let r = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        labels: Vec<&str>,
    ) -> Result<Value, String> {
        let api_key = self.get_api_key()?;
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let mut body_obj = json!({ "title": title, "body": body });
        if !labels.is_empty() {
            body_obj["labels"] = serde_json::json!(labels);
        }
        let r = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .headers(self.headers())
            .json(&body_obj)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn list_pulls(&self, owner: &str, repo: &str, state: &str) -> Result<Value, String> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state={}",
            owner, repo, state
        );
        let r = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<Value, String> {
        let api_key = self.get_api_key()?;
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let r = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn list_user_repos(&self) -> Result<Value, String> {
        let api_key = self.get_api_key()?;
        let r = self
            .client
            .get("https://api.github.com/user/repos?sort=updated&per_page=10")
            .header("Authorization", format!("Bearer {}", api_key))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn list_workflows(&self, owner: &str, repo: &str) -> Result<Value, String> {
        let api_key = self.get_api_key()?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/workflows",
            owner, repo
        );
        let r = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }
}

#[async_trait]
impl Tool for GitHubTool {
    fn name(&self) -> &str {
        "github"
    }

    fn description(&self) -> &str {
        "GitHub API: issues, PRs, repos, workflows"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list_issues", "create_issue", "list_pulls", "get_repo", "list_repos", "list_workflows"],
                    "description": "Action to perform"
                },
                "owner": { "type": "string", "description": "Repo owner" },
                "repo": { "type": "string", "description": "Repo name" },
                "title": { "type": "string" },
                "body": { "type": "string" },
                "state": { "type": "string", "enum": ["open", "closed", "all"] },
                "labels": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list_issues");

        match action {
            "list_issues" => {
                let owner = args.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("open");
                match self.list_issues(owner, repo, state).await {
                    Ok(v) => Ok(ToolResult {
                        success: true,
                        output: serde_json::to_string(&v).unwrap_or_default(),
                        error: None,
                    }),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    }),
                }
            }
            "create_issue" => {
                let owner = args.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
                let labels: Vec<&str> = args
                    .get("labels")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                match self.create_issue(owner, repo, title, body, labels).await {
                    Ok(v) => Ok(ToolResult {
                        success: true,
                        output: serde_json::to_string(&v).unwrap_or_default(),
                        error: None,
                    }),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    }),
                }
            }
            "list_pulls" => {
                let owner = args.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("open");
                match self.list_pulls(owner, repo, state).await {
                    Ok(v) => Ok(ToolResult {
                        success: true,
                        output: serde_json::to_string(&v).unwrap_or_default(),
                        error: None,
                    }),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    }),
                }
            }
            "get_repo" => {
                let owner = args.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                match self.get_repo(owner, repo).await {
                    Ok(v) => Ok(ToolResult {
                        success: true,
                        output: serde_json::to_string(&v).unwrap_or_default(),
                        error: None,
                    }),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    }),
                }
            }
            "list_repos" => match self.list_user_repos().await {
                Ok(v) => Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string(&v).unwrap_or_default(),
                    error: None,
                }),
                Err(e) => Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e),
                }),
            },
            "list_workflows" => {
                let owner = args.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = args.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                match self.list_workflows(owner, repo).await {
                    Ok(v) => Ok(ToolResult {
                        success: true,
                        output: serde_json::to_string(&v).unwrap_or_default(),
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
                error: Some("Unknown action".to_string()),
            }),
        }
    }
}
