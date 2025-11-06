pub mod backup;
pub mod crypto;
mod commands;
mod launchd;
mod scheduler;
pub mod types;

use commands::AppState;
use scheduler::SchedulerState;
use types::BackupConfig;

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

    // Initialize scheduler (blocking)
    let scheduler_state = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            let state = SchedulerState::new()
                .await
                .expect("Failed to create scheduler");
            state.start().await.expect("Failed to start scheduler");
            state
        });

    tauri::Builder::default()
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
            commands::list_available_backups,
            commands::restore_backup,
        ])
        .setup(|app| {
            // Check for CLI arguments (--backup <config_id>)
            let args: Vec<String> = std::env::args().collect();
            log::info!("App started with args: {:?}", args);

            // If launched with --backup argument, run backup and exit
            if args.len() >= 3 && args[1] == "--backup" {
                let config_id = args[2].clone();
                log::info!("Running scheduled backup for config: {}", config_id);

                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = run_scheduled_backup(&app_handle, &config_id).await {
                        log::error!("Scheduled backup failed: {}", e);
                        std::process::exit(1);
                    } else {
                        log::info!("Scheduled backup completed successfully");
                        std::process::exit(0);
                    }
                });

                // Keep app running until backup completes
                return Ok(());
            }

            // Normal app startup - Re-register all schedules
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = restore_schedules(app_handle).await {
                    log::error!("Failed to restore schedules: {}", e);
                }
            });
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

    // Send notification: Backup started
    send_notification(
        app,
        "InLocker - Backup Started",
        &format!("Starting backup: {}", config.name),
    );

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
