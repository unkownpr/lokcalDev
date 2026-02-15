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
    pub mariadb_port: u16,
    pub php_fpm_base_port: u16,
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
            nginx_port: 80,
            mariadb_port: 3306,
            php_fpm_base_port: 9081,
        }
    }
}
