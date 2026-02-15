use crate::error::AppError;
use crate::services::mariadb_manager::MariaDbManager;
use crate::services::nginx_manager::NginxManager;
use crate::services::php_manager::PhpManager;
use crate::services::phpmyadmin_manager::PhpMyAdminManager;
use crate::services::utils;
use crate::state::{AppState, ServiceInfo, ServiceStatus};
use tauri::State;

#[tauri::command]
pub fn get_all_services(state: State<'_, AppState>) -> Result<Vec<ServiceInfo>, AppError> {
    // Refresh real installation/running status from managers
    let nginx_info = NginxManager::get_info();
    let mariadb_info = MariaDbManager::get_info();
    let pma_info = PhpMyAdminManager::get_info();

    let mut services = state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?;

    // Update nginx with real status
    if let Some(svc) = services.get_mut("nginx") {
        svc.installed = nginx_info.installed;
        svc.initialized = true;
        svc.version = nginx_info.version;
        if nginx_info.running {
            svc.status = ServiceStatus::Running;
            svc.pid = nginx_info.pid;
        } else {
            svc.status = ServiceStatus::Stopped;
            svc.pid = None;
        }
    }

    // Update mariadb with real status
    if let Some(svc) = services.get_mut("mariadb") {
        svc.installed = mariadb_info.installed;
        svc.initialized = mariadb_info.initialized;
        svc.version = mariadb_info.version;
        if mariadb_info.running {
            svc.status = ServiceStatus::Running;
            svc.pid = mariadb_info.pid;
        } else {
            svc.status = ServiceStatus::Stopped;
            svc.pid = None;
        }
    }

    // Update phpMyAdmin with real status (no process — installed or not)
    if let Some(svc) = services.get_mut("phpmyadmin") {
        svc.installed = pma_info.installed;
        svc.initialized = pma_info.installed;
        svc.version = pma_info.version;
        svc.status = if pma_info.installed {
            ServiceStatus::Running
        } else {
            ServiceStatus::Stopped
        };
    }

    let mut result: Vec<ServiceInfo> = services.values().cloned().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

#[tauri::command]
pub fn get_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceInfo, AppError> {
    let services = state
        .services
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?;
    services
        .get(&service_id)
        .cloned()
        .ok_or_else(|| AppError::Service(format!("Service '{}' not found", service_id)))
}

#[tauri::command]
pub fn start_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceInfo, AppError> {
    match service_id.as_str() {
        "nginx" => {
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
        "mariadb" => {
            let child = MariaDbManager::start()?;
            let pid = child.id();
            let db_info = MariaDbManager::get_info();

            let info = ServiceInfo {
                id: "mariadb".to_string(),
                name: "MariaDB".to_string(),
                status: ServiceStatus::Running,
                port: Some(3306),
                version: db_info.version,
                pid: Some(pid),
                installed: true,
                initialized: true,
            };

            state
                .services
                .lock()
                .map_err(|e| AppError::Service(e.to_string()))?
                .insert("mariadb".to_string(), info.clone());

            state
                .child_processes
                .lock()
                .map_err(|e| AppError::Service(e.to_string()))?
                .insert("mariadb".to_string(), child);

            Ok(info)
        }
        id if id.starts_with("php-fpm-") => {
            let version = id.strip_prefix("php-fpm-").unwrap();
            let (child, pid) = PhpManager::start_fpm(version)?;
            let port = utils::php_version_to_port(version);

            let info = ServiceInfo {
                id: service_id.clone(),
                name: format!("PHP-FPM {}", version),
                status: ServiceStatus::Running,
                port: Some(port),
                version: Some(version.to_string()),
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
        "phpmyadmin" => {
            // phpMyAdmin has no process — return current info
            let pma = PhpMyAdminManager::get_info();
            let info = ServiceInfo {
                id: "phpmyadmin".to_string(),
                name: "phpMyAdmin".to_string(),
                status: if pma.installed { ServiceStatus::Running } else { ServiceStatus::Stopped },
                port: None,
                version: pma.version,
                pid: None,
                installed: pma.installed,
                initialized: pma.installed,
            };
            Ok(info)
        }
        _ => Err(AppError::Service(format!(
            "Unknown service: '{}'",
            service_id
        ))),
    }
}

#[tauri::command]
pub fn stop_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceInfo, AppError> {
    match service_id.as_str() {
        "nginx" => {
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
        "mariadb" => {
            // Remove child process handle (don't SIGKILL - let stop() SIGTERM gracefully)
            if let Ok(mut procs) = state.child_processes.lock() {
                procs.remove("mariadb");
            }
            MariaDbManager::stop()?;

            let db_info = MariaDbManager::get_info();
            let info = ServiceInfo {
                id: "mariadb".to_string(),
                name: "MariaDB".to_string(),
                status: ServiceStatus::Stopped,
                port: Some(3306),
                version: db_info.version,
                pid: None,
                installed: true,
                initialized: true,
            };

            state
                .services
                .lock()
                .map_err(|e| AppError::Service(e.to_string()))?
                .insert("mariadb".to_string(), info.clone());

            Ok(info)
        }
        id if id.starts_with("php-fpm-") => {
            let version = id.strip_prefix("php-fpm-").unwrap();

            // Remove child process handle (don't SIGKILL - let stop_fpm() SIGTERM gracefully)
            if let Ok(mut procs) = state.child_processes.lock() {
                procs.remove(&service_id);
            }
            PhpManager::stop_fpm(version)?;

            let info = ServiceInfo {
                id: service_id.clone(),
                name: format!("PHP-FPM {}", version),
                status: ServiceStatus::Stopped,
                port: Some(utils::php_version_to_port(version)),
                version: Some(version.to_string()),
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
        "phpmyadmin" => {
            // phpMyAdmin has no process — return current info
            let pma = PhpMyAdminManager::get_info();
            let info = ServiceInfo {
                id: "phpmyadmin".to_string(),
                name: "phpMyAdmin".to_string(),
                status: if pma.installed { ServiceStatus::Running } else { ServiceStatus::Stopped },
                port: None,
                version: pma.version,
                pid: None,
                installed: pma.installed,
                initialized: pma.installed,
            };
            Ok(info)
        }
        _ => Err(AppError::Service(format!(
            "Unknown service: '{}'",
            service_id
        ))),
    }
}

#[tauri::command]
pub fn restart_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceInfo, AppError> {
    stop_service(State::clone(&state), service_id.clone())?;
    start_service(state, service_id)
}
