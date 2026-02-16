mod commands;
mod config;
mod error;
mod services;
mod state;

use config::paths;
use services::mariadb_manager::MariaDbManager;
use services::nginx_manager::NginxManager;
use services::php_manager::PhpManager;
use state::AppState;
use tauri::image::Image;
use tauri::menu::{AboutMetadata, Menu, PredefinedMenuItem, Submenu};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = paths::get_data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    }

    let app_state = AppState::new(data_dir.to_string_lossy().to_string());

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .menu(|app| {
            let icon = Image::from_bytes(include_bytes!("../icons/128x128@2x.png")).ok();
            let about_metadata = AboutMetadata {
                name: Some("LokcalDev".into()),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                copyright: Some("\u{00A9} 2026 ssilistre.dev. All rights reserved.".into()),
                credits: Some("Local development environment manager".into()),
                icon,
                ..Default::default()
            };

            let app_menu = Submenu::with_items(
                app,
                "LokcalDev",
                true,
                &[
                    &PredefinedMenuItem::about(app, None, Some(about_metadata))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::services(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::hide_others(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;

            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;

            let view_menu = Submenu::with_items(
                app,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(app, None)?],
            )?;

            let window_menu = Submenu::with_items(
                app,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, None)?,
                    &PredefinedMenuItem::maximize(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::close_window(app, None)?,
                ],
            )?;

            Menu::with_items(app, &[&app_menu, &edit_menu, &view_menu, &window_menu])
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Service commands
            commands::service_commands::get_all_services,
            commands::service_commands::get_service,
            commands::service_commands::start_service,
            commands::service_commands::stop_service,
            commands::service_commands::restart_service,
            // System commands
            commands::system_commands::get_system_info,
            commands::system_commands::initialize_app,
            // PHP commands
            commands::php_commands::php_list_versions,
            commands::php_commands::php_install_version,
            commands::php_commands::php_remove_version,
            commands::php_commands::php_start_fpm,
            commands::php_commands::php_stop_fpm,
            commands::php_commands::php_restart_fpm,
            commands::php_commands::php_get_ini,
            commands::php_commands::php_set_ini_directive,
            commands::php_commands::php_list_extensions,
            commands::php_commands::php_toggle_extension,
            // Nginx commands
            commands::nginx_commands::nginx_install,
            commands::nginx_commands::nginx_get_info,
            commands::nginx_commands::nginx_start,
            commands::nginx_commands::nginx_stop,
            commands::nginx_commands::nginx_restart,
            commands::nginx_commands::nginx_reload,
            commands::nginx_commands::nginx_test_config,
            // Site commands
            commands::site_commands::site_list,
            commands::site_commands::site_get,
            commands::site_commands::site_create,
            commands::site_commands::site_update,
            commands::site_commands::site_delete,
            commands::site_commands::site_setup_template,
            // Database commands
            commands::database_commands::mariadb_install,
            commands::database_commands::mariadb_get_info,
            commands::database_commands::mariadb_initialize,
            commands::database_commands::mariadb_start,
            commands::database_commands::mariadb_stop,
            commands::database_commands::mariadb_restart,
            commands::database_commands::database_list,
            commands::database_commands::database_create,
            commands::database_commands::database_drop,
            // SSL commands
            commands::ssl_commands::ssl_install_mkcert,
            commands::ssl_commands::ssl_is_mkcert_installed,
            commands::ssl_commands::ssl_install_ca,
            commands::ssl_commands::ssl_is_ca_installed,
            commands::ssl_commands::ssl_generate_certificate,
            commands::ssl_commands::ssl_remove_certificate,
            commands::ssl_commands::ssl_list_certificates,
            commands::ssl_commands::dns_add_entry,
            commands::ssl_commands::dns_remove_entry,
            commands::ssl_commands::dns_list_entries,
            commands::ssl_commands::dns_get_resolver_status,
            commands::ssl_commands::dns_ensure_dnsmasq_running,
            commands::ssl_commands::dns_setup_resolver,
            // Log commands
            commands::log_commands::log_list_files,
            commands::log_commands::log_read_file,
            commands::log_commands::log_start_tailing,
            commands::log_commands::log_stop_tailing,
            commands::log_commands::log_clear_file,
            // Settings commands
            commands::settings_commands::settings_get,
            commands::settings_commands::settings_save,
            commands::settings_commands::settings_reset,
            // phpMyAdmin commands
            commands::phpmyadmin_commands::phpmyadmin_install,
            commands::phpmyadmin_commands::phpmyadmin_get_info,
            // AI commands
            commands::ai_commands::ai_fetch_models,
            commands::ai_commands::ai_chat,
            commands::ai_commands::ai_execute_tool,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            shutdown_services(app_handle);
        }
    });
}

fn shutdown_services(app_handle: &tauri::AppHandle) {
    // 1. Kill all tracked child processes
    let state = app_handle.state::<AppState>();
    if let Ok(mut children) = state.child_processes.lock() {
        for (name, child) in children.iter_mut() {
            log::info!("Shutting down child process: {}", name);
            let _ = child.kill();
            let _ = child.wait();
        }
        children.clear();
    }

    // 2. Safety net: stop services via their managers
    log::info!("Running service shutdown safety net...");
    let _ = NginxManager::stop();
    let _ = MariaDbManager::stop();

    // Stop all PHP-FPM versions
    for version in &["8.1", "8.2", "8.3", "8.4"] {
        let _ = PhpManager::stop_fpm(version);
    }

    log::info!("All services stopped");
}
