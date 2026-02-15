use crate::config::paths;
use crate::error::AppError;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub data_dir: String,
    pub app_version: String,
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, AppError> {
    Ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        data_dir: paths::get_data_dir()
            .to_string_lossy()
            .to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub fn initialize_app() -> Result<String, AppError> {
    let data_dir = paths::get_data_dir();

    let subdirs = [
        "config",
        "config/nginx",
        "config/nginx/sites-enabled",
        "logs",
        "data",
        "ssl",
        "ssl/ca",
        "sites",
        "binaries",
        "binaries/php",
        "binaries/nginx",
        "binaries/mariadb",
        "binaries/mkcert",
    ];
    for subdir in &subdirs {
        let dir = data_dir.join(subdir);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            log::info!("Created directory: {}", dir.display());
        }
    }

    log::info!("App initialized. Data dir: {}", data_dir.display());
    Ok(data_dir.to_string_lossy().to_string())
}
