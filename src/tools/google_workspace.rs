//! Google Workspace API integration tool.
//! Supports OAuth 2.0 (personal Gmail) and Service Account authentication.

use super::traits::{Tool, ToolResult};
use crate::config::GoogleWorkspaceConfig;
use crate::security::SecurityPolicy;
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

pub struct GoogleWorkspaceTool {
    #[allow(dead_code)]
    security: Arc<SecurityPolicy>,
    client: Client,
    oauth_config: Option<OAuthConfig>,
    service_account_path: Option<String>,
    service_account_json: Option<String>,
    access_token: Option<String>,
    token_expiry: Option<u64>,
    auth_method: AuthMethod,
}

#[derive(Clone, Debug, PartialEq)]
enum AuthMethod {
    OAuth2,
    ServiceAccount,
    None,
}

impl GoogleWorkspaceTool {
    pub fn new(security: Arc<SecurityPolicy>, config: GoogleWorkspaceConfig) -> Self {
        let oauth_config = if config.enabled {
            if let (Some(refresh_token), Some(client_id), Some(client_secret)) = (
                &config.refresh_token,
                &config.client_id,
                &config.client_secret,
            ) {
                if !refresh_token.is_empty() && !client_id.is_empty() && !client_secret.is_empty() {
                    Some(OAuthConfig {
                        client_id: client_id.clone(),
                        client_secret: client_secret.clone(),
                        refresh_token: refresh_token.clone(),
                    })
                } else {
                    Self::load_oauth_from_env()
                }
            } else {
                Self::load_oauth_from_env()
            }
        } else {
            Self::load_oauth_from_env()
        };

        let service_account_path = config
            .service_account_path
            .clone()
            .or_else(|| std::env::var("GOOGLE_SERVICE_ACCOUNT_PATH").ok());
        let service_account_json = config
            .service_account_json
            .clone()
            .or_else(|| std::env::var("GOOGLE_SERVICE_ACCOUNT_JSON").ok());

        let auth_method = if oauth_config.is_some() {
            AuthMethod::OAuth2
        } else if service_account_path.is_some() || service_account_json.is_some() {
            AuthMethod::ServiceAccount
        } else {
            AuthMethod::None
        };

        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            security,
            client,
            oauth_config,
            service_account_path,
            service_account_json,
            access_token: None,
            token_expiry: None,
            auth_method,
        }
    }

    fn load_oauth_from_env() -> Option<OAuthConfig> {
        let refresh_token = std::env::var("GOOGLE_REFRESH_TOKEN").ok();
        let client_id = std::env::var("GOOGLE_CLIENT_ID").ok();
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok();

        let refresh = match &refresh_token {
            Some(t) if !t.is_empty() => true,
            _ => false,
        };
        let cid = match &client_id {
            Some(t) if !t.is_empty() => true,
            _ => false,
        };
        let csec = match &client_secret {
            Some(t) if !t.is_empty() => true,
            _ => false,
        };

        if !refresh || !cid || !csec {
            tracing::warn!(
                "Google OAuth not fully configured: refresh={}, client_id={}, client_secret={}",
                refresh,
                cid,
                csec
            );
            return None;
        }

        tracing::info!("Google OAuth credentials loaded from environment");
        Some(OAuthConfig {
            client_id: client_id.unwrap(),
            client_secret: client_secret.unwrap(),
            refresh_token: refresh_token.unwrap(),
        })
    }

    fn get_auth_error_message() -> String {
        "No Google credentials configured. Set GOOGLE_REFRESH_TOKEN, GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET or GOOGLE_SERVICE_ACCOUNT_PATH".to_string()
    }

    async fn get_access_token(&mut self) -> Result<String, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        if let (Some(token), Some(expiry)) = (&self.access_token, self.token_expiry) {
            if now < expiry - 300 {
                return Ok(token.clone());
            }
        }

        match self.auth_method {
            AuthMethod::OAuth2 => self.refresh_oauth_token().await,
            AuthMethod::ServiceAccount => self.get_service_account_token().await,
            AuthMethod::None => Err(Self::get_auth_error_message()),
        }
    }

    async fn refresh_oauth_token(&mut self) -> Result<String, String> {
        let config = self
            .oauth_config
            .as_ref()
            .ok_or_else(|| "OAuth not configured".to_string())?;

        let response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("refresh_token", config.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("OAuth refresh failed: {}", response.status()));
        }

        let token_data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let access_token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| "No access_token in OAuth response".to_string())?
            .to_string();

        let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.token_expiry = Some(now + expires_in);
        self.access_token = Some(access_token.clone());

        Ok(access_token)
    }

    async fn get_service_account_token(&mut self) -> Result<String, String> {
        let credentials: serde_json::Value = if let Some(ref json_str) = self.service_account_json {
            serde_json::from_str(json_str).map_err(|e| e.to_string())?
        } else if let Some(ref path) = self.service_account_path {
            let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            serde_json::from_str(&content).map_err(|e| e.to_string())?
        } else {
            return Err("No service account configured".to_string());
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let service_account = credentials["client_email"]
            .as_str()
            .ok_or_else(|| "Missing client_email".to_string())?;

        let header = json!({"alg": "RS256", "typ": "JWT"});
        let claims = json!({
            "iss": service_account,
            "sub": service_account,
            "aud": "https://oauth2.googleapis.com/token",
            "iat": now,
            "exp": now + 3600,
            "scope": "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/drive https://www.googleapis.com/auth/calendar https://www.googleapis.com/auth/documents https://www.googleapis.com/auth/spreadsheets https://www.googleapis.com/auth/presentations"
        });

        let jwt = format!(
            "{}.{}",
            Self::base64_url_encode(serde_json::to_string(&header).unwrap().as_bytes()),
            Self::base64_url_encode(serde_json::to_string(&claims).unwrap().as_bytes())
        );

        let response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| format!("OAuth request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Service account auth failed: {}",
                response.status()
            ));
        }

        let token_data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let access_token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| "No access_token".to_string())?
            .to_string();

        let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);
        self.token_expiry = Some(now + expires_in);
        self.access_token = Some(access_token.clone());

        Ok(access_token)
    }

    fn base64_url_encode(data: &[u8]) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        URL_SAFE_NO_PAD.encode(data)
    }

    fn clone_for_execution(&self) -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            security: self.security.clone(),
            client,
            oauth_config: self.oauth_config.clone(),
            service_account_path: self.service_account_path.clone(),
            service_account_json: self.service_account_json.clone(),
            access_token: None,
            token_expiry: None,
            auth_method: self.auth_method.clone(),
        }
    }
}

#[async_trait]
impl Tool for GoogleWorkspaceTool {
    fn name(&self) -> &str {
        "google_workspace"
    }

    fn description(&self) -> &str {
        "Google Workspace: Gmail, Drive, Calendar, Docs, Sheets, Slides, Chat"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": {
                    "type": "string",
                    "enum": ["gmail", "drive", "calendar", "docs", "sheets", "chat", "slides"],
                    "description": "Google Workspace service"
                },
                "action": {
                    "type": "string",
                    "enum": ["list", "get", "send", "create", "update", "delete", "append", "attachments", "add_slide", "draft"],
                    "description": "Action to perform"
                },
                "params": {
                    "type": "object",
                    "properties": {
                        "max_results": { "type": "integer" },
                        "query": { "type": "string" },
                        "id": { "type": "string" },
                        "to": { "type": "string" },
                        "subject": { "type": "string" },
                        "body": { "type": "string" },
                        "title": { "type": "string" },
                        "name": { "type": "string" },
                        "content": { "type": "string" },
                        "type": { "type": "string" },
                        "mimeType": { "type": "string" },
                        "parent": { "type": "string" },
                        "range": { "type": "string" },
                        "values": { "type": "array" },
                        "text": { "type": "string" },
                        "calendar_id": { "type": "string" },
                        "event_id": { "type": "string" },
                        "summary": { "type": "string" },
                        "description": { "type": "string" },
                        "start_time": { "type": "string" },
                        "end_time": { "type": "string" },
                        "pageToken": { "type": "string" },
                        "folderId": { "type": "string" },
                        "space": { "type": "string" },
                        "message": { "type": "string" }
                    }
                }
            },
            "required": ["service", "action"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let service = args
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("gmail");
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        let params = args.get("params").and_then(|v| v.as_object());

        if self.auth_method == AuthMethod::None {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(Self::get_auth_error_message()),
            });
        }

        let mut tool = self.clone_for_execution();
        let token = match tool.get_access_token().await {
            Ok(t) => t,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Auth failed: {}", e)),
                });
            }
        };

        let result = match (service, action) {
            ("gmail", "list") => {
                let max = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u32;
                let response = tool
                    .client
                    .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
                    .bearer_auth(&token)
                    .query(&[("maxResults", max.to_string())])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    let status = response.status();
                    let err_body = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!("Gmail API error {}: {}", status, err_body));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("gmail", "get") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .get(&format!(
                        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}",
                        id
                    ))
                    .bearer_auth(&token)
                    .query(&[("format", "full")])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Gmail get error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("gmail", "send") => {
                let to = params
                    .and_then(|p| p.get("to"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'to'"))?;
                let subject = params
                    .and_then(|p| p.get("subject"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("No subject");
                let body = params
                    .and_then(|p| p.get("body"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let message = format!("To: {}\r\nSubject: {}\r\n\r\n{}", to, subject, body);
                use base64::{engine::general_purpose::STANDARD, Engine};
                let encoded = STANDARD.encode(message.as_bytes());
                let response = tool
                    .client
                    .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
                    .bearer_auth(&token)
                    .json(&json!({ "raw": encoded }))
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Gmail send error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("gmail", "draft") | ("gmail", "create") => {
                let to = params
                    .and_then(|p| p.get("to"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'to' parameter"))?;
                let subject = params
                    .and_then(|p| p.get("subject"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let body = params
                    .and_then(|p| p.get("body"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let message = format!("To: {}\r\nSubject: {}\r\n\r\n{}", to, subject, body);
                use base64::{engine::general_purpose::STANDARD, Engine};
                let encoded = STANDARD.encode(message.as_bytes());
                let draft = json!({
                    "message": { "raw": encoded }
                });
                let response = tool
                    .client
                    .post("https://gmail.googleapis.com/gmail/v1/users/me/drafts")
                    .bearer_auth(&token)
                    .json(&draft)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Gmail draft error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("gmail", "delete") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .delete(&format!(
                        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}",
                        id
                    ))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Gmail delete error: {}", response.status()));
                }
                Ok(json!({ "status": "deleted", "id": id }))
            }
            ("gmail", "attachments") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .get(&format!(
                        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}/attachments/{}",
                        id, id
                    ))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Gmail attachments error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("drive", "list") => {
                let query = params.and_then(|p| p.get("query")).and_then(|v| v.as_str());
                let max_results = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u32;
                let page_token = params
                    .and_then(|p| p.get("pageToken"))
                    .and_then(|v| v.as_str());
                let mime_type_filter = params
                    .and_then(|p| p.get("mimeType"))
                    .and_then(|v| v.as_str());
                let folder_id = params
                    .and_then(|p| p.get("folderId"))
                    .and_then(|v| v.as_str());

                let mut query_params = vec![
                    ("pageSize", max_results.to_string()),
                    ("fields", "files(id,name,mimeType,size,createdTime,modifiedTime,webViewLink,parents),nextPageToken".to_string())
                ];

                let mut q = String::new();
                if let Some(f) = folder_id {
                    q.push_str(&format!("'{}' in parents", f));
                }
                if let Some(m) = mime_type_filter {
                    if !q.is_empty() {
                        q.push_str(" and ");
                    }
                    q.push_str(&format!("mimeType='{}'", m));
                }
                if let Some(qq) = query {
                    if !q.is_empty() {
                        q.push_str(" and ");
                    }
                    q.push_str(qq);
                }
                let query_for_log = q.clone();
                if !q.is_empty() {
                    query_params.push(("q", q));
                }

                tracing::info!("Drive list: query={}", query_for_log);

                let mut url = String::from("https://www.googleapis.com/drive/v3/files");
                url.push('?');
                for (i, (k, v)) in query_params.iter().enumerate() {
                    if i > 0 {
                        url.push('&');
                    }
                    url.push_str(&format!("{}={}", k, v));
                }

                let response = tool
                    .client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Drive list error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("drive", "get") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .get(&format!("https://www.googleapis.com/drive/v3/files/{}", id))
                    .bearer_auth(&token)
                    .query(&[(
                        "fields",
                        "id,name,mimeType,size,createdTime,modifiedTime,webViewLink,parents",
                    )])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Drive get error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("drive", "create") => {
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .or_else(|| params.and_then(|p| p.get("title")).and_then(|v| v.as_str()))
                    .ok_or_else(|| anyhow::anyhow!("Missing 'name' or 'title' parameter"))?;
                let parent = params
                    .and_then(|p| p.get("parent"))
                    .and_then(|v| v.as_str());
                let mime_type = params
                    .and_then(|p| p.get("mimeType"))
                    .and_then(|v| v.as_str());
                let file_type = params.and_then(|p| p.get("type")).and_then(|v| v.as_str());
                let content = params
                    .and_then(|p| p.get("content"))
                    .and_then(|v| v.as_str());
                let body = params.and_then(|p| p.get("body")).and_then(|v| v.as_str());

                tracing::info!("Drive create: name={}, type={:?}, mime_type={:?}, parent={:?}, content={:?}, body={:?}", 
                    name, file_type, mime_type, parent, content, body);

                let final_content = content.or(body);
                let is_folder = file_type == Some("folder")
                    || mime_type == Some("application/vnd.google-apps.folder");
                let is_google_app = mime_type
                    .map(|m| m.starts_with("application/vnd.google-apps."))
                    .unwrap_or(false);

                let mut file_metadata = json!({ "name": name });
                if let Some(p) = parent {
                    file_metadata["parents"] = json!([p]);
                }

                let response = if is_folder {
                    file_metadata["mimeType"] = json!("application/vnd.google-apps.folder");
                    tool.client
                        .post("https://www.googleapis.com/drive/v3/files")
                        .bearer_auth(&token)
                        .json(&file_metadata)
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?
                } else if let Some(c) = final_content {
                    if is_google_app {
                        let google_doc_body = json!({
                            "title": name,
                            "body": c
                        });
                        let endpoint = match mime_type {
                            Some("application/vnd.google-apps.document") => {
                                "https://docs.googleapis.com/v1/documents"
                            }
                            Some("application/vnd.google-apps.spreadsheet") => {
                                "https://sheets.googleapis.com/v4/spreadsheets"
                            }
                            _ => "https://www.googleapis.com/drive/v3/files",
                        };
                        if endpoint.contains("documents") {
                            let response = tool
                                .client
                                .post(endpoint)
                                .bearer_auth(&token)
                                .json(&google_doc_body)
                                .send()
                                .await
                                .map_err(|e| anyhow::anyhow!("Google Doc create failed: {}", e))?;
                            if !response.status().is_success() {
                                let status = response.status();
                                let error_text = response.text().await.unwrap_or_default();
                                return Err(anyhow::anyhow!(
                                    "Google Doc error {}: {}",
                                    status,
                                    error_text
                                ));
                            }
                            return response
                                .json()
                                .await
                                .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e));
                        }
                    }
                    let boundary = "-------314159265358979323846";
                    let delimiter = format!("--{}", boundary);
                    let body_str = format!(
                        "{}Content-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n{}Content-Type: text/plain; charset=UTF-8\r\n\r\n{}\r\n--{}--",
                        delimiter,
                        file_metadata.to_string(),
                        delimiter,
                        c,
                        boundary
                    );
                    use base64::{engine::general_purpose::STANDARD, Engine};
                    let encoded = STANDARD.encode(body_str.as_bytes());
                    tool.client
                        .post(
                            "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart",
                        )
                        .bearer_auth(&token)
                        .header(
                            "Content-Type",
                            &format!("multipart/related; boundary={}", boundary),
                        )
                        .body(encoded)
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?
                } else {
                    if let Some(m) = mime_type {
                        file_metadata["mimeType"] = json!(m);
                    }
                    tool.client
                        .post("https://www.googleapis.com/drive/v3/files")
                        .bearer_auth(&token)
                        .json(&file_metadata)
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?
                };

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Drive create error {}: {}",
                        status,
                        error_text
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("drive", "update") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let content = params
                    .and_then(|p| p.get("content"))
                    .and_then(|v| v.as_str());
                let body = params.and_then(|p| p.get("body")).and_then(|v| v.as_str());

                tracing::info!(
                    "Drive update: id={}, content={:?}, body={:?}",
                    id,
                    content,
                    body
                );

                if let Some(c) = content.or(body) {
                    let boundary = "-------314159265358979323846";
                    let delimiter = format!("--{}", boundary);
                    let metadata = json!({ "mimeType": "text/plain" });
                    let body_str = format!(
                        "{}Content-Type: application/json; charset=UTF-8\r\n\r\n{}\r\n{}Content-Type: text/plain; charset=UTF-8\r\n\r\n{}\r\n--{}--",
                        delimiter,
                        metadata.to_string(),
                        delimiter,
                        c,
                        boundary
                    );
                    use base64::{engine::general_purpose::STANDARD, Engine};
                    let encoded = STANDARD.encode(body_str.as_bytes());
                    let response = tool.client
                        .patch(&format!("https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=multipart", id))
                        .bearer_auth(&token)
                        .header("Content-Type", &format!("multipart/related; boundary={}", boundary))
                        .body(encoded)
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Drive update failed: {}", e))?;
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(anyhow::anyhow!(
                            "Drive update error {}: {}",
                            status,
                            error_text
                        ));
                    }
                    response
                        .json()
                        .await
                        .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
                } else {
                    Err(anyhow::anyhow!(
                        "Drive update requires 'content' or 'body' parameter"
                    ))
                }
            }
            ("drive", "delete") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .delete(&format!("https://www.googleapis.com/drive/v3/files/{}", id))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Drive delete error: {}", response.status()));
                }
                Ok(json!({ "status": "deleted", "id": id }))
            }
            ("calendar", "list") => {
                let max = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u32;
                let response = tool
                    .client
                    .get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
                    .bearer_auth(&token)
                    .query(&[("maxResults", max.to_string())])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Calendar API error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("calendar", "get") => {
                let calendar_id = params
                    .and_then(|p| p.get("calendar_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("primary");
                let max = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as u32;
                let response = tool
                    .client
                    .get(&format!(
                        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
                        calendar_id
                    ))
                    .bearer_auth(&token)
                    .query(&[
                        ("maxResults", &max.to_string() as &str),
                        ("singleEvents", "true"),
                    ])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Calendar API error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("calendar", "create") => {
                let calendar_id = params
                    .and_then(|p| p.get("calendar_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("primary");
                let summary = params
                    .and_then(|p| p.get("summary"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'summary'"))?;
                let description = params
                    .and_then(|p| p.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let start_time = params
                    .and_then(|p| p.get("start_time"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'start_time'"))?;
                let end_time = params
                    .and_then(|p| p.get("end_time"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'end_time'"))?;
                let event = json!({
                    "summary": summary,
                    "description": description,
                    "start": { "dateTime": start_time, "timeZone": "UTC" },
                    "end": { "dateTime": end_time, "timeZone": "UTC" }
                });
                let response = tool
                    .client
                    .post(&format!(
                        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
                        calendar_id
                    ))
                    .bearer_auth(&token)
                    .json(&event)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Calendar create error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("calendar", "update") => {
                let calendar_id = params
                    .and_then(|p| p.get("calendar_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("primary");
                let event_id = params
                    .and_then(|p| p.get("event_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'event_id' parameter"))?;
                let summary = params
                    .and_then(|p| p.get("summary"))
                    .and_then(|v| v.as_str());
                let description = params
                    .and_then(|p| p.get("description"))
                    .and_then(|v| v.as_str());
                let start_time = params
                    .and_then(|p| p.get("start_time"))
                    .and_then(|v| v.as_str());
                let end_time = params
                    .and_then(|p| p.get("end_time"))
                    .and_then(|v| v.as_str());
                let mut event = serde_json::Map::new();
                if let Some(s) = summary {
                    event.insert("summary".to_string(), json!(s));
                }
                if let Some(d) = description {
                    event.insert("description".to_string(), json!(d));
                }
                if let Some(st) = start_time {
                    event.insert(
                        "start".to_string(),
                        json!({ "dateTime": st, "timeZone": "UTC" }),
                    );
                }
                if let Some(et) = end_time {
                    event.insert(
                        "end".to_string(),
                        json!({ "dateTime": et, "timeZone": "UTC" }),
                    );
                }
                let response = tool
                    .client
                    .patch(&format!(
                        "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
                        calendar_id, event_id
                    ))
                    .bearer_auth(&token)
                    .json(&event)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Calendar update error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("calendar", "delete") => {
                let calendar_id = params
                    .and_then(|p| p.get("calendar_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("primary");
                let event_id = params
                    .and_then(|p| p.get("event_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'event_id' parameter"))?;
                let response = tool
                    .client
                    .delete(&format!(
                        "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
                        calendar_id, event_id
                    ))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Calendar delete error: {}",
                        response.status()
                    ));
                }
                Ok(json!({ "status": "deleted", "event_id": event_id }))
            }
            ("sheets", "get") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let range = params
                    .and_then(|p| p.get("range"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Sheet1!A1:Z100");
                let response = tool
                    .client
                    .get(&format!(
                        "https://sheets.googleapis.com/v4/spreadsheets/{}",
                        id
                    ))
                    .bearer_auth(&token)
                    .query(&[("ranges", range)])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Sheets get error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("sheets", "create") => {
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
                let spreadsheet = json!({
                    "properties": { "title": name }
                });
                let response = tool
                    .client
                    .post("https://sheets.googleapis.com/v4/spreadsheets")
                    .bearer_auth(&token)
                    .json(&spreadsheet)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Sheets create error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("sheets", "update") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let range = params
                    .and_then(|p| p.get("range"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Sheet1!A1");
                let values = params
                    .and_then(|p| p.get("values"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'values' parameter (array of arrays)")
                    })?;

                let body = json!({
                    "values": values
                });

                let response = tool
                    .client
                    .put(&format!(
                        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED",
                        id, range
                    ))
                    .bearer_auth(&token)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Sheets update error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("sheets", "append") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let range = params
                    .and_then(|p| p.get("range"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Sheet1!A1");
                let values = params
                    .and_then(|p| p.get("values"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing 'values' parameter (array of arrays)")
                    })?;

                let body = json!({
                    "values": values
                });

                let response = tool
                    .client
                    .post(&format!(
                        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
                        id, range
                    ))
                    .bearer_auth(&token)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Sheets append error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("sheets", "delete") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .delete(&format!(
                        "https://sheets.googleapis.com/v4/spreadsheets/{}",
                        id
                    ))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Sheets delete error: {}",
                        response.status()
                    ));
                }
                Ok(json!({ "status": "deleted", "id": id }))
            }
            ("docs", "list") => {
                let max_results = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u32;
                let query = params.and_then(|p| p.get("query")).and_then(|v| v.as_str());

                let mut q = String::from("mimeType='application/vnd.google-apps.document'");
                if let Some(s) = query {
                    q.push_str(&format!(" and name contains '{}'", s));
                }

                let response = tool
                    .client
                    .get("https://www.googleapis.com/drive/v3/files")
                    .bearer_auth(&token)
                    .query(&[
                        ("q", q.as_str()),
                        ("pageSize", &max_results.to_string()),
                        ("fields", "files(id,name,mimeType,createdTime,webViewLink)"),
                    ])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Docs list error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("sheets", "list") => {
                let max_results = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u32;
                let query = params.and_then(|p| p.get("query")).and_then(|v| v.as_str());

                let mut q = String::from("mimeType='application/vnd.google-apps.spreadsheet'");
                if let Some(s) = query {
                    q.push_str(&format!(" and name contains '{}'", s));
                }

                let response = tool
                    .client
                    .get("https://www.googleapis.com/drive/v3/files")
                    .bearer_auth(&token)
                    .query(&[
                        ("q", q.as_str()),
                        ("pageSize", &max_results.to_string()),
                        ("fields", "files(id,name,mimeType,createdTime,webViewLink)"),
                    ])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Sheets list error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("docs", "get") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .get(&format!("https://docs.googleapis.com/v1/documents/{}", id))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Docs get error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("docs", "create") => {
                let title = params
                    .and_then(|p| p.get("title"))
                    .and_then(|v| v.as_str())
                    .or_else(|| params.and_then(|p| p.get("name")).and_then(|v| v.as_str()))
                    .ok_or_else(|| anyhow::anyhow!("Missing 'title' or 'name' parameter"))?;
                let content = params
                    .and_then(|p| p.get("content"))
                    .and_then(|v| v.as_str());
                let body = params.and_then(|p| p.get("body")).and_then(|v| v.as_str());

                let final_content = content.or(body);

                let document = json!({
                    "title": title
                });
                let response = tool
                    .client
                    .post("https://docs.googleapis.com/v1/documents")
                    .bearer_auth(&token)
                    .json(&document)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Docs create error {}: {}",
                        status,
                        error_text
                    ));
                }

                let created_doc: Value = response
                    .json()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))?;

                if let Some(c) = final_content {
                    let doc_id = created_doc["documentId"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("No document ID in response"))?;

                    let requests = json!([
                        {
                            "insertText": {
                                "location": { "index": 1 },
                                "text": c
                            }
                        }
                    ]);

                    let update_response = tool
                        .client
                        .post(&format!(
                            "https://docs.googleapis.com/v1/documents/{}/batchUpdate",
                            doc_id
                        ))
                        .bearer_auth(&token)
                        .json(&json!({ "requests": requests }))
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Docs add content failed: {}", e))?;

                    if !update_response.status().is_success() {
                        return Err(anyhow::anyhow!(
                            "Docs add content error: {}",
                            update_response.status()
                        ));
                    }
                }

                Ok(created_doc)
            }
            ("docs", "update") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let content = params
                    .and_then(|p| p.get("content"))
                    .and_then(|v| v.as_str());
                let body = params.and_then(|p| p.get("body")).and_then(|v| v.as_str());

                let final_content = content.or(body);
                if let Some(c) = final_content {
                    let requests = json!([
                        {
                            "insertText": {
                                "location": { "index": 1 },
                                "text": c
                            }
                        }
                    ]);

                    let response = tool
                        .client
                        .post(&format!(
                            "https://docs.googleapis.com/v1/documents/{}/batchUpdate",
                            id
                        ))
                        .bearer_auth(&token)
                        .json(&json!({ "requests": requests }))
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Docs update failed: {}", e))?;

                    if !response.status().is_success() {
                        return Err(anyhow::anyhow!("Docs update error: {}", response.status()));
                    }
                    response
                        .json()
                        .await
                        .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
                } else {
                    Err(anyhow::anyhow!(
                        "Docs update requires 'content' or 'body' parameter"
                    ))
                }
            }
            ("slides", "create") => {
                let title = params
                    .and_then(|p| p.get("title"))
                    .and_then(|v| v.as_str())
                    .or_else(|| params.and_then(|p| p.get("name")).and_then(|v| v.as_str()))
                    .ok_or_else(|| anyhow::anyhow!("Missing 'title' or 'name' parameter"))?;

                let presentation = json!({
                    "title": title
                });

                let response = tool
                    .client
                    .post("https://slides.googleapis.com/v1/presentations")
                    .bearer_auth(&token)
                    .json(&presentation)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Slides create error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("slides", "get") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let response = tool
                    .client
                    .get(&format!(
                        "https://slides.googleapis.com/v1/presentations/{}",
                        id
                    ))
                    .bearer_auth(&token)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Slides get error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("chat", "send") => {
                let space = params
                    .and_then(|p| p.get("space"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("spaces/AAAARAAAAA");
                let message = params
                    .and_then(|p| p.get("message"))
                    .and_then(|v| v.as_str())
                    .or_else(|| params.and_then(|p| p.get("body")).and_then(|v| v.as_str()))
                    .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter"))?;

                let body = json!({
                    "text": message
                });

                let response = tool
                    .client
                    .post(&format!(
                        "https://chat.googleapis.com/v1/{}/messages",
                        space
                    ))
                    .bearer_auth(&token)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Chat send error {}: {}",
                        status,
                        error_text
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("chat", "list") => {
                let max_results = params
                    .and_then(|p| p.get("max_results"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(50) as u32;

                let response = tool
                    .client
                    .get("https://chat.googleapis.com/v1/spaces")
                    .bearer_auth(&token)
                    .query(&[("pageSize", max_results.to_string())])
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!("Chat list error: {}", response.status()));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            ("slides", "add_slide") => {
                let id = params
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                let layout = params
                    .and_then(|p| p.get("layout"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("BLANK");

                let requests = json!([
                    {
                        "createSlide": {
                            "objectId": "",
                            "slideLayoutReference": {
                                "layout": layout
                            }
                        }
                    }
                ]);

                let response = tool
                    .client
                    .post(&format!(
                        "https://slides.googleapis.com/v1/presentations/{}/batchUpdate",
                        id
                    ))
                    .bearer_auth(&token)
                    .json(&json!({ "requests": requests }))
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("Slides add_slide failed: {}", e))?;
                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Slides add_slide error: {}",
                        response.status()
                    ));
                }
                response
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow::anyhow!("JSON parse failed: {}", e))
            }
            _ => Err(anyhow::anyhow!("Unsupported: {}/{}", service, action)),
        };

        match result {
            Ok(output) => Ok(ToolResult {
                success: true,
                output: serde_json::to_string_pretty(&output).unwrap_or_default(),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}
