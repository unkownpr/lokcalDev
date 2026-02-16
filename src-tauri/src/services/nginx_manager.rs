use crate::config::paths;
use crate::error::AppError;
use crate::services::download_manager::DownloadManager;
use crate::services::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NginxInfo {
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub pid: Option<u32>,
    pub port: u16,
    pub config_path: String,
}

pub struct NginxManager;

impl NginxManager {
    // ── macOS: Homebrew-based paths ──────────────────────────────────

    #[cfg(target_os = "macos")]
    fn get_nginx_binary() -> PathBuf {
        utils::get_brew_prefix().join("opt").join("nginx").join("bin").join("nginx")
    }

    // ── Windows: Direct download paths ──────────────────────────────

    #[cfg(target_os = "windows")]
    fn get_nginx_binary() -> PathBuf {
        paths::get_nginx_dir().join("nginx.exe")
    }

    // ── Fallback ────────────────────────────────────────────────────

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_nginx_binary() -> PathBuf {
        paths::get_nginx_dir().join("sbin").join("nginx")
    }

    // ── Common paths ────────────────────────────────────────────────

    fn get_config_path() -> PathBuf {
        paths::get_nginx_config_dir().join("nginx.conf")
    }

    fn get_pid_path() -> PathBuf {
        paths::get_nginx_config_dir().join("nginx.pid")
    }

    fn get_log_dir() -> PathBuf {
        paths::get_logs_dir()
    }

    // ── Helpers ─────────────────────────────────────────────────────

    fn emit_progress(app: &AppHandle, status: &str, message: &str) {
        let _ = app.emit(
            "download-progress",
            DownloadManager::progress(
                "nginx",
                0,
                None,
                if status == "completed" || status == "extracting" { 100.0 } else { 0.0 },
                status,
                Some(message.to_string()),
            ),
        );
    }

    fn detect_version() -> Option<String> {
        let binary = Self::get_nginx_binary();
        if !binary.exists() {
            return None;
        }
        Command::new(&binary)
            .arg("-v")
            .output()
            .ok()
            .and_then(|o| {
                // nginx writes version to stderr: "nginx version: nginx/1.28.2"
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                stderr
                    .split('/')
                    .last()
                    .map(|v| v.trim().to_string())
            })
    }

    // ── Install (macOS — Homebrew) ──────────────────────────────────

    #[cfg(target_os = "macos")]
    pub async fn install(app: &AppHandle) -> Result<NginxInfo, AppError> {
        if Self::get_nginx_binary().exists() {
            return Err(AppError::Service("Nginx is already installed".to_string()));
        }

        utils::ensure_homebrew()?;

        Self::emit_progress(app, "extracting", "Installing Nginx via Homebrew...");

        let install = Command::new("brew")
            .args(["install", "nginx"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                let msg = format!("brew install failed: {}", e);
                Self::emit_progress(app, "failed", &msg);
                AppError::Process(msg)
            })?;

        // Check result by binary existence, not exit code
        // brew may return non-zero for warnings that don't prevent installation
        if !Self::get_nginx_binary().exists() {
            let stderr = String::from_utf8_lossy(&install.stderr);
            let stdout = String::from_utf8_lossy(&install.stdout);
            let msg = format!(
                "Nginx installation failed: {}",
                format!("{} {}", stdout, stderr).chars().take(500).collect::<String>()
            );
            Self::emit_progress(app, "failed", &msg);
            return Err(AppError::Process(msg));
        }

        // Generate default config
        Self::ensure_config()?;

        Self::emit_progress(app, "completed", "Nginx installed");
        Ok(Self::get_info())
    }

    // ── Install (Windows — direct download) ─────────────────────────

    #[cfg(target_os = "windows")]
    pub async fn install(app: &AppHandle) -> Result<NginxInfo, AppError> {
        let nginx_dir = paths::get_nginx_dir();
        if Self::get_nginx_binary().exists() {
            return Err(AppError::Service("Nginx is already installed".to_string()));
        }

        let version = "1.28.2";
        let url = format!("https://nginx.org/download/nginx-{}.zip", version);
        let archive_path = nginx_dir.join("nginx.zip");

        DownloadManager::download_file(app, "nginx", &url, &archive_path).await?;
        DownloadManager::extract_zip(app, "nginx", &archive_path, &nginx_dir)?;

        let _ = std::fs::remove_file(&archive_path);

        // Flatten: extracted zip creates nginx-1.28.2/ subdirectory
        utils::flatten_extracted_dir(&nginx_dir, "nginx-")?;

        // Generate default config
        Self::ensure_config()?;

        Ok(Self::get_info())
    }

    // ── Install (fallback) ──────────────────────────────────────────

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub async fn install(_app: &AppHandle) -> Result<NginxInfo, AppError> {
        Err(AppError::Service("Nginx install not supported on this platform yet".to_string()))
    }

    // ── get_info ────────────────────────────────────────────────────

    pub fn get_info() -> NginxInfo {
        let binary = Self::get_nginx_binary();
        let installed = binary.exists();
        let pid_file = Self::get_pid_path();
        let (running, pid) = if installed {
            utils::read_pid_file(&pid_file)
        } else {
            (false, None)
        };

        NginxInfo {
            installed,
            running,
            version: if installed { Self::detect_version() } else { None },
            pid,
            port: 80,
            config_path: Self::get_config_path().to_string_lossy().to_string(),
        }
    }

    // ── privileged command helper (macOS) ──────────────────────────

    #[cfg(target_os = "macos")]
    fn run_privileged(cmd: &str) -> Result<(), AppError> {
        let escaped = cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "do shell script \"{}\" with administrator privileges",
            escaped
        );
        let output = Command::new("osascript")
            .current_dir("/tmp")
            .args(["-e", &script])
            .output()
            .map_err(|e| AppError::Process(format!("Failed to run privileged command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Process(stderr.trim().to_string()));
        }
        Ok(())
    }

    /// Get current username for nginx user directive
    fn get_current_username() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "nobody".to_string())
    }

    /// Get current user's primary group
    fn get_current_groupname() -> String {
        #[cfg(unix)]
        {
            Command::new("id")
                .arg("-gn")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "staff".to_string())
        }
        #[cfg(not(unix))]
        {
            "nogroup".to_string()
        }
    }

    /// Quote a path for shell usage (handles spaces)
    fn shell_quote(path: &std::path::Path) -> String {
        format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
    }

    // ── start ───────────────────────────────────────────────────────

    pub fn start() -> Result<u32, AppError> {
        let binary = Self::get_nginx_binary();
        if !binary.exists() {
            return Err(AppError::NotFound("Nginx is not installed".to_string()));
        }

        // Always ensure config is up to date (adds user directive, PHP support, etc.)
        let config_changed = Self::ensure_config()?;

        // If nginx is already running, reload if config changed, then return existing PID
        let current = Self::get_info();
        if current.running {
            if config_changed {
                log::info!("Config updated, reloading nginx");
                let _ = Self::reload();
            }
            if let Some(pid) = current.pid {
                log::info!("Nginx is already running (PID: {})", pid);
                return Ok(pid);
            }
        }

        let config = Self::get_config_path();

        // On macOS, port 80 requires root — start as daemon with admin privileges
        #[cfg(target_os = "macos")]
        {
            let cmd = format!(
                "{} -c {}",
                Self::shell_quote(&binary),
                Self::shell_quote(&config)
            );
            Self::run_privileged(&cmd)?;
        }

        // On other platforms, start normally as daemon
        #[cfg(not(target_os = "macos"))]
        {
            let output = Command::new(&binary)
                .arg("-c")
                .arg(&config)
                .output()
                .map_err(|e| AppError::Process(format!("Failed to start Nginx: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Process(format!("Failed to start Nginx: {}", stderr)));
            }
        }

        // Wait for PID file
        std::thread::sleep(std::time::Duration::from_millis(500));

        let pid_path = Self::get_pid_path();
        let pid = std::fs::read_to_string(&pid_path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        log::info!("Started Nginx (PID: {})", pid);
        Ok(pid)
    }

    // ── stop ────────────────────────────────────────────────────────

    pub fn stop() -> Result<(), AppError> {
        let binary = Self::get_nginx_binary();
        let config = Self::get_config_path();

        if binary.exists() && config.exists() {
            let cmd = format!(
                "{} -c {} -s stop",
                Self::shell_quote(&binary),
                Self::shell_quote(&config)
            );

            #[cfg(target_os = "macos")]
            {
                // nginx master runs as root, need admin to stop
                let _ = Self::run_privileged(&cmd);
            }

            #[cfg(not(target_os = "macos"))]
            {
                let _ = Command::new(&binary)
                    .arg("-c")
                    .arg(&config)
                    .arg("-s")
                    .arg("stop")
                    .output();
            }
        }

        let pid_path = Self::get_pid_path();
        if pid_path.exists() {
            let _ = std::fs::remove_file(&pid_path);
        }

        log::info!("Stopped Nginx");
        Ok(())
    }

    // ── reload ──────────────────────────────────────────────────────

    pub fn reload() -> Result<(), AppError> {
        let binary = Self::get_nginx_binary();
        if !binary.exists() {
            return Err(AppError::NotFound("Nginx is not installed".to_string()));
        }
        let config = Self::get_config_path();

        let cmd = format!(
            "{} -c {} -s reload",
            Self::shell_quote(&binary),
            Self::shell_quote(&config)
        );

        #[cfg(target_os = "macos")]
        {
            Self::run_privileged(&cmd)?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            let output = Command::new(&binary)
                .arg("-c")
                .arg(&config)
                .arg("-s")
                .arg("reload")
                .output()
                .map_err(|e| AppError::Process(format!("Failed to reload Nginx: {}", e)))?;

            if !output.status.success() {
                return Err(AppError::Process(format!(
                    "Nginx reload failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        log::info!("Nginx reloaded");
        Ok(())
    }

    // ── test_config ─────────────────────────────────────────────────

    pub fn test_config() -> Result<String, AppError> {
        let binary = Self::get_nginx_binary();
        if !binary.exists() {
            return Err(AppError::NotFound("Nginx is not installed".to_string()));
        }
        let config = Self::get_config_path();
        let output = Command::new(&binary)
            .arg("-c")
            .arg(&config)
            .arg("-t")
            .output()
            .map_err(|e| AppError::Process(format!("Failed to test config: {}", e)))?;

        let result = String::from_utf8_lossy(&output.stderr).to_string();
        if output.status.success() {
            Ok(result)
        } else {
            Err(AppError::Config(result))
        }
    }

    // ── detect active PHP-FPM port ────────────────────────────────

    /// Find the port of the first running PHP-FPM, or the first installed PHP version.
    /// Falls back to 9081 (PHP 8.1) if nothing is found.
    fn detect_php_fpm_port() -> u16 {
        let versions = ["8.1", "8.2", "8.3", "8.4"];

        // First, try to find a running PHP-FPM
        for v in &versions {
            let pid_path = paths::get_data_dir()
                .join("data")
                .join(format!("php-fpm-{}.pid", v));
            let (alive, _) = utils::read_pid_file(&pid_path);
            if alive {
                return utils::php_version_to_port(v);
            }
        }

        // Fallback: first installed PHP version (check Homebrew on macOS)
        #[cfg(target_os = "macos")]
        {
            let brew_prefix = utils::get_brew_prefix();
            for v in &versions {
                let php_bin = brew_prefix
                    .join("opt")
                    .join(format!("php@{}", v))
                    .join("bin")
                    .join("php");
                if php_bin.exists() {
                    return utils::php_version_to_port(v);
                }
            }
        }

        // Default fallback
        9081
    }

    // ── phpMyAdmin location block ──────────────────────────────────

    fn get_phpmyadmin_location_block(php_port: u16) -> String {
        let pma_dir = paths::get_phpmyadmin_dir();
        if !pma_dir.join("index.php").exists() {
            return String::new();
        }
        // Use root (parent of phpmyadmin dir) instead of alias to avoid
        // nginx's known issue with alias + nested PHP location blocks
        let pma_root = utils::to_forward_slash(
            &pma_dir.parent().unwrap_or(&pma_dir).to_path_buf(),
        );
        let fastcgi_params = utils::to_forward_slash(&paths::get_nginx_config_dir().join("fastcgi_params"));
        format!(
            r#"

        location ^~ /phpmyadmin {{
            root "{}";
            index index.php;

            location ~ \.php$ {{
                root "{}";
                fastcgi_pass 127.0.0.1:{};
                fastcgi_index index.php;
                include "{}";
            }}
        }}"#,
            pma_root, pma_root, php_port, fastcgi_params,
        )
    }

    // ── ensure_config ───────────────────────────────────────────────

    /// Ensures nginx.conf is present and up-to-date.
    /// Returns `true` if the config was (re)written.
    fn ensure_config() -> Result<bool, AppError> {
        let config_dir = paths::get_nginx_config_dir();
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(config_dir.join("sites-enabled"))?;

        let config_path = Self::get_config_path();
        // Regenerate config if missing, lacks PHP support, lacks user directive,
        // or phpMyAdmin installation state changed
        let pma_installed = paths::get_phpmyadmin_dir().join("index.php").exists();
        let active_php_port = Self::detect_php_fpm_port();
        let needs_config = if config_path.exists() {
            let existing = std::fs::read_to_string(&config_path).unwrap_or_default();
            let has_pma = existing.contains("^~ /phpmyadmin");
            let current_port_str = format!("fastcgi_pass 127.0.0.1:{}", active_php_port);
            !existing.contains("fastcgi_pass")
                || !existing.contains("\nuser ")
                || pma_installed != has_pma
                || !existing.contains(&current_port_str)
        } else {
            true
        };
        if needs_config {
            let log_dir = Self::get_log_dir();
            let pid_path = Self::get_pid_path();
            let www_dir = paths::get_data_dir().join("www");
            std::fs::create_dir_all(&www_dir)?;

            // Detect current user/group so nginx workers can access user directories
            let username = Self::get_current_username();
            let groupname = Self::get_current_groupname();

            // Detect active PHP-FPM port
            let php_port = Self::detect_php_fpm_port();

            let content = format!(
                r#"user {username} {groupname};
worker_processes auto;
pid "{pid}";
error_log "{error_log}" warn;

events {{
    worker_connections 256;
}}

http {{
    include       "{mime_types}";
    default_type  application/octet-stream;

    log_format  main  '$remote_addr - $remote_user [$time_local] "$request" '
                      '$status $body_bytes_sent "$http_referer" '
                      '"$http_user_agent"';

    access_log  "{access_log}"  main;

    sendfile        on;
    tcp_nopush      on;
    keepalive_timeout  65;
    gzip  on;

    include "{sites_enabled}/*.conf";

    server {{
        listen 80 default_server;
        server_name localhost;
        root "{default_root}";
        index index.php index.html index.htm;

        location / {{
            try_files $uri $uri/ /index.php?$query_string;
        }}

        location ~ \.php$ {{
            try_files $uri =404;
            fastcgi_pass 127.0.0.1:{php_port};
            fastcgi_index index.php;
            include "{fastcgi_params}";
        }}

        location ~ /\.ht {{
            deny all;
        }}{phpmyadmin_location}
    }}
}}
"#,
                username = username,
                groupname = groupname,
                php_port = php_port,
                phpmyadmin_location = Self::get_phpmyadmin_location_block(php_port),
                pid = utils::to_forward_slash(&pid_path),
                error_log = utils::to_forward_slash(&log_dir.join("nginx-error.log")),
                mime_types = utils::to_forward_slash(&config_dir.join("mime.types")),
                access_log = utils::to_forward_slash(&log_dir.join("nginx-access.log")),
                sites_enabled = utils::to_forward_slash(&config_dir.join("sites-enabled")),
                default_root = utils::to_forward_slash(&www_dir),
                fastcgi_params = utils::to_forward_slash(&config_dir.join("fastcgi_params")),
            );
            std::fs::write(&config_path, content)?;
        }

        // Write default index.php if www dir has no index file
        let www_dir = paths::get_data_dir().join("www");
        std::fs::create_dir_all(&www_dir)?;
        let index_php = www_dir.join("index.php");
        if !index_php.exists() {
            let index_content = include_str!("../../resources/index.php");
            std::fs::write(&index_php, index_content)?;
        }

        // Write mime.types if missing
        let mime_path = config_dir.join("mime.types");
        if !mime_path.exists() {
            let mime_content = include_str!("../../resources/mime.types");
            std::fs::write(&mime_path, mime_content).or_else(|_| {
                // Fallback minimal mime.types
                std::fs::write(&mime_path, "types {\n    text/html html htm;\n    text/css css;\n    application/javascript js;\n    application/json json;\n    image/png png;\n    image/jpeg jpg jpeg;\n    image/gif gif;\n    image/svg+xml svg;\n}")
            })?;
        }

        // Write fastcgi_params if missing
        let fastcgi_path = config_dir.join("fastcgi_params");
        if !fastcgi_path.exists() {
            let fastcgi_content = concat!(
                "fastcgi_param  QUERY_STRING       $query_string;\n",
                "fastcgi_param  REQUEST_METHOD     $request_method;\n",
                "fastcgi_param  CONTENT_TYPE       $content_type;\n",
                "fastcgi_param  CONTENT_LENGTH     $content_length;\n",
                "fastcgi_param  SCRIPT_NAME        $fastcgi_script_name;\n",
                "fastcgi_param  REQUEST_URI        $request_uri;\n",
                "fastcgi_param  DOCUMENT_URI       $document_uri;\n",
                "fastcgi_param  DOCUMENT_ROOT      $document_root;\n",
                "fastcgi_param  SERVER_PROTOCOL    $server_protocol;\n",
                "fastcgi_param  GATEWAY_INTERFACE  CGI/1.1;\n",
                "fastcgi_param  SERVER_SOFTWARE    nginx/$nginx_version;\n",
                "fastcgi_param  REMOTE_ADDR        $remote_addr;\n",
                "fastcgi_param  REMOTE_PORT        $remote_port;\n",
                "fastcgi_param  SERVER_ADDR        $server_addr;\n",
                "fastcgi_param  SERVER_PORT        $server_port;\n",
                "fastcgi_param  SERVER_NAME        $server_name;\n",
                "fastcgi_param  SCRIPT_FILENAME    $document_root$fastcgi_script_name;\n",
                "fastcgi_param  REDIRECT_STATUS    200;\n",
            );
            std::fs::write(&fastcgi_path, fastcgi_content)?;
        }

        Ok(needs_config)
    }
}
