use crate::error::AppError;
use crate::services::site_manager::{CreateSiteRequest, Site, SiteManager, UpdateSiteRequest};
use crate::services::template_manager::TemplateManager;
use tauri::AppHandle;

#[tauri::command]
pub fn site_list() -> Result<Vec<Site>, AppError> {
    SiteManager::list()
}

#[tauri::command]
pub fn site_get(id: String) -> Result<Site, AppError> {
    SiteManager::get(&id)
}

#[tauri::command]
pub fn site_create(
    name: String,
    domain: String,
    document_root: String,
    php_version: String,
    ssl: bool,
    template: Option<String>,
) -> Result<Site, AppError> {
    SiteManager::create(CreateSiteRequest {
        name,
        domain,
        document_root,
        php_version,
        ssl,
        template,
    })
}

#[tauri::command]
pub fn site_update(
    id: String,
    name: Option<String>,
    domain: Option<String>,
    document_root: Option<String>,
    php_version: Option<String>,
    ssl: Option<bool>,
    active: Option<bool>,
) -> Result<Site, AppError> {
    SiteManager::update(
        &id,
        UpdateSiteRequest {
            name,
            domain,
            document_root,
            php_version,
            ssl,
            active,
        },
    )
}

#[tauri::command]
pub fn site_delete(id: String) -> Result<(), AppError> {
    SiteManager::delete(&id)
}

#[tauri::command]
pub async fn site_setup_template(
    app: AppHandle,
    site_id: String,
    template: String,
) -> Result<(), AppError> {
    TemplateManager::setup(&app, &site_id, &template).await
}
