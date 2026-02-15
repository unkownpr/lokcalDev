use crate::error::AppError;
use crate::services::phpmyadmin_manager::{PhpMyAdminInfo, PhpMyAdminManager};
use tauri::AppHandle;

#[tauri::command]
pub async fn phpmyadmin_install(app: AppHandle) -> Result<PhpMyAdminInfo, AppError> {
    PhpMyAdminManager::install(&app).await
}

#[tauri::command]
pub fn phpmyadmin_get_info() -> Result<PhpMyAdminInfo, AppError> {
    Ok(PhpMyAdminManager::get_info())
}
