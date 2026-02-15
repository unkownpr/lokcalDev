use crate::error::AppError;
use crate::services::mariadb_manager::{DatabaseEntry, MariaDbInfo, MariaDbManager};
use crate::state::{AppState, ServiceInfo, ServiceStatus};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn mariadb_install(app: AppHandle) -> Result<MariaDbInfo, AppError> {
    MariaDbManager::install(&app).await
}

#[tauri::command]
pub fn mariadb_get_info() -> Result<MariaDbInfo, AppError> {
    Ok(MariaDbManager::get_info())
}

#[tauri::command]
pub fn mariadb_initialize() -> Result<(), AppError> {
    MariaDbManager::initialize_db()
}

#[tauri::command]
pub fn mariadb_start(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
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

#[tauri::command]
pub fn mariadb_stop(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
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

#[tauri::command]
pub fn mariadb_restart(state: State<'_, AppState>) -> Result<ServiceInfo, AppError> {
    // Stop first (don't SIGKILL - let stop() SIGTERM gracefully)
    if let Ok(mut procs) = state.child_processes.lock() {
        procs.remove("mariadb");
    }
    MariaDbManager::stop()?;

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

#[tauri::command]
pub fn database_list() -> Result<Vec<DatabaseEntry>, AppError> {
    MariaDbManager::list_databases()
}

#[tauri::command]
pub fn database_create(name: String) -> Result<(), AppError> {
    MariaDbManager::create_database(&name)
}

#[tauri::command]
pub fn database_drop(name: String) -> Result<(), AppError> {
    MariaDbManager::drop_database(&name)
}
