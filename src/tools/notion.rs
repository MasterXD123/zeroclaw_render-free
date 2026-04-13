//! Notion API integration tool.
//! Provides access to Notion databases, pages, and search.

use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct NotionTool {
    #[allow(dead_code)]
    security: Arc<SecurityPolicy>,
    client: Client,
    api_key: Option<String>,
}

impl NotionTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        let api_key = std::env::var("NOTION_KEY").ok();
        Self {
            security,
            client: Client::new(),
            api_key,
        }
    }

    fn get_api_key(&self) -> Result<String, String> {
        self.api_key
            .clone()
            .ok_or_else(|| "NOTION_KEY not configured".to_string())
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
            "Authorization",
            format!("Bearer {}", self.get_api_key().unwrap_or_default())
                .parse()
                .unwrap(),
        );
        h.insert("Content-Type", "application/json".parse().unwrap());
        h.insert("Notion-Version", "2022-06-28".parse().unwrap());
        h
    }

    pub async fn search(&self, query: &str) -> Result<Value, String> {
        let _api_key = self.get_api_key()?;
        let url = "https://api.notion.com/v1/search";
        let body = json!({ "query": query, "filter": { "value": "page", "property": "object" } });
        let r = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_page(&self, page_id: &str) -> Result<Value, String> {
        let url = format!("https://api.notion.com/v1/pages/{}", page_id);
        let r = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_database(&self, db_id: &str) -> Result<Value, String> {
        let url = format!("https://api.notion.com/v1/databases/{}", db_id);
        let r = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn query_database(
        &self,
        db_id: &str,
        filter: Option<Value>,
    ) -> Result<Value, String> {
        let url = format!("https://api.notion.com/v1/databases/{}/query", db_id);
        let mut body = json!({});
        if let Some(f) = filter {
            body["filter"] = f;
        }
        let r = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn create_page(
        &self,
        parent_id: &str,
        properties: Value,
        children: Option<Value>,
    ) -> Result<Value, String> {
        let url = "https://api.notion.com/v1/pages";
        let mut body = json!({
            "parent": { "page_id": parent_id },
            "properties": properties
        });
        if let Some(c) = children {
            body["children"] = c;
        }
        let r = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn get_block_children(&self, block_id: &str) -> Result<Value, String> {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", block_id);
        let r = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            return Err(format!("Error: {}", r.status()));
        }
        r.json().await.map_err(|e| e.to_string())
    }
}

#[async_trait]
impl Tool for NotionTool {
    fn name(&self) -> &str {
        "notion"
    }

    fn description(&self) -> &str {
        "Notion API: search pages/databases, get pages, query databases, create pages"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["search", "get_page", "get_database", "query_database", "create_page", "get_blocks"],
                    "description": "Action to perform"
                },
                "id": { "type": "string", "description": "Page ID, Database ID, or Block ID" },
                "query": { "type": "string", "description": "Search query" },
                "filter": { "type": "object", "description": "Filter for database query (JSON)" },
                "properties": { "type": "object", "description": "Properties for page creation (JSON)" },
                "children": { "type": "array", "description": "Block children for page creation" }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("search");

        match action {
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                match self.search(query).await {
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
            "get_page" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.get_page(id).await {
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
            "get_database" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.get_database(id).await {
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
            "query_database" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let filter = args.get("filter").cloned();
                match self.query_database(id, filter).await {
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
            "create_page" => {
                let parent_id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let properties = args.get("properties").cloned().unwrap_or(json!({}));
                let children = args.get("children").cloned();
                match self.create_page(parent_id, properties, children).await {
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
            "get_blocks" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                match self.get_block_children(id).await {
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
