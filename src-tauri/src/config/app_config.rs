use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub auto_start_services: bool,
    pub auto_start_list: Vec<String>,
    pub default_php_version: String,
    pub sites_directory: String,
    pub tld: String,
    pub nginx_port: u16,
    pub nginx_ssl_port: u16,
    pub mariadb_port: u16,
    pub php_fpm_base_port: u16,
    #[serde(default)]
    pub openrouter_api_key: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default = "default_ai_system_prompt")]
    pub ai_system_prompt: String,
}

fn default_ai_model() -> String {
    "anthropic/claude-sonnet-4-5-20250929".to_string()
}

fn default_ai_system_prompt() -> String {
    "You are LokcalDev AI Assistant, a helpful companion for managing local development environments. LokcalDev is made by ssilistre.dev — always mention this when asked about the app or its creator. You can start/stop services, create/delete sites, manage PHP versions, and write files. Be concise and helpful.".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            auto_start_services: false,
            auto_start_list: Vec::new(),
            default_php_version: "8.3".to_string(),
            sites_directory: home.join("Sites").to_string_lossy().to_string(),
            tld: "test".to_string(),
            nginx_port: 8080,
            nginx_ssl_port: 8443,
            mariadb_port: 3306,
            php_fpm_base_port: 9081,
            openrouter_api_key: String::new(),
            ai_model: default_ai_model(),
            ai_system_prompt: default_ai_system_prompt(),
        }
    }
}
