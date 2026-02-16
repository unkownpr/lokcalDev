use crate::config::app_config::AppConfig;
use crate::config::paths;
use crate::error::AppError;
use crate::services::mariadb_manager::MariaDbManager;
use crate::services::nginx_manager::NginxManager;
use crate::services::php_manager::PhpManager;
use crate::services::site_manager::{CreateSiteRequest, SiteManager};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

// ── Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<AiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: AiToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStreamChunk {
    pub chunk_type: String, // "content", "tool_calls", "done", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<AiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Tool Definitions ──────────────────────────────────────────────

fn get_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "list_services",
                "description": "List all services and their current status (running/stopped)",
                "parameters": { "type": "object", "properties": {} }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "start_service",
                "description": "Start a service. Valid service IDs: nginx, mariadb, php-fpm-8.1, php-fpm-8.2, php-fpm-8.3, php-fpm-8.4",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "service_id": { "type": "string", "description": "The service ID to start" }
                    },
                    "required": ["service_id"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "stop_service",
                "description": "Stop a service. Valid service IDs: nginx, mariadb, php-fpm-8.1, php-fpm-8.2, php-fpm-8.3, php-fpm-8.4",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "service_id": { "type": "string", "description": "The service ID to stop" }
                    },
                    "required": ["service_id"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "list_sites",
                "description": "List all configured sites with their domains and status",
                "parameters": { "type": "object", "properties": {} }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "create_site",
                "description": "Create a new local development site",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Site name (e.g. 'My App')" },
                        "domain": { "type": "string", "description": "Domain (e.g. 'myapp.test')" },
                        "php_version": { "type": "string", "description": "PHP version (e.g. '8.3')", "default": "8.3" },
                        "template": { "type": "string", "description": "Site template: 'default', 'laravel', 'wordpress'", "default": "default" }
                    },
                    "required": ["name", "domain"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "delete_site",
                "description": "Delete a site by its domain",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "domain": { "type": "string", "description": "Domain of the site to delete" }
                    },
                    "required": ["domain"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write a file to a site's document root directory",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "domain": { "type": "string", "description": "Domain of the site" },
                        "filename": { "type": "string", "description": "Filename to create (e.g. 'index.php', 'style.css')" },
                        "content": { "type": "string", "description": "File content to write" }
                    },
                    "required": ["domain", "filename", "content"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "list_php_versions",
                "description": "List all PHP versions with their installation and running status",
                "parameters": { "type": "object", "properties": {} }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "get_system_info",
                "description": "Get system information including OS, architecture, and data directory",
                "parameters": { "type": "object", "properties": {} }
            }
        }),
    ]
}

// ── Config Reader ─────────────────────────────────────────────────

fn read_config() -> AppConfig {
    let settings_path = paths::get_config_dir().join("settings.toml");
    if settings_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

// ── Fetch Models ──────────────────────────────────────────────────

pub async fn fetch_models(api_key: &str) -> Result<Vec<AiModel>, AppError> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://openrouter.ai/api/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| AppError::Service(format!("Failed to fetch models: {}", e)))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Service(format!("Failed to parse models: {}", e)))?;

    let models = body["data"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| {
            let id = m["id"].as_str()?.to_string();
            let name = m["name"].as_str().unwrap_or(&id).to_string();
            Some(AiModel { id, name })
        })
        .collect();

    Ok(models)
}

// ── Chat Stream ───────────────────────────────────────────────────

pub async fn chat_stream(app: &AppHandle, messages: Vec<AiMessage>) -> Result<(), AppError> {
    let config = read_config();

    if config.openrouter_api_key.is_empty() {
        let _ = app.emit("ai-stream", AiStreamChunk {
            chunk_type: "error".to_string(),
            content: None,
            tool_calls: None,
            error: Some("OpenRouter API key is not configured. Go to Settings > AI to add your key.".to_string()),
        });
        return Ok(());
    }

    // Build messages array for the API
    let mut api_messages: Vec<Value> = vec![json!({
        "role": "system",
        "content": config.ai_system_prompt,
    })];

    for msg in &messages {
        let mut m = json!({ "role": msg.role });
        if let Some(ref content) = msg.content {
            m["content"] = json!(content);
        }
        if let Some(ref tool_calls) = msg.tool_calls {
            m["tool_calls"] = serde_json::to_value(tool_calls).unwrap_or_default();
        }
        if let Some(ref tool_call_id) = msg.tool_call_id {
            m["tool_call_id"] = json!(tool_call_id);
        }
        api_messages.push(m);
    }

    let body = json!({
        "model": config.ai_model,
        "messages": api_messages,
        "tools": get_tool_definitions(),
        "stream": true,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", config.openrouter_api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://lokcaldev.app")
        .header("X-Title", "LokcalDev")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Service(format!("OpenRouter request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        let _ = app.emit("ai-stream", AiStreamChunk {
            chunk_type: "error".to_string(),
            content: None,
            tool_calls: None,
            error: Some(format!("OpenRouter API error ({}): {}", status, error_body)),
        });
        return Ok(());
    }

    let mut stream = resp.bytes_stream();
    let mut accumulated_tool_calls: Vec<AiToolCall> = Vec::new();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let bytes = match chunk_result {
            Ok(b) => b,
            Err(e) => {
                let _ = app.emit("ai-stream", AiStreamChunk {
                    chunk_type: "error".to_string(),
                    content: None,
                    tool_calls: None,
                    error: Some(format!("Stream error: {}", e)),
                });
                return Ok(());
            }
        };

        buffer.push_str(&String::from_utf8_lossy(&bytes));

        // Process complete SSE lines
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.is_empty() || !line.starts_with("data: ") {
                continue;
            }

            let data = &line[6..];
            if data == "[DONE]" {
                if !accumulated_tool_calls.is_empty() {
                    let _ = app.emit("ai-stream", AiStreamChunk {
                        chunk_type: "tool_calls".to_string(),
                        content: None,
                        tool_calls: Some(accumulated_tool_calls.clone()),
                        error: None,
                    });
                }
                let _ = app.emit("ai-stream", AiStreamChunk {
                    chunk_type: "done".to_string(),
                    content: None,
                    tool_calls: None,
                    error: None,
                });
                return Ok(());
            }

            if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                let delta = &parsed["choices"][0]["delta"];

                // Handle content delta
                if let Some(content) = delta["content"].as_str() {
                    if !content.is_empty() {
                        let _ = app.emit("ai-stream", AiStreamChunk {
                            chunk_type: "content".to_string(),
                            content: Some(content.to_string()),
                            tool_calls: None,
                            error: None,
                        });
                    }
                }

                // Handle tool_calls delta
                if let Some(tool_calls) = delta["tool_calls"].as_array() {
                    for tc in tool_calls {
                        let index = tc["index"].as_u64().unwrap_or(0) as usize;

                        // Ensure vector is large enough
                        while accumulated_tool_calls.len() <= index {
                            accumulated_tool_calls.push(AiToolCall {
                                id: String::new(),
                                call_type: "function".to_string(),
                                function: AiToolFunction {
                                    name: String::new(),
                                    arguments: String::new(),
                                },
                            });
                        }

                        if let Some(id) = tc["id"].as_str() {
                            accumulated_tool_calls[index].id = id.to_string();
                        }
                        if let Some(name) = tc["function"]["name"].as_str() {
                            accumulated_tool_calls[index].function.name.push_str(name);
                        }
                        if let Some(args) = tc["function"]["arguments"].as_str() {
                            accumulated_tool_calls[index].function.arguments.push_str(args);
                        }
                    }
                }
            }
        }
    }

    // If stream ended without [DONE], still emit tool calls + done
    if !accumulated_tool_calls.is_empty() {
        let _ = app.emit("ai-stream", AiStreamChunk {
            chunk_type: "tool_calls".to_string(),
            content: None,
            tool_calls: Some(accumulated_tool_calls),
            error: None,
        });
    }
    let _ = app.emit("ai-stream", AiStreamChunk {
        chunk_type: "done".to_string(),
        content: None,
        tool_calls: None,
        error: None,
    });

    Ok(())
}

// ── Tool Execution ────────────────────────────────────────────────

pub fn execute_tool(tool_name: &str, arguments: &str) -> Result<String, AppError> {
    let args: Value = serde_json::from_str(arguments).unwrap_or(json!({}));

    match tool_name {
        "list_services" => {
            let nginx = NginxManager::get_info();
            let mariadb = MariaDbManager::get_info();
            let php_versions = PhpManager::list_versions();

            let mut services = vec![
                json!({
                    "id": "nginx",
                    "name": "Nginx",
                    "running": nginx.running,
                    "version": nginx.version,
                    "port": nginx.port,
                }),
                json!({
                    "id": "mariadb",
                    "name": "MariaDB",
                    "running": mariadb.running,
                    "version": mariadb.version,
                    "port": mariadb.port,
                }),
            ];

            for v in &php_versions {
                if v.installed {
                    services.push(json!({
                        "id": format!("php-fpm-{}", v.version),
                        "name": format!("PHP-FPM {}", v.version),
                        "running": v.running,
                        "version": v.version,
                        "port": v.port,
                    }));
                }
            }

            Ok(serde_json::to_string_pretty(&services).unwrap_or_default())
        }

        "start_service" => {
            let service_id = args["service_id"].as_str().unwrap_or("");
            match service_id {
                "nginx" => {
                    NginxManager::start()?;
                    Ok(format!("Nginx started successfully on port {}", NginxManager::get_info().port))
                }
                "mariadb" => {
                    let _child = MariaDbManager::start()?;
                    // Note: child process is not tracked here, service_commands handles that
                    Ok("MariaDB started successfully".to_string())
                }
                id if id.starts_with("php-fpm-") => {
                    let version = id.strip_prefix("php-fpm-").unwrap();
                    let (_child, _pid) = PhpManager::start_fpm(version)?;
                    Ok(format!("PHP-FPM {} started successfully", version))
                }
                _ => Err(AppError::Service(format!("Unknown service: {}", service_id))),
            }
        }

        "stop_service" => {
            let service_id = args["service_id"].as_str().unwrap_or("");
            match service_id {
                "nginx" => {
                    NginxManager::stop()?;
                    Ok("Nginx stopped successfully".to_string())
                }
                "mariadb" => {
                    MariaDbManager::stop()?;
                    Ok("MariaDB stopped successfully".to_string())
                }
                id if id.starts_with("php-fpm-") => {
                    let version = id.strip_prefix("php-fpm-").unwrap();
                    PhpManager::stop_fpm(version)?;
                    Ok(format!("PHP-FPM {} stopped successfully", version))
                }
                _ => Err(AppError::Service(format!("Unknown service: {}", service_id))),
            }
        }

        "list_sites" => {
            let sites = SiteManager::list()?;
            let site_list: Vec<Value> = sites
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "name": s.name,
                        "domain": s.domain,
                        "documentRoot": s.document_root,
                        "phpVersion": s.php_version,
                        "ssl": s.ssl,
                        "active": s.active,
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&site_list).unwrap_or_default())
        }

        "create_site" => {
            let name = args["name"].as_str().unwrap_or("New Site");
            let domain = args["domain"].as_str().unwrap_or("site.test");
            let php_version = args["php_version"].as_str().unwrap_or("8.3");
            let _template = args["template"].as_str().unwrap_or("default");

            let config = read_config();
            let document_root = std::path::Path::new(&config.sites_directory)
                .join(domain.split('.').next().unwrap_or(domain))
                .to_string_lossy()
                .to_string();

            let req = CreateSiteRequest {
                name: name.to_string(),
                domain: domain.to_string(),
                document_root,
                php_version: php_version.to_string(),
                ssl: false,
                template: None,
            };

            let site = SiteManager::create(req)?;
            Ok(format!(
                "Site '{}' created successfully at {}\nDomain: {}\nPHP: {}",
                site.name, site.document_root, site.domain, site.php_version
            ))
        }

        "delete_site" => {
            let domain = args["domain"].as_str().unwrap_or("");
            let sites = SiteManager::list()?;
            let site = sites
                .iter()
                .find(|s| s.domain == domain)
                .ok_or_else(|| AppError::NotFound(format!("Site with domain '{}' not found", domain)))?;

            SiteManager::delete(&site.id)?;
            Ok(format!("Site '{}' ({}) deleted successfully", site.name, domain))
        }

        "write_file" => {
            let domain = args["domain"].as_str().unwrap_or("");
            let filename = args["filename"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");

            let sites = SiteManager::list()?;
            let site = sites
                .iter()
                .find(|s| s.domain == domain)
                .ok_or_else(|| AppError::NotFound(format!("Site with domain '{}' not found", domain)))?;

            let file_path = std::path::Path::new(&site.document_root).join(filename);

            // Ensure parent directories exist
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, content)?;
            Ok(format!(
                "File '{}' written to {}", filename, file_path.to_string_lossy()
            ))
        }

        "list_php_versions" => {
            let versions = PhpManager::list_versions();
            let list: Vec<Value> = versions
                .iter()
                .map(|v| {
                    json!({
                        "version": v.version,
                        "installed": v.installed,
                        "running": v.running,
                        "port": v.port,
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&list).unwrap_or_default())
        }

        "get_system_info" => {
            let info = json!({
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "dataDir": paths::get_data_dir().to_string_lossy(),
                "nginxPort": NginxManager::get_info().port,
                "mariadbPort": MariaDbManager::get_info().port,
            });
            Ok(serde_json::to_string_pretty(&info).unwrap_or_default())
        }

        _ => Err(AppError::Service(format!("Unknown tool: {}", tool_name))),
    }
}
