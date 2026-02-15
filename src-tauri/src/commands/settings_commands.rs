use crate::config::app_config::AppConfig;
use crate::config::paths;
use crate::error::AppError;

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
