pub mod backup;
pub mod crypto;
mod commands;
mod launchd;
mod scheduler;
pub mod types;

use commands::AppState;
use scheduler::SchedulerState;
use types::BackupConfig;
use tauri::{Emitter, Manager};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger (must be called before any logging)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("InLocker starting...");

    // Check for CLI mode BEFORE initializing GUI
    let args: Vec<String> = std::env::args().collect();
    log::info!("App started with args: {:?}", args);

    // Detect CLI mode for scheduled backup
    let cli_backup_config_id = if args.len() >= 3 && args[1] == "--backup" {
        Some(args[2].clone())
    } else {
        None
    };

    if let Some(ref config_id) = cli_backup_config_id {
        log::info!("Running in CLI mode for scheduled backup: {}", config_id);
    }

    // Normal GUI mode: Initialize scheduler (skip for CLI mode)
    let scheduler_state = if cli_backup_config_id.is_none() {
        tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(async {
                let state = SchedulerState::new()
                    .await
                    .expect("Failed to create scheduler");
                state.start().await.expect("Failed to start scheduler");
                state
            })
    } else {
        // CLI mode: create minimal scheduler state
        tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(async {
                SchedulerState::new()
                    .await
                    .expect("Failed to create scheduler")
            })
    };

    tauri::Builder::default()
        // IMPORTANTE: single-instance DEVE ser o PRIMEIRO plugin (ordem importa!)
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Se usuário tentar abrir segunda instância, foca janela principal existente
            log::info!("Single instance detected, focusing existing main window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .manage(scheduler_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::select_folder,
            commands::select_file,
            commands::save_config,
            commands::load_configs,
            commands::delete_config,
            commands::run_backup_now,
            commands::cancel_backup,
            commands::cancel_restore,
            commands::register_schedule,
            commands::unregister_schedule,
            commands::check_schedule_status,
            commands::diagnose_schedule,
            commands::test_schedule_now,
            commands::is_scheduled_mode,
            commands::open_schedule_logs,
            commands::list_available_backups,
            commands::restore_backup,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // CLI mode: run backup with progress UI
            if let Some(config_id) = cli_backup_config_id {
                // Show SCHEDULED PROGRESS window (NOT main window)
                if let Some(window) = app.get_webview_window("scheduled-progress") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    log::info!("Opened scheduled-progress window for backup: {}", config_id);
                } else {
                    log::error!("Failed to get scheduled-progress window!");
                }

                tauri::async_runtime::spawn(async move {
                    log::info!("Executing scheduled backup for config: {}", config_id);
                    match run_scheduled_backup(&app_handle, &config_id).await {
                        Ok(_) => {
                            log::info!("Scheduled backup completed successfully");
                            std::process::exit(0);
                        }
                        Err(e) => {
                            log::error!("Scheduled backup failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                });
            } else {
                // Normal app startup - Show MAIN window and re-register schedules
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    log::info!("Opened main window for normal app usage");
                } else {
                    log::error!("Failed to get main window!");
                }

                tauri::async_runtime::spawn(async move {
                    if let Err(e) = restore_schedules(app_handle).await {
                        log::error!("Failed to restore schedules: {}", e);
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Run a scheduled backup (triggered by launchd via CLI args)
async fn run_scheduled_backup(app: &tauri::AppHandle, config_id: &str) -> Result<(), String> {
    use tauri::Manager;

    // Load configs from file (app may have been started headless)
    let config_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("configs.json");

    let config = if config_path.exists() {
        let json = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read configs: {}", e))?;
        let configs: Vec<BackupConfig> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse configs: {}", e))?;

        configs
            .into_iter()
            .find(|c| c.id == config_id)
            .ok_or(format!("Config not found: {}", config_id))?
    } else {
        return Err("No configs file found".to_string());
    };

    log::info!("Running backup for: {}", config.name);

    // Emit progress event: Initializing
    let _ = app.emit("backup-progress", serde_json::json!({
        "stage": "initializing",
        "message": format!("Inicializando backup: {}", config.name),
        "percentage": 0
    }));

    // Send notification: Backup started
    send_notification(
        app,
        "InLocker - Backup Started",
        &format!("Starting backup: {}", config.name),
    );

    // Emit progress event: Scanning
    let _ = app.emit("backup-progress", serde_json::json!({
        "stage": "scanning",
        "message": "Escaneando arquivos...",
        "percentage": 10
    }));

    // Execute backup using the backup module directly
    use crate::backup;
    use crate::types::{BackupManifest, BackupType};
    use std::path::Path;

    let source_path = Path::new(&config.source_path);
    let dest_path = Path::new(&config.destination_path);

    // Load previous manifest for incremental backup
    let manifest_path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join(format!("manifest_{}.json", config_id));

    let previous_manifest = if manifest_path.exists() && config.backup_type == BackupType::Incremental {
        let json = std::fs::read_to_string(&manifest_path).ok();
        json.and_then(|j| serde_json::from_str::<BackupManifest>(&j).ok())
    } else {
        None
    };

    // Emit progress event: Compressing
    let _ = app.emit("backup-progress", serde_json::json!({
        "stage": "compressing",
        "message": "Comprimindo arquivos...",
        "percentage": 30
    }));

    // Perform backup
    // TODO: Add password parameter when CLI supports it
    match backup::compress_folder(
        &config_id,
        source_path,
        dest_path,
        &config.backup_type,
        &config.mode,
        previous_manifest.as_ref(),
        Some(app),
        None, // No encryption for CLI mode yet
        None, // No cancellation support for scheduled backups
    ) {
        Ok(job) => {
            // Emit progress event: Finalizing
            let _ = app.emit("backup-progress", serde_json::json!({
                "stage": "finalizing",
                "message": "Finalizando backup...",
                "percentage": 90
            }));
            log::info!("Backup completed: {} files, {} bytes",
                job.files_count.unwrap_or(0),
                job.compressed_size.unwrap_or(0)
            );

            // Update manifest
            if let Ok((all_files, _)) = backup::scan_all_files(source_path) {
                if let Ok(new_manifest) = backup::build_manifest(&config_id, &all_files, source_path) {
                    if let Ok(manifest_json) = serde_json::to_string_pretty(&new_manifest) {
                        let _ = std::fs::write(&manifest_path, manifest_json);
                    }
                }
            }

            // Send success notification
            let files_count = job.files_count.unwrap_or(0);
            let size_mb = job.compressed_size.unwrap_or(0) as f64 / 1_048_576.0;

            // Emit progress event: Completed
            let _ = app.emit("backup-progress", serde_json::json!({
                "stage": "completed",
                "message": format!("Backup concluído! {} arquivos ({:.1} MB)", files_count, size_mb),
                "percentage": 100,
                "files_processed": files_count,
                "total_files": files_count
            }));

            send_notification(
                app,
                "InLocker - Backup Completed ✓",
                &format!(
                    "{}: {} files backed up ({:.1} MB)",
                    config.name, files_count, size_mb
                ),
            );

            // Note: last_backup_at will be updated next time the app opens normally
            Ok(())
        }
        Err(e) => {
            log::error!("Backup failed: {}", e);

            // Send error notification
            send_notification(
                app,
                "InLocker - Backup Failed ✗",
                &format!("{}: {}", config.name, e),
            );

            Err(format!("Backup failed: {}", e))
        }
    }
}

/// Send a native macOS notification
fn send_notification(app: &tauri::AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;

    if let Err(e) = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
    {
        log::error!("Failed to send notification: {}", e);
    }
}

/// Restore all schedules on app startup
async fn restore_schedules(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;

    let state = app.state::<AppState>();
    let scheduler_state = app.state::<SchedulerState>();

    // Load configs and clone them
    let configs_to_restore: Vec<BackupConfig> = {
        let configs = state.configs.lock().map_err(|e| e.to_string())?;
        configs
            .iter()
            .filter(|c| {
                c.schedule.as_ref().map(|s| s.enabled).unwrap_or(false)
            })
            .cloned()
            .collect()
    }; // Lock is released here

    // Restore schedules
    for config in configs_to_restore {
        log::info!("Restoring schedule for config: {}", config.id);
        match scheduler_state.register_schedule(app.clone(), config.clone()).await {
            Ok(_) => log::info!("Schedule restored for: {}", config.id),
            Err(e) => log::error!("Failed to restore schedule for {}: {}", config.id, e),
        }
    }

    Ok(())
}
