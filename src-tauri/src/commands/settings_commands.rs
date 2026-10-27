use crate::config::app_config::AppConfig;
use crate::config::paths;
use crate::error::AppError;
use crate::services::php_manager::PhpManager;
use crate::services::utils;

fn get_settings_path() -> std::path::PathBuf {
    paths::get_config_dir().join("settings.toml")
}

#[tauri::command]
pub fn settings_get() -> Result<AppConfig, AppError> {
    let path = get_settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        // Fall back to defaults if file has old format (snake_case) or is corrupt
        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                log::warn!("Settings file has invalid format, using defaults: {}", e);
                let config = AppConfig::default();
                // Overwrite with correct format
                if let Ok(new_content) = toml::to_string_pretty(&config) {
                    let _ = std::fs::write(&path, new_content);
                }
                Ok(config)
            }
        }
    } else {
        Ok(AppConfig::default())
    }
}

#[tauri::command]
pub fn settings_save(config: AppConfig) -> Result<(), AppError> {
    if config.nginx_port == 0 || config.nginx_ssl_port == 0 || config.nginx_port == config.nginx_ssl_port {
        return Err(AppError::Config(
            "Nginx HTTP and HTTPS ports must be non-zero and different".into(),
        ));
    }
    // The TLD ends up in dnsmasq config and /etc/resolver/<tld>.
    utils::validate_domain(&config.tld)?;
    PhpManager::validate_version(&config.default_php_version)?;

    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(&config)
        .map_err(|e| AppError::Config(e.to_string()))?;
    std::fs::write(&path, content)?;
    log::info!("Settings saved");
    Ok(())
}

#[tauri::command]
pub fn settings_reset() -> Result<AppConfig, AppError> {
    let config = AppConfig::default();
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(&config)
        .map_err(|e| AppError::Config(e.to_string()))?;
    std::fs::write(&path, content)?;
    log::info!("Settings reset to defaults");
    Ok(config)
}
