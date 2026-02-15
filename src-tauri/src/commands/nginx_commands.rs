use crate::error::AppError;
use crate::services::nginx_manager::{NginxInfo, NginxManager};
use crate::state::{AppState, ServiceInfo, ServiceStatus};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn nginx_install(app: AppHandle) -> Result<NginxInfo, AppError> {
    NginxManager::install(&app).await
}

#[tauri::command]
pub fn nginx_get_info() -> Result<NginxInfo, AppError> {
    Ok(NginxManager::get_info())
}

#[tauri::command]
pub fn nginx_start(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
    let pid = NginxManager::start()?;
    let nginx_info = NginxManager::get_info();

    let info = ServiceInfo {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        status: ServiceStatus::Running,
        port: Some(80),
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

    let info = ServiceInfo {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        status: ServiceStatus::Stopped,
        port: Some(80),
        version: NginxManager::get_info().version,
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

    let pid = NginxManager::start()?;
    let nginx_info = NginxManager::get_info();

    let info = ServiceInfo {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        status: ServiceStatus::Running,
        port: Some(80),
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
