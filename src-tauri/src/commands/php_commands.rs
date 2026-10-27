use crate::error::AppError;
use crate::services::php_manager::{PhpExtension, PhpIniDirective, PhpManager, PhpVersion};
use crate::services::utils;
use crate::state::{AppState, ServiceInfo, ServiceStatus};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn php_list_versions() -> Result<Vec<PhpVersion>, AppError> {
    Ok(PhpManager::list_versions())
}

#[tauri::command]
pub async fn php_install_version(
    app: AppHandle,
    version: String,
) -> Result<PhpVersion, AppError> {
    PhpManager::install_version(&app, &version).await
}

#[tauri::command(async)]
pub fn php_remove_version(version: String) -> Result<(), AppError> {
    PhpManager::remove_version(&version)
}

#[tauri::command(async)]
pub fn php_start_fpm(
    state: State<'_, AppState>,
    version: String,
) -> Result<ServiceInfo, AppError> {
    let (child, pid) = PhpManager::start_fpm(&version)?;

    let service_id = format!("php-fpm-{}", version);
    let port = utils::php_version_to_port(&version);

    let info = ServiceInfo {
        id: service_id.clone(),
        name: format!("PHP-FPM {}", version),
        status: ServiceStatus::Running,
        port: Some(port),
        version: Some(version),
        pid: Some(pid),
        installed: true,
        initialized: true,
    };

    state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert(service_id.clone(), info.clone());

    state
        .child_processes
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert(service_id, child);

    Ok(info)
}

#[tauri::command(async)]
pub fn php_stop_fpm(
    state: State<'_, AppState>,
    version: String,
) -> Result<ServiceInfo, AppError> {
    let service_id = format!("php-fpm-{}", version);

    // Take the child handle (don't SIGKILL - let the manager SIGTERM gracefully and wait),
    // then reap it so it doesn't linger as a zombie
    let child = state
        .child_processes
        .lock()
        .ok()
        .and_then(|mut procs| procs.remove(&service_id));
    PhpManager::stop_fpm(&version)?;
    if let Some(mut child) = child {
        let _ = child.try_wait();
    }

    let info = ServiceInfo {
        id: service_id.clone(),
        name: format!("PHP-FPM {}", version),
        status: ServiceStatus::Stopped,
        port: Some(utils::php_version_to_port(&version)),
        version: Some(version),
        pid: None,
        installed: true,
        initialized: true,
    };

    state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert(service_id, info.clone());

    Ok(info)
}

#[tauri::command(async)]
pub fn php_restart_fpm(
    state: State<'_, AppState>,
    version: String,
) -> Result<ServiceInfo, AppError> {
    let service_id = format!("php-fpm-{}", version);

    // Take the child handle (don't SIGKILL - let the manager SIGTERM gracefully and wait),
    // then reap it so it doesn't linger as a zombie
    let child = state
        .child_processes
        .lock()
        .ok()
        .and_then(|mut procs| procs.remove(&service_id));
    PhpManager::stop_fpm(&version)?;
    if let Some(mut child) = child {
        let _ = child.try_wait();
    }

    // Start again
    let (child, pid) = PhpManager::start_fpm(&version)?;
    let port = utils::php_version_to_port(&version);

    let info = ServiceInfo {
        id: service_id.clone(),
        name: format!("PHP-FPM {}", version),
        status: ServiceStatus::Running,
        port: Some(port),
        version: Some(version),
        pid: Some(pid),
        installed: true,
        initialized: true,
    };

    state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert(service_id.clone(), info.clone());

    state
        .child_processes
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?
        .insert(service_id, child);

    Ok(info)
}

#[tauri::command]
pub fn php_get_ini(version: String) -> Result<Vec<PhpIniDirective>, AppError> {
    PhpManager::get_ini(&version)
}

#[tauri::command]
pub fn php_set_ini_directive(
    version: String,
    key: String,
    value: String,
) -> Result<(), AppError> {
    PhpManager::set_ini_directive(&version, &key, &value)
}

#[tauri::command]
pub fn php_list_extensions(version: String) -> Result<Vec<PhpExtension>, AppError> {
    PhpManager::list_extensions(&version)
}

#[tauri::command]
pub fn php_toggle_extension(
    version: String,
    extension: String,
    enable: bool,
) -> Result<(), AppError> {
    PhpManager::toggle_extension(&version, &extension, enable)
}
