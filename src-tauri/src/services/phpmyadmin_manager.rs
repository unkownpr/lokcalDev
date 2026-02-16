use crate::config::paths;
use crate::error::AppError;
use crate::services::download_manager::DownloadManager;
use crate::services::nginx_config::NginxConfigGenerator;
use crate::services::nginx_manager::NginxManager;
use crate::services::site_manager::SiteManager;
use crate::services::utils;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const PHPMYADMIN_VERSION: &str = "5.2.2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpMyAdminInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub path: String,
}

pub struct PhpMyAdminManager;

impl PhpMyAdminManager {
    fn emit_progress(app: &AppHandle, status: &str, message: &str) {
        let _ = app.emit(
            "download-progress",
            DownloadManager::progress(
                "phpmyadmin",
                0,
                None,
                if status == "completed" || status == "extracting" { 100.0 } else { 0.0 },
                status,
                Some(message.to_string()),
            ),
        );
    }

    pub fn get_info() -> PhpMyAdminInfo {
        let pma_dir = paths::get_phpmyadmin_dir();
        let installed = pma_dir.join("index.php").exists();
        let version = if installed {
            Self::detect_version().or_else(|| Some(PHPMYADMIN_VERSION.to_string()))
        } else {
            None
        };

        PhpMyAdminInfo {
            installed,
            version,
            path: pma_dir.to_string_lossy().to_string(),
        }
    }

    fn detect_version() -> Option<String> {
        let version_file = paths::get_phpmyadmin_dir().join("VERSION");
        if version_file.exists() {
            std::fs::read_to_string(&version_file)
                .ok()
                .map(|v| v.trim().to_string())
        } else {
            None
        }
    }

    pub async fn install(app: &AppHandle) -> Result<PhpMyAdminInfo, AppError> {
        let pma_dir = paths::get_phpmyadmin_dir();

        if pma_dir.join("index.php").exists() {
            return Err(AppError::Service("phpMyAdmin is already installed".to_string()));
        }

        std::fs::create_dir_all(&pma_dir)?;

        let url = format!(
            "https://files.phpmyadmin.net/phpMyAdmin/{}/phpMyAdmin-{}-all-languages.zip",
            PHPMYADMIN_VERSION, PHPMYADMIN_VERSION
        );
        let archive_path = pma_dir.join("phpmyadmin.zip");

        Self::emit_progress(app, "downloading", "Downloading phpMyAdmin...");
        DownloadManager::download_file(app, "phpmyadmin", &url, &archive_path).await?;

        Self::emit_progress(app, "extracting", "Extracting phpMyAdmin...");
        DownloadManager::extract_zip(app, "phpmyadmin", &archive_path, &pma_dir)?;

        let _ = std::fs::remove_file(&archive_path);

        // Flatten: zip creates phpMyAdmin-5.2.2-all-languages/ subdirectory
        utils::flatten_extracted_dir(&pma_dir, "phpMyAdmin-")?;

        // Write config
        Self::create_config()?;

        // Regenerate nginx config to add /phpmyadmin location and reload
        Self::refresh_nginx()?;

        Self::emit_progress(app, "completed", "phpMyAdmin installed");
        log::info!("phpMyAdmin installed to {}", pma_dir.display());

        Ok(Self::get_info())
    }

    fn refresh_nginx() -> Result<(), AppError> {
        // Regenerate all site configs to include /phpmyadmin location
        if let Ok(sites) = SiteManager::list() {
            let ssl_dir = paths::get_ssl_dir();
            for site in &sites {
                if !site.active {
                    continue;
                }
                let php_port = utils::php_version_to_port(&site.php_version);
                let (ssl_cert, ssl_key) = if site.ssl {
                    (
                        Some(ssl_dir.join(format!("{}.pem", site.domain)).to_string_lossy().to_string()),
                        Some(ssl_dir.join(format!("{}-key.pem", site.domain)).to_string_lossy().to_string()),
                    )
                } else {
                    (None, None)
                };
                let config = NginxConfigGenerator::generate_site_config(
                    &site.domain,
                    &site.document_root,
                    php_port,
                    site.ssl,
                    ssl_cert.as_deref(),
                    ssl_key.as_deref(),
                );
                let _ = NginxConfigGenerator::write_site_config(&site.domain, &config);
            }
            log::info!("Regenerated {} site configs with phpMyAdmin location", sites.len());
        }

        // Restart nginx so ensure_config picks up phpMyAdmin in default server
        let nginx_info = NginxManager::get_info();
        if nginx_info.installed && nginx_info.running {
            let _ = NginxManager::stop();
            let _ = NginxManager::start();
            log::info!("Nginx restarted to include phpMyAdmin location");
        }
        Ok(())
    }

    fn create_config() -> Result<(), AppError> {
        let pma_dir = paths::get_phpmyadmin_dir();
        let config_path = pma_dir.join("config.inc.php");

        let socket_path = paths::get_data_dir().join("data").join("mariadb.sock");
        let socket_str = socket_path.to_string_lossy().replace('\\', "/");

        let blowfish_secret: String = (0..32)
            .map(|_| {
                let idx = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos()
                    % 62) as usize;
                let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                chars[idx % chars.len()] as char
            })
            .collect();

        let config_content = format!(
            r#"<?php
$cfg['blowfish_secret'] = '{blowfish_secret}';

$i = 0;
$i++;

$cfg['Servers'][$i]['auth_type'] = 'config';
$cfg['Servers'][$i]['user'] = 'root';
$cfg['Servers'][$i]['password'] = '';
$cfg['Servers'][$i]['AllowNoPassword'] = true;

/* Socket connection (Unix) or TCP (Windows) */
if (PHP_OS_FAMILY === 'Windows') {{
    $cfg['Servers'][$i]['host'] = '127.0.0.1';
    $cfg['Servers'][$i]['port'] = '3306';
}} else {{
    $cfg['Servers'][$i]['socket'] = '{socket_str}';
    $cfg['Servers'][$i]['host'] = 'localhost';
}}

$cfg['UploadDir'] = '';
$cfg['SaveDir'] = '';

$cfg['TempDir'] = __DIR__ . '/tmp';
"#,
            blowfish_secret = blowfish_secret,
            socket_str = socket_str,
        );

        std::fs::write(&config_path, config_content)?;

        // Create tmp dir for phpMyAdmin
        let tmp_dir = pma_dir.join("tmp");
        std::fs::create_dir_all(&tmp_dir)?;

        log::info!("phpMyAdmin config written to {}", config_path.display());
        Ok(())
    }
}
