use crate::config::paths;
use crate::error::AppError;
use crate::services::download_manager::DownloadManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;

const MKCERT_VERSION: &str = "1.4.4";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateInfo {
    pub domain: String,
    pub cert_path: String,
    pub key_path: String,
    pub exists: bool,
}

pub struct SslManager;

impl SslManager {
    fn get_mkcert_binary() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            paths::get_mkcert_dir().join("mkcert.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            paths::get_mkcert_dir().join("mkcert")
        }
    }

    fn get_download_url() -> String {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            format!(
                "https://github.com/FiloSottile/mkcert/releases/download/v{}/mkcert-v{}-darwin-arm64",
                MKCERT_VERSION, MKCERT_VERSION
            )
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            format!(
                "https://github.com/FiloSottile/mkcert/releases/download/v{}/mkcert-v{}-darwin-amd64",
                MKCERT_VERSION, MKCERT_VERSION
            )
        }
        #[cfg(target_os = "windows")]
        {
            format!(
                "https://github.com/FiloSottile/mkcert/releases/download/v{}/mkcert-v{}-windows-amd64.exe",
                MKCERT_VERSION, MKCERT_VERSION
            )
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            format!(
                "https://github.com/FiloSottile/mkcert/releases/download/v{}/mkcert-v{}-linux-amd64",
                MKCERT_VERSION, MKCERT_VERSION
            )
        }
    }

    pub async fn install_mkcert(app: &AppHandle) -> Result<(), AppError> {
        let binary = Self::get_mkcert_binary();
        if binary.exists() {
            return Err(AppError::Service("mkcert is already installed".to_string()));
        }

        let url = Self::get_download_url();
        DownloadManager::download_file(app, "mkcert", &url, &binary).await?;

        // Set executable permission
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
        }

        log::info!("mkcert installed");
        Ok(())
    }

    pub fn is_mkcert_installed() -> bool {
        Self::get_mkcert_binary().exists()
    }

    pub fn install_ca() -> Result<(), AppError> {
        let binary = Self::get_mkcert_binary();
        if !binary.exists() {
            return Err(AppError::NotFound("mkcert is not installed".to_string()));
        }

        let ca_dir = paths::get_ssl_dir().join("ca");
        std::fs::create_dir_all(&ca_dir)?;

        let output = Command::new(&binary)
            .env("CAROOT", &ca_dir)
            .arg("-install")
            .output()
            .map_err(|e| AppError::Process(format!("Failed to install CA: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Process(format!(
                "CA install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("CA root certificate installed");
        Ok(())
    }

    pub fn is_ca_installed() -> bool {
        let ca_dir = paths::get_ssl_dir().join("ca");
        ca_dir.join("rootCA.pem").exists()
    }

    pub fn generate_certificate(domain: &str) -> Result<CertificateInfo, AppError> {
        let binary = Self::get_mkcert_binary();
        if !binary.exists() {
            return Err(AppError::NotFound("mkcert is not installed".to_string()));
        }

        let ssl_dir = paths::get_ssl_dir();
        let ca_dir = ssl_dir.join("ca");
        let cert_path = ssl_dir.join(format!("{}.pem", domain));
        let key_path = ssl_dir.join(format!("{}-key.pem", domain));

        let output = Command::new(&binary)
            .env("CAROOT", &ca_dir)
            .arg("-cert-file")
            .arg(&cert_path)
            .arg("-key-file")
            .arg(&key_path)
            .arg(domain)
            .output()
            .map_err(|e| {
                AppError::Process(format!("Failed to generate certificate: {}", e))
            })?;

        if !output.status.success() {
            return Err(AppError::Process(format!(
                "Certificate generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("Generated SSL certificate for {}", domain);
        Ok(CertificateInfo {
            domain: domain.to_string(),
            cert_path: cert_path.to_string_lossy().to_string(),
            key_path: key_path.to_string_lossy().to_string(),
            exists: true,
        })
    }

    pub fn remove_certificate(domain: &str) -> Result<(), AppError> {
        let ssl_dir = paths::get_ssl_dir();
        let cert_path = ssl_dir.join(format!("{}.pem", domain));
        let key_path = ssl_dir.join(format!("{}-key.pem", domain));

        if cert_path.exists() {
            std::fs::remove_file(&cert_path)?;
        }
        if key_path.exists() {
            std::fs::remove_file(&key_path)?;
        }

        log::info!("Removed SSL certificate for {}", domain);
        Ok(())
    }

    pub fn list_certificates() -> Result<Vec<CertificateInfo>, AppError> {
        let ssl_dir = paths::get_ssl_dir();
        let mut certs = Vec::new();

        if !ssl_dir.exists() {
            return Ok(certs);
        }

        for entry in std::fs::read_dir(&ssl_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".pem") && !name.ends_with("-key.pem") && !name.starts_with("rootCA") {
                    let domain = name.trim_end_matches(".pem").to_string();
                    let key_path = ssl_dir.join(format!("{}-key.pem", domain));
                    certs.push(CertificateInfo {
                        domain: domain.clone(),
                        cert_path: path.to_string_lossy().to_string(),
                        key_path: key_path.to_string_lossy().to_string(),
                        exists: key_path.exists(),
                    });
                }
            }
        }

        certs.sort_by(|a, b| a.domain.cmp(&b.domain));
        Ok(certs)
    }
}
