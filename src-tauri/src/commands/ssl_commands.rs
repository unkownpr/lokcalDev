use crate::error::AppError;
use crate::services::dns_manager::{DnsEntry, DnsManager, ResolverStatus};
use crate::services::ssl_manager::{CertificateInfo, SslManager};
use tauri::AppHandle;

#[tauri::command]
pub async fn ssl_install_mkcert(app: AppHandle) -> Result<(), AppError> {
    SslManager::install_mkcert(&app).await
}

#[tauri::command]
pub fn ssl_is_mkcert_installed() -> Result<bool, AppError> {
    Ok(SslManager::is_mkcert_installed())
}

#[tauri::command]
pub fn ssl_install_ca() -> Result<(), AppError> {
    SslManager::install_ca()
}

#[tauri::command]
pub fn ssl_is_ca_installed() -> Result<bool, AppError> {
    Ok(SslManager::is_ca_installed())
}

#[tauri::command]
pub fn ssl_generate_certificate(domain: String) -> Result<CertificateInfo, AppError> {
    SslManager::generate_certificate(&domain)
}

#[tauri::command]
pub fn ssl_remove_certificate(domain: String) -> Result<(), AppError> {
    SslManager::remove_certificate(&domain)
}

#[tauri::command]
pub fn ssl_list_certificates() -> Result<Vec<CertificateInfo>, AppError> {
    SslManager::list_certificates()
}

#[tauri::command]
pub fn dns_add_entry(domain: String, ip: String) -> Result<(), AppError> {
    DnsManager::add_entry(&domain, &ip)
}

#[tauri::command]
pub fn dns_remove_entry(domain: String) -> Result<(), AppError> {
    DnsManager::remove_entry(&domain)
}

#[tauri::command]
pub fn dns_list_entries() -> Result<Vec<DnsEntry>, AppError> {
    DnsManager::list_entries()
}

#[tauri::command]
pub fn dns_get_resolver_status(tld: String) -> ResolverStatus {
    DnsManager::get_resolver_status(&tld)
}

#[tauri::command]
pub fn dns_ensure_dnsmasq_running(tld: String) -> Result<(), AppError> {
    DnsManager::ensure_dnsmasq_running(&tld)
}

#[tauri::command]
pub fn dns_setup_resolver(tld: String) -> Result<(), AppError> {
    DnsManager::setup_resolver(&tld)
}
