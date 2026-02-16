use crate::config::paths;
use crate::error::AppError;
use crate::services::download_manager::DownloadManager;
use crate::services::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MariaDbInfo {
    pub installed: bool,
    pub initialized: bool,
    pub running: bool,
    pub version: Option<String>,
    pub pid: Option<u32>,
    pub port: u16,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEntry {
    pub name: String,
}

pub struct MariaDbManager;

impl MariaDbManager {
    // ── macOS: Homebrew-based paths ──────────────────────────────────

    #[cfg(target_os = "macos")]
    fn get_mariadb_prefix() -> PathBuf {
        utils::get_brew_prefix().join("opt").join("mariadb")
    }

    #[cfg(target_os = "macos")]
    fn get_base_dir() -> PathBuf {
        Self::get_mariadb_prefix()
    }

    #[cfg(target_os = "macos")]
    fn get_mysqld_binary() -> PathBuf {
        Self::get_mariadb_prefix().join("bin").join("mysqld")
    }

    #[cfg(target_os = "macos")]
    fn get_mysql_binary() -> PathBuf {
        Self::get_mariadb_prefix().join("bin").join("mysql")
    }

    #[cfg(target_os = "macos")]
    fn get_install_db_binary() -> PathBuf {
        Self::get_mariadb_prefix().join("bin").join("mariadb-install-db")
    }

    // ── Windows: Direct download paths ──────────────────────────────

    #[cfg(target_os = "windows")]
    fn get_base_dir() -> PathBuf {
        paths::get_mariadb_dir()
    }

    #[cfg(target_os = "windows")]
    fn get_mysqld_binary() -> PathBuf {
        Self::get_base_dir().join("bin").join("mysqld.exe")
    }

    #[cfg(target_os = "windows")]
    fn get_mysql_binary() -> PathBuf {
        Self::get_base_dir().join("bin").join("mysql.exe")
    }

    #[cfg(target_os = "windows")]
    fn get_install_db_binary() -> PathBuf {
        Self::get_base_dir().join("bin").join("mariadb-install-db.exe")
    }

    // ── Fallback for other platforms ────────────────────────────────

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_base_dir() -> PathBuf {
        paths::get_mariadb_dir()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_mysqld_binary() -> PathBuf {
        Self::get_base_dir().join("bin").join("mysqld")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_mysql_binary() -> PathBuf {
        Self::get_base_dir().join("bin").join("mysql")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_install_db_binary() -> PathBuf {
        Self::get_base_dir().join("scripts").join("mysql_install_db")
    }

    // ── Common paths ────────────────────────────────────────────────

    fn get_data_dir() -> PathBuf {
        paths::get_data_dir().join("data").join("mariadb")
    }

    fn get_pid_path() -> PathBuf {
        paths::get_data_dir().join("data").join("mariadb.pid")
    }

    #[cfg(not(target_os = "windows"))]
    fn get_socket_path() -> PathBuf {
        paths::get_data_dir().join("data").join("mariadb.sock")
    }

    fn get_log_path() -> PathBuf {
        paths::get_logs_dir().join("mariadb.log")
    }

    fn get_connection_args() -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            vec![
                "--host=127.0.0.1".to_string(),
                "--port=3306".to_string(),
            ]
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![
                format!("--socket={}", Self::get_socket_path().display()),
            ]
        }
    }

    // ── Helpers ─────────────────────────────────────────────────────

    fn emit_progress(app: &AppHandle, status: &str, message: &str) {
        let _ = app.emit(
            "download-progress",
            DownloadManager::progress(
                "mariadb",
                0,
                None,
                if status == "completed" || status == "extracting" { 100.0 } else { 0.0 },
                status,
                Some(message.to_string()),
            ),
        );
    }

    fn detect_version() -> Option<String> {
        let mysqld = Self::get_mysqld_binary();
        if !mysqld.exists() {
            return None;
        }
        Command::new(&mysqld)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                // "mysqld  Ver 11.4.10-MariaDB for osx10.19 on arm64 (Homebrew)"
                for word in stdout.split_whitespace() {
                    if word.contains("-MariaDB") {
                        return Some(word.replace("-MariaDB", ""));
                    }
                }
                None
            })
    }

    // ── Install (macOS — Homebrew) ──────────────────────────────────

    #[cfg(target_os = "macos")]
    pub async fn install(app: &AppHandle) -> Result<MariaDbInfo, AppError> {
        if Self::get_mysqld_binary().exists() {
            return Err(AppError::Service("MariaDB is already installed".to_string()));
        }

        utils::ensure_homebrew()?;

        Self::emit_progress(app, "extracting", "Installing MariaDB via Homebrew...");

        let install = Command::new("brew")
            .args(["install", "mariadb"])
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
        if !Self::get_mysqld_binary().exists() {
            let stderr = String::from_utf8_lossy(&install.stderr);
            let stdout = String::from_utf8_lossy(&install.stdout);
            let msg = format!(
                "MariaDB installation failed: {}",
                format!("{} {}", stdout, stderr).chars().take(500).collect::<String>()
            );
            Self::emit_progress(app, "failed", &msg);
            return Err(AppError::Process(msg));
        }

        Self::emit_progress(app, "completed", "MariaDB installed");
        Ok(Self::get_info())
    }

    // ── Install (Windows — direct download) ─────────────────────────

    #[cfg(target_os = "windows")]
    pub async fn install(app: &AppHandle) -> Result<MariaDbInfo, AppError> {
        let base_dir = Self::get_base_dir();
        if Self::get_mysqld_binary().exists() {
            return Err(AppError::Service("MariaDB is already installed".to_string()));
        }

        let version = "11.4.10";
        let url = format!(
            "https://archive.mariadb.org/mariadb-{}/winx64-packages/mariadb-{}-winx64.zip",
            version, version
        );
        let archive_path = base_dir.join("mariadb.zip");

        DownloadManager::download_file(app, "mariadb", &url, &archive_path).await?;
        DownloadManager::extract_zip(app, "mariadb", &archive_path, &base_dir)?;

        let _ = std::fs::remove_file(&archive_path);

        // Flatten: extracted zip creates mariadb-11.4.10-winx64/ subdirectory
        utils::flatten_extracted_dir(&base_dir, "mariadb-")?;

        Ok(Self::get_info())
    }

    // ── Install (fallback) ──────────────────────────────────────────

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub async fn install(_app: &AppHandle) -> Result<MariaDbInfo, AppError> {
        Err(AppError::Service("MariaDB install not supported on this platform yet".to_string()))
    }

    // ── get_info ────────────────────────────────────────────────────

    pub fn get_info() -> MariaDbInfo {
        let mysqld = Self::get_mysqld_binary();
        let installed = mysqld.exists();
        let data_dir = Self::get_data_dir();
        let initialized = data_dir.exists() && data_dir.join("mysql").exists();
        let pid_file = Self::get_pid_path();
        let (running, pid) = if installed {
            utils::read_pid_file(&pid_file)
        } else {
            (false, None)
        };

        MariaDbInfo {
            installed,
            initialized,
            running,
            version: if installed { Self::detect_version() } else { None },
            pid,
            port: 3306,
            data_dir: data_dir.to_string_lossy().to_string(),
        }
    }

    // ── initialize_db ───────────────────────────────────────────────

    pub fn initialize_db() -> Result<(), AppError> {
        let install_db = Self::get_install_db_binary();
        let base_dir = Self::get_base_dir();
        let data_dir = Self::get_data_dir();

        std::fs::create_dir_all(&data_dir)?;

        #[cfg(target_os = "windows")]
        let output = {
            Command::new(&install_db)
                .arg("--no-defaults")
                .arg(format!("--datadir={}", utils::to_forward_slash(&data_dir)))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| {
                    AppError::Process(format!("Failed to initialize MariaDB: {}", e))
                })?
        };

        #[cfg(not(target_os = "windows"))]
        let output = {
            Command::new(&install_db)
                .arg("--no-defaults")
                .arg(format!("--basedir={}", base_dir.display()))
                .arg(format!("--datadir={}", data_dir.display()))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| {
                    AppError::Process(format!("Failed to initialize MariaDB: {}", e))
                })?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_lower = stderr.to_lowercase();
            // Check if it really failed (not just warnings)
            if stderr_lower.contains("error") && !data_dir.join("mysql").exists() {
                return Err(AppError::Process(format!(
                    "mysql_install_db failed: {}",
                    stderr
                )));
            }
            log::warn!("mysql_install_db stderr (non-fatal): {}", stderr);
        }

        log::info!("MariaDB initialized in {}", data_dir.display());
        Ok(())
    }

    // ── start ───────────────────────────────────────────────────────

    pub fn start() -> Result<Child, AppError> {
        let mysqld = Self::get_mysqld_binary();
        if !mysqld.exists() {
            return Err(AppError::NotFound("MariaDB is not installed".to_string()));
        }

        let base_dir = Self::get_base_dir();
        let data_dir = Self::get_data_dir();
        let pid_path = Self::get_pid_path();
        let log_path = Self::get_log_path();

        let mut cmd = Command::new(&mysqld);
        cmd.arg("--no-defaults")
            .arg(format!("--basedir={}", utils::to_forward_slash(&base_dir)))
            .arg(format!("--datadir={}", utils::to_forward_slash(&data_dir)))
            .arg(format!("--pid-file={}", utils::to_forward_slash(&pid_path)))
            .arg(format!("--log-error={}", utils::to_forward_slash(&log_path)))
            .arg("--port=3306")
            .arg("--bind-address=127.0.0.1")
            .arg("--skip-grant-tables")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(not(target_os = "windows"))]
        {
            let socket_path = Self::get_socket_path();
            cmd.arg(format!("--socket={}", socket_path.display()));
        }

        #[cfg(target_os = "windows")]
        {
            cmd.arg("--skip-named-pipe");
        }

        let child = cmd.spawn()
            .map_err(|e| AppError::Process(format!("Failed to start MariaDB: {}", e)))?;

        let pid = child.id();
        let _ = std::fs::write(&pid_path, pid.to_string());

        log::info!("Started MariaDB (PID: {})", pid);
        Ok(child)
    }

    // ── stop ────────────────────────────────────────────────────────

    pub fn stop() -> Result<(), AppError> {
        let pid_path = Self::get_pid_path();
        if pid_path.exists() {
            if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    utils::kill_process(pid);
                }
            }
            let _ = std::fs::remove_file(&pid_path);
        }
        log::info!("Stopped MariaDB");
        Ok(())
    }

    // ── Database CRUD ───────────────────────────────────────────────

    pub fn list_databases() -> Result<Vec<DatabaseEntry>, AppError> {
        let mysql = Self::get_mysql_binary();
        if !mysql.exists() {
            return Err(AppError::NotFound("MariaDB is not installed".to_string()));
        }

        let conn_args = Self::get_connection_args();
        let mut cmd = Command::new(&mysql);
        cmd.args(["-u", "root"]);
        for arg in &conn_args {
            cmd.arg(arg);
        }
        cmd.args(["-e", "SHOW DATABASES;", "-N"]);

        let output = cmd.output()
            .map_err(|e| AppError::Process(format!("Failed to list databases: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Process(format!(
                "Failed to list databases: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let databases: Vec<DatabaseEntry> = stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| DatabaseEntry {
                name: l.trim().to_string(),
            })
            .collect();

        Ok(databases)
    }

    pub fn create_database(name: &str) -> Result<(), AppError> {
        let mysql = Self::get_mysql_binary();
        if !mysql.exists() {
            return Err(AppError::NotFound("MariaDB is not installed".to_string()));
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::Config(
                "Database name can only contain alphanumeric characters and underscores"
                    .to_string(),
            ));
        }

        let conn_args = Self::get_connection_args();
        let mut cmd = Command::new(&mysql);
        cmd.args(["-u", "root"]);
        for arg in &conn_args {
            cmd.arg(arg);
        }
        cmd.args(["-e", &format!("CREATE DATABASE `{}`;", name)]);

        let output = cmd.output()
            .map_err(|e| AppError::Process(format!("Failed to create database: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Process(format!(
                "Failed to create database: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("Created database: {}", name);
        Ok(())
    }

    pub fn drop_database(name: &str) -> Result<(), AppError> {
        let mysql = Self::get_mysql_binary();
        if !mysql.exists() {
            return Err(AppError::NotFound("MariaDB is not installed".to_string()));
        }

        // Validate database name to prevent SQL injection
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(AppError::Config(
                "Database name can only contain alphanumeric characters and underscores"
                    .to_string(),
            ));
        }

        let conn_args = Self::get_connection_args();
        let mut cmd = Command::new(&mysql);
        cmd.args(["-u", "root"]);
        for arg in &conn_args {
            cmd.arg(arg);
        }
        cmd.args(["-e", &format!("DROP DATABASE `{}`;", name)]);

        let output = cmd.output()
            .map_err(|e| AppError::Process(format!("Failed to drop database: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Process(format!(
                "Failed to drop database: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("Dropped database: {}", name);
        Ok(())
    }
}
