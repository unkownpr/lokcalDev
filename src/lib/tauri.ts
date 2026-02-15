import { invoke } from "@tauri-apps/api/core"
import type { ServiceInfo } from "@/types/service"
import type { SystemInfo, AppConfig } from "@/types/config"
import type { PhpVersion, PhpIniDirective, PhpExtension } from "@/types/php"
import type { NginxInfo, Site } from "@/types/nginx"
import type { MariaDbInfo, DatabaseEntry, PhpMyAdminInfo } from "@/types/database"
import type { CertificateInfo, DnsEntry, LogFile } from "@/types/ssl"

// Service commands
export async function getAllServices(): Promise<ServiceInfo[]> {
  return invoke<ServiceInfo[]>("get_all_services")
}

export async function getService(serviceId: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("get_service", { serviceId })
}

export async function startService(serviceId: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("start_service", { serviceId })
}

export async function stopService(serviceId: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("stop_service", { serviceId })
}

export async function restartService(serviceId: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("restart_service", { serviceId })
}

// System commands
export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>("get_system_info")
}

export async function initializeApp(): Promise<string> {
  return invoke<string>("initialize_app")
}

// PHP commands
export async function phpListVersions(): Promise<PhpVersion[]> {
  return invoke<PhpVersion[]>("php_list_versions")
}

export async function phpInstallVersion(version: string): Promise<PhpVersion> {
  return invoke<PhpVersion>("php_install_version", { version })
}

export async function phpRemoveVersion(version: string): Promise<void> {
  return invoke<void>("php_remove_version", { version })
}

export async function phpStartFpm(version: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("php_start_fpm", { version })
}

export async function phpStopFpm(version: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("php_stop_fpm", { version })
}

export async function phpRestartFpm(version: string): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("php_restart_fpm", { version })
}

export async function phpGetIni(version: string): Promise<PhpIniDirective[]> {
  return invoke<PhpIniDirective[]>("php_get_ini", { version })
}

export async function phpSetIniDirective(version: string, key: string, value: string): Promise<void> {
  return invoke<void>("php_set_ini_directive", { version, key, value })
}

export async function phpListExtensions(version: string): Promise<PhpExtension[]> {
  return invoke<PhpExtension[]>("php_list_extensions", { version })
}

export async function phpToggleExtension(version: string, extension: string, enable: boolean): Promise<void> {
  return invoke<void>("php_toggle_extension", { version, extension, enable })
}

// Nginx commands
export async function nginxInstall(): Promise<NginxInfo> {
  return invoke<NginxInfo>("nginx_install")
}

export async function nginxGetInfo(): Promise<NginxInfo> {
  return invoke<NginxInfo>("nginx_get_info")
}

export async function nginxStart(): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("nginx_start")
}

export async function nginxStop(): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("nginx_stop")
}

export async function nginxRestart(): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("nginx_restart")
}

export async function nginxReload(): Promise<void> {
  return invoke<void>("nginx_reload")
}

export async function nginxTestConfig(): Promise<string> {
  return invoke<string>("nginx_test_config")
}

// Site commands
export async function siteList(): Promise<Site[]> {
  return invoke<Site[]>("site_list")
}

export async function siteGet(id: string): Promise<Site> {
  return invoke<Site>("site_get", { id })
}

export async function siteCreate(
  name: string,
  domain: string,
  documentRoot: string,
  phpVersion: string,
  ssl: boolean,
): Promise<Site> {
  return invoke<Site>("site_create", { name, domain, documentRoot, phpVersion, ssl })
}

export async function siteUpdate(
  id: string,
  name?: string,
  domain?: string,
  documentRoot?: string,
  phpVersion?: string,
  ssl?: boolean,
  active?: boolean,
): Promise<Site> {
  return invoke<Site>("site_update", { id, name, domain, documentRoot, phpVersion, ssl, active })
}

export async function siteDelete(id: string): Promise<void> {
  return invoke<void>("site_delete", { id })
}

// Database commands
export async function mariadbInstall(): Promise<MariaDbInfo> {
  return invoke<MariaDbInfo>("mariadb_install")
}

export async function mariadbGetInfo(): Promise<MariaDbInfo> {
  return invoke<MariaDbInfo>("mariadb_get_info")
}

export async function mariadbInitialize(): Promise<void> {
  return invoke<void>("mariadb_initialize")
}

export async function mariadbStart(): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("mariadb_start")
}

export async function mariadbStop(): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("mariadb_stop")
}

export async function mariadbRestart(): Promise<ServiceInfo> {
  return invoke<ServiceInfo>("mariadb_restart")
}

export async function databaseList(): Promise<DatabaseEntry[]> {
  return invoke<DatabaseEntry[]>("database_list")
}

export async function databaseCreate(name: string): Promise<void> {
  return invoke<void>("database_create", { name })
}

export async function databaseDrop(name: string): Promise<void> {
  return invoke<void>("database_drop", { name })
}

// phpMyAdmin commands
export async function phpmyadminInstall(): Promise<PhpMyAdminInfo> {
  return invoke<PhpMyAdminInfo>("phpmyadmin_install")
}

export async function phpmyadminGetInfo(): Promise<PhpMyAdminInfo> {
  return invoke<PhpMyAdminInfo>("phpmyadmin_get_info")
}

// SSL commands
export async function sslInstallMkcert(): Promise<void> {
  return invoke<void>("ssl_install_mkcert")
}

export async function sslIsMkcertInstalled(): Promise<boolean> {
  return invoke<boolean>("ssl_is_mkcert_installed")
}

export async function sslInstallCa(): Promise<void> {
  return invoke<void>("ssl_install_ca")
}

export async function sslIsCaInstalled(): Promise<boolean> {
  return invoke<boolean>("ssl_is_ca_installed")
}

export async function sslGenerateCertificate(domain: string): Promise<CertificateInfo> {
  return invoke<CertificateInfo>("ssl_generate_certificate", { domain })
}

export async function sslRemoveCertificate(domain: string): Promise<void> {
  return invoke<void>("ssl_remove_certificate", { domain })
}

export async function sslListCertificates(): Promise<CertificateInfo[]> {
  return invoke<CertificateInfo[]>("ssl_list_certificates")
}

export async function dnsAddEntry(domain: string, ip: string): Promise<void> {
  return invoke<void>("dns_add_entry", { domain, ip })
}

export async function dnsRemoveEntry(domain: string): Promise<void> {
  return invoke<void>("dns_remove_entry", { domain })
}

export async function dnsListEntries(): Promise<DnsEntry[]> {
  return invoke<DnsEntry[]>("dns_list_entries")
}

export async function dnsSetupResolver(tld: string): Promise<void> {
  return invoke<void>("dns_setup_resolver", { tld })
}

// Log commands
export async function logListFiles(): Promise<LogFile[]> {
  return invoke<LogFile[]>("log_list_files")
}

export async function logReadFile(path: string, lines?: number): Promise<string[]> {
  return invoke<string[]>("log_read_file", { path, lines })
}

export async function logStartTailing(path: string): Promise<void> {
  return invoke<void>("log_start_tailing", { path })
}

export async function logStopTailing(): Promise<void> {
  return invoke<void>("log_stop_tailing")
}

export async function logClearFile(path: string): Promise<void> {
  return invoke<void>("log_clear_file", { path })
}

// Settings commands
export async function settingsGet(): Promise<AppConfig> {
  return invoke<AppConfig>("settings_get")
}

export async function settingsSave(config: AppConfig): Promise<void> {
  return invoke<void>("settings_save", { config })
}

export async function settingsReset(): Promise<AppConfig> {
  return invoke<AppConfig>("settings_reset")
}
