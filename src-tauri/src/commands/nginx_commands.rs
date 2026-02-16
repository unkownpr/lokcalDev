use crate::error::AppError;
use crate::services::nginx_manager::{NginxInfo, NginxManager};
use crate::services::php_manager::PhpManager;
use crate::services::site_manager::SiteManager;
use crate::state::{AppState, ServiceInfo, ServiceStatus};
use std::collections::HashSet;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn nginx_install(app: AppHandle) -> Result<NginxInfo, AppError> {
    NginxManager::install(&app).await
}

#[tauri::command]
pub fn nginx_get_info() -> Result<NginxInfo, AppError> {
    Ok(NginxManager::get_info())
}

/// Auto-start PHP-FPM for all PHP versions used by active sites.
fn auto_start_required_fpm(state: &State<'_, AppState>) {
    let sites = match SiteManager::list() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Collect unique PHP versions from active sites
    let versions: HashSet<String> = sites
        .iter()
        .filter(|s| s.active)
        .map(|s| s.php_version.clone())
        .collect();

    let php_versions = PhpManager::list_versions();

    for version in &versions {
        // Check if this PHP version is installed and not already running
        let php_info = php_versions.iter().find(|v| &v.version == version);
        if let Some(info) = php_info {
            if info.installed && !info.running {
                match PhpManager::start_fpm(version) {
                    Ok((child, pid)) => {
                        // Store child process handle for cleanup on exit
                        let key = format!("php-fpm-{}", version);
                        if let Ok(mut children) = state.child_processes.lock() {
                            children.insert(key.clone(), child);
                        }
                        // Update service state
                        let svc = ServiceInfo {
                            id: key.clone(),
                            name: format!("PHP-FPM {}", version),
                            status: ServiceStatus::Running,
                            port: Some(info.port),
                            version: Some(version.clone()),
                            pid: Some(pid),
                            installed: true,
                            initialized: true,
                        };
                        if let Ok(mut services) = state.services.lock() {
                            services.insert(key, svc);
                        }
                        log::info!("Auto-started PHP-FPM {} for active sites", version);
                    }
                    Err(e) => {
                        log::warn!("Failed to auto-start PHP-FPM {}: {}", version, e);
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub fn nginx_start(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
    // Auto-start PHP-FPM for active sites before starting nginx
    auto_start_required_fpm(&state);

    let pid = NginxManager::start()?;
    let nginx_info = NginxManager::get_info();

    let info = ServiceInfo {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        status: ServiceStatus::Running,
        port: Some(nginx_info.port),
        version: nginx_info.version,
        pid: if pid > 0 { Some(pid) } else { nginx_info.pid },
        installed: true,
        initialized: true,
    };

    state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert("nginx".to_string(), info.clone());

    Ok(info)
}

#[tauri::command]
pub fn nginx_stop(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
    NginxManager::stop()?;
    let nginx_info = NginxManager::get_info();

    let info = ServiceInfo {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        status: ServiceStatus::Stopped,
        port: Some(nginx_info.port),
        version: nginx_info.version,
        pid: None,
        installed: true,
        initialized: true,
    };

    state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert("nginx".to_string(), info.clone());

    Ok(info)
}

#[tauri::command]
pub fn nginx_restart(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
    NginxManager::stop()?;

    // Auto-start PHP-FPM for active sites before restarting nginx
    auto_start_required_fpm(&state);

    let pid = NginxManager::start()?;
    let nginx_info = NginxManager::get_info();

    let info = ServiceInfo {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        status: ServiceStatus::Running,
        port: Some(nginx_info.port),
        version: nginx_info.version,
        pid: if pid > 0 { Some(pid) } else { nginx_info.pid },
        installed: true,
        initialized: true,
    };

    state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert("nginx".to_string(), info.clone());

    Ok(info)
}

#[tauri::command]
pub fn nginx_reload() -> Result<(), AppError> {
    NginxManager::reload()
}

#[tauri::command]
pub fn nginx_test_config() -> Result<String, AppError> {
    NginxManager::test_config()
}
