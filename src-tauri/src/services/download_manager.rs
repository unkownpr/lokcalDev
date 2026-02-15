use crate::error::AppError;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::Archive;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub id: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: f64,
    pub status: String,
    pub message: Option<String>,
}

pub struct DownloadManager;

impl DownloadManager {
    pub fn progress(id: &str, downloaded: u64, total: Option<u64>, percent: f64, status: &str, message: Option<String>) -> DownloadProgress {
        DownloadProgress {
            id: id.to_string(),
            downloaded,
            total,
            percent,
            status: status.to_string(),
            message,
        }
    }

    fn emit_progress(app: &AppHandle, id: &str, downloaded: u64, total: Option<u64>, percent: f64, status: &str, message: Option<String>) {
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                id: id.to_string(),
                downloaded,
                total,
                percent,
                status: status.to_string(),
                message,
            },
        );
    }

    pub async fn download_file(
        app: &AppHandle,
        id: &str,
        url: &str,
        dest: &Path,
    ) -> Result<PathBuf, AppError> {
        log::info!("Downloading {} to {}", url, dest.display());

        // Emit "starting" so the UI shows immediately
        Self::emit_progress(app, id, 0, None, 0.0, "downloading", Some("Connecting...".to_string()));

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let client = reqwest::Client::builder()
            .user_agent("LokcalDev/0.1")
            .build()
            .map_err(|e| {
                let msg = format!("Failed to create HTTP client: {}", e);
                Self::emit_progress(app, id, 0, None, 0.0, "failed", Some(msg.clone()));
                AppError::Download(msg)
            })?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                let msg = format!("HTTP request failed: {}", e);
                log::error!("{}", msg);
                Self::emit_progress(app, id, 0, None, 0.0, "failed", Some(msg.clone()));
                AppError::Download(msg)
            })?;

        if !response.status().is_success() {
            let msg = format!("HTTP {} for {}", response.status(), url);
            log::error!("{}", msg);
            Self::emit_progress(app, id, 0, None, 0.0, "failed", Some(msg.clone()));
            return Err(AppError::Download(msg));
        }

        let total = response.content_length();
        let mut downloaded: u64 = 0;
        let mut file =
            std::fs::File::create(dest).map_err(|e| {
                let msg = format!("Failed to create file: {}", e);
                Self::emit_progress(app, id, 0, None, 0.0, "failed", Some(msg.clone()));
                AppError::Download(msg)
            })?;

        Self::emit_progress(app, id, 0, total, 0.0, "downloading", Some("Downloading...".to_string()));

        let mut stream = response.bytes_stream();
        let mut last_emit = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                let msg = format!("Stream error: {}", e);
                Self::emit_progress(app, id, downloaded, total, 0.0, "failed", Some(msg.clone()));
                AppError::Download(msg)
            })?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            // Throttle events to every 100ms to avoid flooding
            if last_emit.elapsed().as_millis() >= 100 || downloaded == total.unwrap_or(0) {
                let percent = total.map(|t| (downloaded as f64 / t as f64) * 100.0).unwrap_or(0.0);
                Self::emit_progress(app, id, downloaded, total, percent, "downloading", None);
                last_emit = std::time::Instant::now();
            }
        }

        Self::emit_progress(app, id, downloaded, total, 100.0, "completed", Some("Download complete".to_string()));

        log::info!("Download complete: {}", dest.display());
        Ok(dest.to_path_buf())
    }

    #[allow(dead_code)]
    pub fn extract_tar_gz(app: &AppHandle, id: &str, archive_path: &Path, dest_dir: &Path) -> Result<(), AppError> {
        log::info!(
            "Extracting {} to {}",
            archive_path.display(),
            dest_dir.display()
        );
        std::fs::create_dir_all(dest_dir)?;

        Self::emit_progress(app, id, 0, None, 100.0, "extracting", Some("Extracting archive...".to_string()));

        let file = std::fs::File::open(archive_path)?;
        let gz = GzDecoder::new(file);
        let mut archive = Archive::new(gz);
        archive.unpack(dest_dir)?;

        Self::emit_progress(app, id, 0, None, 100.0, "completed", Some("Installation complete".to_string()));

        log::info!("Extraction complete: {}", dest_dir.display());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn extract_zip(app: &AppHandle, id: &str, archive_path: &Path, dest_dir: &Path) -> Result<(), AppError> {
        log::info!(
            "Extracting ZIP {} to {}",
            archive_path.display(),
            dest_dir.display()
        );
        std::fs::create_dir_all(dest_dir)?;

        Self::emit_progress(app, id, 0, None, 100.0, "extracting", Some("Extracting archive...".to_string()));

        let file = std::fs::File::open(archive_path)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| AppError::Download(e.to_string()))?;

        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| AppError::Download(e.to_string()))?;

            let safe_name = match entry.enclosed_name() {
                Some(name) => name.to_owned(),
                None => continue, // skip entries with unsafe paths
            };
            let out_path = dest_dir.join(safe_name);

            if entry.is_dir() {
                std::fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&out_path)?;
                std::io::copy(&mut entry, &mut outfile)?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = entry.unix_mode() {
                        std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
                    }
                }
            }
        }

        Self::emit_progress(app, id, 0, None, 100.0, "completed", Some("Installation complete".to_string()));

        log::info!("ZIP extraction complete: {}", dest_dir.display());
        Ok(())
    }
}
