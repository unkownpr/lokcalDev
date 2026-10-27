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

/// Like `is_process_alive`, but treats zombies (exited, not yet reaped) as gone.
/// Used when waiting for a process we may have spawned ourselves to exit.
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        if let Ok(o) = Command::new("ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .output()
        {
            let stat = String::from_utf8_lossy(&o.stdout);
            let stat = stat.trim();
            return !stat.is_empty() && !stat.starts_with('Z');
        }
    }
    is_process_alive(pid)
}

/// Poll until the process exits or the timeout elapses. Returns true if it exited.
pub fn wait_for_exit(pid: u32, timeout: std::time::Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if !is_process_running(pid) {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// Gracefully stop a process and wait for it to exit, so its port, socket and
/// data-dir locks are released before a restart. Force-kills after `timeout`.
pub fn stop_process(pid: u32, timeout: std::time::Duration) {
    kill_process(pid);
    if wait_for_exit(pid, timeout) {
        return;
    }
    log::warn!("Process {} did not exit within {:?}, force killing", pid, timeout);
    #[cfg(unix)]
    {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }
    wait_for_exit(pid, std::time::Duration::from_secs(2));
}

/// Run blocking work (brew, archive extraction, service restarts) on the blocking
/// thread pool so long installs don't pin an async runtime worker.
pub async fn run_blocking<T, F>(f: F) -> Result<T, AppError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Process(format!("Background task failed: {}", e)))
}

/// Run `detect` (which spawns `binary --version` or similar) only when `binary` changed
/// since the last successful call, keyed on its path and mtime. Status polling calls
/// this every few seconds; the version only changes on install/upgrade.
pub fn cached_version(binary: &Path, detect: impl FnOnce() -> Option<String>) -> Option<String> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use std::time::SystemTime;
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, String)>>> = OnceLock::new();

    let mtime = std::fs::metadata(binary).and_then(|m| m.modified()).ok();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let (Some(mtime), Ok(map)) = (mtime, cache.lock()) {
        if let Some((cached_mtime, version)) = map.get(binary) {
            if *cached_mtime == mtime {
                return Some(version.clone());
            }
        }
    }

    let version = detect()?;
    if let (Some(mtime), Ok(mut map)) = (mtime, cache.lock()) {
        map.insert(binary.to_path_buf(), (mtime, version.clone()));
    }
    Some(version)
}

/// Give a freshly spawned service `grace` to fail. If it already exited (bad config,
/// port in use, uninitialized data dir), return an error carrying the tail of its log
/// instead of reporting a dead PID as running.
pub fn ensure_still_running(
    child: &mut std::process::Child,
    name: &str,
    log_path: Option<&Path>,
    grace: std::time::Duration,
) -> Result<(), AppError> {
    let deadline = std::time::Instant::now() + grace;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            let tail = log_path
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|log| {
                    let lines: Vec<&str> = log.lines().collect();
                    lines[lines.len().saturating_sub(20)..].join("\n")
                })
                .unwrap_or_default();
            return Err(AppError::Process(format!(
                "{} exited immediately ({}){}{}",
                name,
                status,
                if tail.is_empty() { "" } else { ":\n" },
                tail
            )));
        }
        if std::time::Instant::now() >= deadline {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
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

/// Read a PID file and return the PID if the process is running.
/// Zombies count as stopped: a crashed child we still hold has not been reaped yet.
pub fn read_pid_file(pid_path: &Path) -> (bool, Option<u32>) {
    if pid_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let alive = is_process_running(pid);
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

/// Validate a hostname used in file names, nginx configs, hosts entries and
/// privileged shell commands. Allows only `[A-Za-z0-9.-]`, no empty labels and
/// no label starting or ending with `-`.
pub fn validate_domain(domain: &str) -> Result<(), AppError> {
    let valid = !domain.is_empty()
        && domain.len() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        });
    if valid {
        Ok(())
    } else {
        Err(AppError::Config(format!("Invalid domain name: '{}'", domain)))
    }
}
