use crate::config::paths;
use crate::error::AppError;
use crate::state::AppState;
use serde::Serialize;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use tauri::{AppHandle, Emitter, State};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize)]
pub struct LogFile {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogLine {
    pub file: String,
    pub line: String,
}

/// Validate that the given path is inside the logs directory
fn validate_log_path(path: &str) -> Result<std::path::PathBuf, AppError> {
    let logs_dir = paths::get_logs_dir();
    let requested = std::path::Path::new(path);

    // Canonicalize the logs dir (it must exist)
    std::fs::create_dir_all(&logs_dir)?;
    let canonical_logs = std::fs::canonicalize(&logs_dir)?;

    // For the requested path, canonicalize the parent dir and append the filename
    // This handles both existing and not-yet-existing files
    let canonical_requested = if requested.exists() {
        std::fs::canonicalize(requested)?
    } else {
        // If file doesn't exist, check parent
        let parent = requested.parent().ok_or_else(|| {
            AppError::Config("Invalid log file path".to_string())
        })?;
        let canonical_parent = std::fs::canonicalize(parent).map_err(|_| {
            AppError::Config("Invalid log file path".to_string())
        })?;
        canonical_parent.join(requested.file_name().ok_or_else(|| {
            AppError::Config("Invalid log file path".to_string())
        })?)
    };

    if !canonical_requested.starts_with(&canonical_logs) {
        return Err(AppError::Config(
            "Access denied: path is outside the logs directory".to_string(),
        ));
    }

    Ok(canonical_requested)
}

#[tauri::command]
pub fn log_list_files() -> Result<Vec<LogFile>, AppError> {
    let logs_dir = paths::get_logs_dir();
    std::fs::create_dir_all(&logs_dir)?;

    let mut files = Vec::new();
    for entry in std::fs::read_dir(&logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".log") {
                    let metadata = std::fs::metadata(&path)?;
                    files.push(LogFile {
                        name: name.to_string(),
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                    });
                }
            }
        }
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

#[tauri::command]
pub fn log_read_file(path: String, lines: Option<usize>) -> Result<Vec<String>, AppError> {
    let validated_path = validate_log_path(&path)?;
    let file = std::fs::File::open(&validated_path)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

    let count = lines.unwrap_or(500);
    let start = if all_lines.len() > count {
        all_lines.len() - count
    } else {
        0
    };

    Ok(all_lines[start..].to_vec())
}

#[tauri::command]
pub async fn log_start_tailing(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    let validated_path = validate_log_path(&path)?;

    // Cancel any existing tailing task
    {
        let mut cancel_guard = state
            .log_tail_cancel
            .lock()
            .map_err(|e| AppError::Service(e.to_string()))?;
        if let Some(old_token) = cancel_guard.take() {
            old_token.cancel();
        }
        let token = CancellationToken::new();
        *cancel_guard = Some(token.clone());

        let file_name = validated_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let path_str = validated_path.to_string_lossy().to_string();

        tokio::spawn(async move {
            let mut last_pos = 0u64;

            loop {
                tokio::select! {
                    _ = token.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                        if let Ok(file) = std::fs::File::open(&path_str) {
                            if let Ok(metadata) = file.metadata() {
                                let current_size = metadata.len();
                                if current_size > last_pos {
                                    let mut reader = BufReader::new(file);
                                    if reader.seek(SeekFrom::Start(last_pos)).is_ok() {
                                        for line in reader.lines() {
                                            if let Ok(line) = line {
                                                let _ = app.emit(
                                                    "log-line",
                                                    LogLine {
                                                        file: file_name.clone(),
                                                        line,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    last_pos = current_size;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    Ok(())
}

#[tauri::command]
pub fn log_stop_tailing(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut cancel_guard = state
        .log_tail_cancel
        .lock()
        .map_err(|e| AppError::Service(e.to_string()))?;
    if let Some(token) = cancel_guard.take() {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub fn log_clear_file(path: String) -> Result<(), AppError> {
    let validated_path = validate_log_path(&path)?;
    std::fs::write(&validated_path, "")?;
    log::info!("Cleared log file: {}", validated_path.display());
    Ok(())
}
