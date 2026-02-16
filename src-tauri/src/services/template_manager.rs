use crate::error::AppError;
use crate::services::download_manager::DownloadManager;
use crate::services::mariadb_manager::MariaDbManager;
use crate::services::site_manager::SiteManager;
use crate::services::utils;
use tauri::{AppHandle, Emitter};

pub struct TemplateManager;

impl TemplateManager {
    fn emit_progress(app: &AppHandle, site_id: &str, status: &str, message: &str) {
        let id = format!("template-{}", site_id);
        let _ = app.emit(
            "download-progress",
            DownloadManager::progress(
                &id,
                0,
                None,
                match status {
                    "completed" => 100.0,
                    "extracting" | "configuring" => 80.0,
                    _ => 0.0,
                },
                status,
                Some(message.to_string()),
            ),
        );
    }

    pub async fn setup(
        app: &AppHandle,
        site_id: &str,
        template: &str,
    ) -> Result<(), AppError> {
        // Mark as installing
        SiteManager::update_template_status(site_id, "installing")?;

        let result = match template {
            "wordpress" => Self::setup_wordpress(app, site_id).await,
            "laravel" => Self::setup_laravel(app, site_id).await,
            "fatfree" => Self::setup_fatfree(app, site_id).await,
            _ => Err(AppError::Config(format!("Unknown template: {}", template))),
        };

        match &result {
            Ok(()) => {
                SiteManager::update_template_status(site_id, "completed")?;
                Self::emit_progress(app, site_id, "completed", &format!("{} installed successfully", template));
            }
            Err(e) => {
                SiteManager::update_template_status(site_id, "failed")?;
                Self::emit_progress(app, site_id, "failed", &e.to_string());
            }
        }

        result
    }

    // ── WordPress ──────────────────────────────────────────────────

    async fn setup_wordpress(app: &AppHandle, site_id: &str) -> Result<(), AppError> {
        let site = SiteManager::get(site_id)?;
        let doc_root = std::path::PathBuf::from(&site.document_root);
        let download_id = format!("template-{}", site_id);

        // 1. Download WordPress
        Self::emit_progress(app, site_id, "downloading", "Downloading WordPress...");
        let archive_path = doc_root.join("wordpress.zip");
        DownloadManager::download_file(
            app,
            &download_id,
            "https://wordpress.org/latest.zip",
            &archive_path,
        )
        .await?;

        // 2. Extract
        Self::emit_progress(app, site_id, "extracting", "Extracting WordPress...");
        DownloadManager::extract_zip(app, &download_id, &archive_path, &doc_root)?;

        // 3. Delete archive
        let _ = std::fs::remove_file(&archive_path);

        // 4. Flatten wordpress/ subdirectory
        utils::flatten_extracted_dir(&doc_root,"wordpress")?;

        // 5. Auto-create MariaDB database (non-fatal)
        let db_name = Self::sanitize_db_name(&site.name);
        Self::emit_progress(app, site_id, "configuring", "Creating database...");
        if let Err(e) = MariaDbManager::create_database(&db_name) {
            log::warn!("Could not auto-create WordPress database '{}': {}", db_name, e);
        }

        // 6. Generate wp-config.php
        Self::emit_progress(app, site_id, "configuring", "Generating wp-config.php...");
        Self::generate_wp_config(&doc_root, &db_name)?;

        log::info!("WordPress installed for site {}", site_id);
        Ok(())
    }

    fn generate_wp_config(
        doc_root: &std::path::Path,
        db_name: &str,
    ) -> Result<(), AppError> {
        let salts = Self::generate_wp_salts();
        let config = format!(
            r#"<?php
define( 'DB_NAME', '{db_name}' );
define( 'DB_USER', 'root' );
define( 'DB_PASSWORD', '' );
define( 'DB_HOST', '127.0.0.1:3306' );
define( 'DB_CHARSET', 'utf8mb4' );
define( 'DB_COLLATE', '' );

{salts}

$table_prefix = 'wp_';

define( 'WP_DEBUG', true );

if ( ! defined( 'ABSPATH' ) ) {{
    define( 'ABSPATH', __DIR__ . '/' );
}}

require_once ABSPATH . 'wp-settings.php';
"#,
            db_name = db_name,
            salts = salts,
        );

        std::fs::write(doc_root.join("wp-config.php"), config)?;
        Ok(())
    }

    fn generate_wp_salts() -> String {
        let keys = [
            "AUTH_KEY",
            "SECURE_AUTH_KEY",
            "LOGGED_IN_KEY",
            "NONCE_KEY",
            "AUTH_SALT",
            "SECURE_AUTH_SALT",
            "LOGGED_IN_SALT",
            "NONCE_SALT",
        ];

        keys.iter()
            .map(|key| {
                let salt = Self::random_string(64);
                format!("define( '{}', '{}' );", key, salt)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn random_string(len: usize) -> String {
        use std::time::SystemTime;
        let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";
        let mut result = String::with_capacity(len);
        for i in 0..len {
            let seed = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
                .wrapping_add(i as u32)
                .wrapping_mul(2654435761); // Knuth multiplicative hash
            let idx = (seed as usize) % chars.len();
            result.push(chars[idx] as char);
        }
        result
    }

    fn sanitize_db_name(name: &str) -> String {
        let sanitized: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        // Ensure it starts with a letter and isn't empty
        let sanitized = sanitized.trim_start_matches('_').to_string();
        if sanitized.is_empty() {
            "wordpress".to_string()
        } else {
            format!("wp_{}", sanitized)
        }
    }

    // ── Laravel ────────────────────────────────────────────────────

    async fn setup_laravel(app: &AppHandle, site_id: &str) -> Result<(), AppError> {
        let site = SiteManager::get(site_id)?;
        let doc_root = std::path::PathBuf::from(&site.document_root);
        let download_id = format!("template-{}", site_id);

        // 1. Download Laravel skeleton
        Self::emit_progress(app, site_id, "downloading", "Downloading Laravel...");
        let archive_path = doc_root.join("laravel.zip");
        DownloadManager::download_file(
            app,
            &download_id,
            "https://github.com/laravel/laravel/archive/refs/heads/master.zip",
            &archive_path,
        )
        .await?;

        // 2. Extract
        Self::emit_progress(app, site_id, "extracting", "Extracting Laravel...");
        DownloadManager::extract_zip(app, &download_id, &archive_path, &doc_root)?;

        // 3. Delete archive
        let _ = std::fs::remove_file(&archive_path);

        // 4. Flatten laravel-master/ subdirectory
        utils::flatten_extracted_dir(&doc_root,"laravel-")?;

        // 5. Ensure public/ directory exists
        let public_dir = doc_root.join("public");
        std::fs::create_dir_all(&public_dir)?;

        // 6. Copy .env.example → .env if present
        Self::emit_progress(app, site_id, "configuring", "Configuring Laravel...");
        let env_example = doc_root.join(".env.example");
        let env_file = doc_root.join(".env");
        if env_example.exists() && !env_file.exists() {
            std::fs::copy(&env_example, &env_file)?;
        }

        log::info!("Laravel installed for site {}", site_id);
        Ok(())
    }

    // ── Fat-Free Framework ──────────────────────────────────────────

    async fn setup_fatfree(app: &AppHandle, site_id: &str) -> Result<(), AppError> {
        let site = SiteManager::get(site_id)?;
        let doc_root = std::path::PathBuf::from(&site.document_root);
        let download_id = format!("template-{}", site_id);

        // 1. Download Fat-Free Framework
        Self::emit_progress(app, site_id, "downloading", "Downloading Fat-Free Framework...");
        let archive_path = doc_root.join("fatfree.zip");
        DownloadManager::download_file(
            app,
            &download_id,
            "https://github.com/bcosca/fatfree/archive/refs/heads/master.zip",
            &archive_path,
        )
        .await?;

        // 2. Extract
        Self::emit_progress(app, site_id, "extracting", "Extracting Fat-Free Framework...");
        DownloadManager::extract_zip(app, &download_id, &archive_path, &doc_root)?;

        // 3. Delete archive
        let _ = std::fs::remove_file(&archive_path);

        // 4. Flatten fatfree-master/ subdirectory
        utils::flatten_extracted_dir(&doc_root,"fatfree-")?;

        // 5. Create tmp/ directory (F3 uses it for cache, sessions, etc.)
        Self::emit_progress(app, site_id, "configuring", "Configuring Fat-Free Framework...");
        let tmp_dir = doc_root.join("tmp");
        std::fs::create_dir_all(&tmp_dir)?;

        // 6. Create a basic index.php entry point if not present
        let index_php = doc_root.join("index.php");
        if !index_php.exists() {
            let starter = r#"<?php
$f3 = require('lib/base.php');

$f3->set('DEBUG', 3);
$f3->set('UI', 'ui/');
$f3->set('TEMP', 'tmp/');

$f3->route('GET /', function() {
    echo '<h1>Fat-Free Framework</h1><p>Your F3 app is running.</p>';
});

$f3->run();
"#;
            std::fs::write(&index_php, starter)?;
        }

        log::info!("Fat-Free Framework installed for site {}", site_id);
        Ok(())
    }

}
