use crate::config::paths;
use crate::error::AppError;
use crate::services::download_manager::DownloadManager;
use crate::services::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tauri::{AppHandle, Emitter};

const PHP_VERSIONS: &[&str] = &["8.1", "8.2", "8.3", "8.4"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpVersion {
    pub version: String,
    pub installed: bool,
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpIniDirective {
    pub key: String,
    pub value: String,
    pub section: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpExtension {
    pub name: String,
    pub enabled: bool,
    pub builtin: bool,
}

pub struct PhpManager;

impl PhpManager {
    // --- macOS: Homebrew-based paths ---

    #[cfg(target_os = "macos")]
    fn get_php_prefix(version: &str) -> PathBuf {
        utils::get_brew_prefix()
            .join("opt")
            .join(format!("php@{}", version))
    }

    #[cfg(target_os = "macos")]
    fn get_php_binary(version: &str) -> PathBuf {
        Self::get_php_prefix(version).join("bin").join("php")
    }

    #[cfg(target_os = "macos")]
    fn get_php_fpm_binary(version: &str) -> PathBuf {
        Self::get_php_prefix(version).join("sbin").join("php-fpm")
    }

    #[cfg(target_os = "macos")]
    fn get_php_ini_path(version: &str) -> PathBuf {
        utils::get_brew_prefix()
            .join("etc")
            .join("php")
            .join(version)
            .join("php.ini")
    }

    #[cfg(target_os = "macos")]
    fn is_installed_via_brew(version: &str) -> bool {
        let php_bin = Self::get_php_binary(version);
        php_bin.exists()
    }

    // --- Windows: Direct binary download paths ---

    #[cfg(target_os = "windows")]
    fn get_version_dir(version: &str) -> PathBuf {
        paths::get_php_dir().join(version)
    }

    #[cfg(target_os = "windows")]
    fn get_php_binary(version: &str) -> PathBuf {
        Self::get_version_dir(version).join("php.exe")
    }

    #[cfg(target_os = "windows")]
    fn get_php_fpm_binary(version: &str) -> PathBuf {
        Self::get_version_dir(version).join("php-cgi.exe")
    }

    #[cfg(target_os = "windows")]
    fn get_php_ini_path(version: &str) -> PathBuf {
        Self::get_version_dir(version).join("php.ini")
    }

    // --- Fallback for other platforms ---

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_version_dir(version: &str) -> PathBuf {
        paths::get_php_dir().join(version)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_php_binary(version: &str) -> PathBuf {
        Self::get_version_dir(version).join("bin").join("php")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_php_fpm_binary(version: &str) -> PathBuf {
        Self::get_version_dir(version).join("sbin").join("php-fpm")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_php_ini_path(version: &str) -> PathBuf {
        Self::get_version_dir(version).join("lib").join("php.ini")
    }

    // --- Common paths ---

    fn get_fpm_pid_path(version: &str) -> PathBuf {
        paths::get_data_dir()
            .join("data")
            .join(format!("php-fpm-{}.pid", version))
    }

    fn get_fpm_log_path(version: &str) -> PathBuf {
        paths::get_logs_dir().join(format!("php-fpm-{}.log", version))
    }

    // --- Download URLs (Windows only) ---

    #[cfg(target_os = "windows")]
    fn get_download_url(version: &str) -> String {
        let full_version = match version {
            "8.1" => "8.1.31",
            "8.2" => "8.2.28",
            "8.3" => "8.3.16",
            "8.4" => "8.4.4",
            _ => "8.3.16",
        };
        format!(
            "https://windows.php.net/downloads/releases/php-{}-nts-Win32-vs17-x64.zip",
            full_version
        )
    }

    // --- emit helper ---

    fn emit_progress(app: &AppHandle, id: &str, status: &str, message: &str) {
        let _ = app.emit(
            "download-progress",
            DownloadManager::progress(id, 0, None, if status == "completed" || status == "extracting" { 100.0 } else { 0.0 }, status, Some(message.to_string())),
        );
    }

    // --- Core functions ---

    pub fn list_versions() -> Vec<PhpVersion> {
        PHP_VERSIONS
            .iter()
            .map(|v| {
                let installed = Self::get_php_binary(v).exists();
                let pid_file = Self::get_fpm_pid_path(v);
                let (running, pid) = if installed {
                    utils::read_pid_file(&pid_file)
                } else {
                    (false, None)
                };

                let path = if installed {
                    Self::get_php_binary(v)
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.to_string_lossy().to_string())
                } else {
                    None
                };

                PhpVersion {
                    version: v.to_string(),
                    installed,
                    running,
                    port: utils::php_version_to_port(v),
                    pid,
                    path,
                }
            })
            .collect()
    }

    // --- macOS: Install via Homebrew ---

    #[cfg(target_os = "macos")]
    pub async fn install_version(app: &AppHandle, version: &str) -> Result<PhpVersion, AppError> {
        if Self::is_installed_via_brew(version) {
            return Err(AppError::Service(format!(
                "PHP {} is already installed",
                version
            )));
        }

        utils::ensure_homebrew()?;

        let dl_id = format!("php-{}", version);

        // Tap shivammathur/php
        Self::emit_progress(app, &dl_id, "extracting", "Tapping shivammathur/php...");
        let tap = Command::new("brew")
            .args(["tap", "shivammathur/php"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AppError::Process(format!("Failed to tap: {}", e)))?;

        if !tap.status.success() {
            let stderr = String::from_utf8_lossy(&tap.stderr);
            if !stderr.contains("already tapped") {
                let msg = format!("brew tap failed: {}", stderr.chars().take(500).collect::<String>());
                Self::emit_progress(app, &dl_id, "failed", &msg);
                return Err(AppError::Process(msg));
            }
        }

        // Install PHP
        Self::emit_progress(app, &dl_id, "extracting", &format!("Installing PHP {} via Homebrew...", version));
        let formula = format!("shivammathur/php/php@{}", version);
        let install = Command::new("brew")
            .args(["install", &formula])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                let msg = format!("brew install failed: {}", e);
                Self::emit_progress(app, &dl_id, "failed", &msg);
                AppError::Process(msg)
            })?;

        // Check result by binary existence, not exit code
        // brew may return non-zero for warnings that don't prevent installation
        if !Self::get_php_binary(version).exists() {
            let stderr = String::from_utf8_lossy(&install.stderr);
            let stdout = String::from_utf8_lossy(&install.stdout);
            let msg = format!(
                "PHP {} installation failed: {}",
                version,
                format!("{} {}", stdout, stderr).chars().take(500).collect::<String>()
            );
            Self::emit_progress(app, &dl_id, "failed", &msg);
            return Err(AppError::Process(msg));
        }

        // Setup custom FPM pool config with our port
        Self::ensure_fpm_pool_config(version)?;

        Self::emit_progress(app, &dl_id, "completed", &format!("PHP {} installed", version));

        Ok(PhpVersion {
            version: version.to_string(),
            installed: true,
            running: false,
            port: utils::php_version_to_port(version),
            pid: None,
            path: Self::get_php_binary(version)
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_string_lossy().to_string()),
        })
    }

    // --- Windows: Install via direct download ---

    #[cfg(target_os = "windows")]
    pub async fn install_version(app: &AppHandle, version: &str) -> Result<PhpVersion, AppError> {
        let version_dir = Self::get_version_dir(version);
        if version_dir.exists() {
            return Err(AppError::Service(format!(
                "PHP {} is already installed",
                version
            )));
        }

        let url = Self::get_download_url(version);
        let archive_path = paths::get_php_dir().join(format!("php-{}.zip", version));

        DownloadManager::download_file(app, &format!("php-{}", version), &url, &archive_path).await?;
        DownloadManager::extract_zip(app, &format!("php-{}", version), &archive_path, &version_dir)?;

        let _ = std::fs::remove_file(&archive_path);
        Self::ensure_php_ini_default(version)?;

        Ok(PhpVersion {
            version: version.to_string(),
            installed: true,
            running: false,
            port: utils::php_version_to_port(version),
            pid: None,
            path: Some(version_dir.to_string_lossy().to_string()),
        })
    }

    // --- Fallback for other platforms ---

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub async fn install_version(_app: &AppHandle, _version: &str) -> Result<PhpVersion, AppError> {
        Err(AppError::Service("PHP install not supported on this platform yet".to_string()))
    }

    // --- Remove ---

    #[cfg(target_os = "macos")]
    pub fn remove_version(version: &str) -> Result<(), AppError> {
        let _ = Self::stop_fpm(version);
        let formula = format!("shivammathur/php/php@{}", version);
        let output = Command::new("brew")
            .args(["uninstall", &formula])
            .output()
            .map_err(|e| AppError::Process(format!("brew uninstall failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Process(format!("brew uninstall failed: {}", stderr)));
        }
        log::info!("Removed PHP {} via Homebrew", version);
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn remove_version(version: &str) -> Result<(), AppError> {
        let version_dir = Self::get_version_dir(version);
        if !version_dir.exists() {
            return Err(AppError::NotFound(format!(
                "PHP {} is not installed",
                version
            )));
        }
        let _ = Self::stop_fpm(version);
        std::fs::remove_dir_all(&version_dir)?;
        log::info!("Removed PHP {}", version);
        Ok(())
    }

    // --- FPM start/stop ---

    pub fn start_fpm(version: &str) -> Result<(Child, u32), AppError> {
        let fpm_bin = Self::get_php_fpm_binary(version);
        if !fpm_bin.exists() {
            return Err(AppError::NotFound(format!(
                "PHP-FPM binary not found for version {}. Path: {}",
                version,
                fpm_bin.display()
            )));
        }

        // Ensure we have a custom pool config with our port
        Self::ensure_fpm_pool_config(version)?;

        let lokcaldev_conf = paths::get_data_dir()
            .join("config")
            .join(format!("php-fpm-{}.conf", version));

        let child = Command::new(&fpm_bin)
            .arg("--fpm-config")
            .arg(&lokcaldev_conf)
            .arg("--nodaemonize")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                AppError::Process(format!("Failed to start PHP-FPM {}: {}", version, e))
            })?;

        let pid = child.id();

        // Write PID file
        let pid_path = Self::get_fpm_pid_path(version);
        let _ = std::fs::write(&pid_path, pid.to_string());

        log::info!(
            "Started PHP-FPM {} on port {} (PID: {})",
            version,
            utils::php_version_to_port(version),
            pid
        );
        Ok((child, pid))
    }

    pub fn stop_fpm(version: &str) -> Result<(), AppError> {
        let pid_path = Self::get_fpm_pid_path(version);
        if pid_path.exists() {
            if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    utils::kill_process(pid);
                }
            }
            let _ = std::fs::remove_file(&pid_path);
        }
        log::info!("Stopped PHP-FPM {}", version);
        Ok(())
    }

    // --- FPM config: LokcalDev's own config in data dir ---

    fn ensure_fpm_pool_config(version: &str) -> Result<(), AppError> {
        let config_dir = paths::get_data_dir().join("config");
        std::fs::create_dir_all(&config_dir)?;

        let conf_path = config_dir.join(format!("php-fpm-{}.conf", version));
        let port = utils::php_version_to_port(version);
        let pid_path = Self::get_fpm_pid_path(version);
        let log_path = Self::get_fpm_log_path(version);

        // Always regenerate to ensure correct port
        let content = format!(
            "[global]\n\
             pid = {}\n\
             error_log = {}\n\
             log_level = notice\n\
             daemonize = no\n\
             \n\
             [www]\n\
             listen = 127.0.0.1:{}\n\
             pm = dynamic\n\
             pm.max_children = 5\n\
             pm.start_servers = 2\n\
             pm.min_spare_servers = 1\n\
             pm.max_spare_servers = 3\n",
            pid_path.display(),
            log_path.display(),
            port,
        );
        std::fs::write(&conf_path, content)?;

        Ok(())
    }

    // --- php.ini fallback for Windows ---

    #[allow(dead_code)]
    fn ensure_php_ini_default(version: &str) -> Result<(), AppError> {
        let ini_path = Self::get_php_ini_path(version);
        if let Some(parent) = ini_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !ini_path.exists() {
            let content = concat!(
                "[PHP]\n",
                "engine = On\n",
                "short_open_tag = Off\n",
                "precision = 14\n",
                "output_buffering = 4096\n",
                "memory_limit = 256M\n",
                "error_reporting = E_ALL\n",
                "display_errors = On\n",
                "display_startup_errors = On\n",
                "log_errors = On\n",
                "post_max_size = 64M\n",
                "default_charset = \"UTF-8\"\n",
                "file_uploads = On\n",
                "upload_max_filesize = 64M\n",
                "max_file_uploads = 20\n",
                "date.timezone = UTC\n",
            );
            std::fs::write(&ini_path, content)?;
        }
        Ok(())
    }

    // --- INI read/write ---

    pub fn get_ini(version: &str) -> Result<Vec<PhpIniDirective>, AppError> {
        let ini_path = Self::get_php_ini_path(version);
        if !ini_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&ini_path)?;
        let mut directives = Vec::new();
        let mut current_section = "PHP".to_string();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                continue;
            }
            if let Some((key, value)) = trimmed.split_once('=') {
                directives.push(PhpIniDirective {
                    key: key.trim().to_string(),
                    value: value.trim().to_string(),
                    section: current_section.clone(),
                });
            }
        }

        Ok(directives)
    }

    pub fn set_ini_directive(version: &str, key: &str, value: &str) -> Result<(), AppError> {
        let ini_path = Self::get_php_ini_path(version);
        if !ini_path.exists() {
            return Err(AppError::NotFound(format!(
                "php.ini not found for PHP {}",
                version
            )));
        }

        let content = std::fs::read_to_string(&ini_path)?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut found = false;

        for line in &mut lines {
            let trimmed = line.trim();
            if trimmed.starts_with(';') || trimmed.is_empty() || trimmed.starts_with('[') {
                continue;
            }
            if let Some((k, _)) = trimmed.split_once('=') {
                if k.trim() == key {
                    *line = format!("{} = {}", key, value);
                    found = true;
                    break;
                }
            }
        }

        if !found {
            lines.push(format!("{} = {}", key, value));
        }

        std::fs::write(&ini_path, lines.join("\n") + "\n")?;
        Ok(())
    }

    // --- Extensions ---

    pub fn list_extensions(version: &str) -> Result<Vec<PhpExtension>, AppError> {
        let php_bin = Self::get_php_binary(version);
        if !php_bin.exists() {
            return Err(AppError::NotFound(format!(
                "PHP {} is not installed",
                version
            )));
        }

        let output = Command::new(&php_bin)
            .arg("-m")
            .output()
            .map_err(|e| AppError::Process(format!("Failed to list extensions: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut extensions = Vec::new();
        let mut in_zend = false;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "[PHP Modules]" {
                in_zend = false;
                continue;
            }
            if trimmed == "[Zend Modules]" {
                in_zend = true;
                continue;
            }
            extensions.push(PhpExtension {
                name: trimmed.to_string(),
                enabled: true,
                builtin: in_zend,
            });
        }

        extensions.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(extensions)
    }

    pub fn toggle_extension(
        version: &str,
        extension: &str,
        enable: bool,
    ) -> Result<(), AppError> {
        let ini_path = Self::get_php_ini_path(version);
        if !ini_path.exists() {
            return Err(AppError::NotFound(format!(
                "php.ini not found for PHP {}",
                version
            )));
        }

        let content = std::fs::read_to_string(&ini_path)?;
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let ext_line = format!("extension={}", extension);
        let disabled_line = format!(";extension={}", extension);

        let mut found = false;
        for line in &mut lines {
            let trimmed = line.trim();
            if trimmed == ext_line || trimmed == disabled_line {
                *line = if enable {
                    ext_line.clone()
                } else {
                    disabled_line.clone()
                };
                found = true;
                break;
            }
        }

        if !found {
            if enable {
                lines.push(ext_line);
            } else {
                lines.push(disabled_line);
            }
        }

        std::fs::write(&ini_path, lines.join("\n") + "\n")?;
        Ok(())
    }
}
