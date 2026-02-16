use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::AppError;

/// Check if a process with the given PID is still alive.
///
/// On Unix, `kill -0` succeeds for processes we own, but returns EPERM for
/// root-owned processes (e.g. nginx on port 80). EPERM means the process
/// exists but we lack permission to signal it — it IS alive.
/// Only ESRCH ("No such process") means the process is truly gone.
pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let output = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output();
        match output {
            Ok(o) => {
                if o.status.success() {
                    return true;
                }
                // EPERM → process exists but owned by another user (e.g. root)
                let stderr = String::from_utf8_lossy(&o.stderr);
                stderr.contains("Operation not permitted")
            }
            Err(_) => false,
        }
    }
    #[cfg(not(unix))]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output();
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains(&format!("\"{}\"", pid))
            }
            Err(_) => false,
        }
    }
}

/// Send SIGTERM (unix) or taskkill (windows) to a process.
pub fn kill_process(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill").arg(pid.to_string()).output();
    }
    #[cfg(not(unix))]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output();
    }
}

/// Get the Homebrew prefix path (macOS only). Cached after first call.
#[cfg(target_os = "macos")]
pub fn get_brew_prefix() -> PathBuf {
    static BREW_PREFIX: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BREW_PREFIX
        .get_or_init(|| {
            let output = Command::new("brew")
                .arg("--prefix")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "/opt/homebrew".to_string());
            PathBuf::from(output)
        })
        .clone()
}

/// Convert a path to forward slashes (for config compatibility on Windows).
pub fn to_forward_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// After extracting an archive that creates a subdirectory
/// (e.g. nginx-1.28.2/), move all contents up to base_dir.
pub fn flatten_extracted_dir(base_dir: &Path, prefix: &str) -> Result<(), AppError> {
    let mut extracted_dir = None;
    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(prefix) {
                extracted_dir = Some(entry.path());
                break;
            }
        }
    }

    if let Some(sub_dir) = extracted_dir {
        for entry in std::fs::read_dir(&sub_dir)? {
            let entry = entry?;
            let dest = base_dir.join(entry.file_name());
            if !dest.exists() {
                std::fs::rename(entry.path(), &dest)?;
            }
        }
        let _ = std::fs::remove_dir_all(&sub_dir);
    }
    Ok(())
}

/// Check that Homebrew is installed (macOS only).
#[cfg(target_os = "macos")]
pub fn ensure_homebrew() -> Result<(), AppError> {
    let brew_check = Command::new("brew").arg("--version").output();
    if brew_check.is_err() || !brew_check.unwrap().status.success() {
        return Err(AppError::Service(
            "Homebrew is not installed. Please install Homebrew first: https://brew.sh".to_string(),
        ));
    }
    Ok(())
}

/// Read a PID file and return the PID if the process is alive.
pub fn read_pid_file(pid_path: &Path) -> (bool, Option<u32>) {
    if pid_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let alive = is_process_alive(pid);
                return (alive, if alive { Some(pid) } else { None });
            }
        }
    }
    (false, None)
}

/// Map PHP version string to its FPM port.
pub fn php_version_to_port(version: &str) -> u16 {
    match version {
        "8.1" => 9081,
        "8.2" => 9082,
        "8.3" => 9083,
        "8.4" => 9084,
        _ => 9085,
    }
}
