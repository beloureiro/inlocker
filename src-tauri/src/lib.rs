pub mod backup;
pub mod crypto;
mod commands;
mod launchd;
mod scheduler;
pub mod types;

use commands::AppState;
use scheduler::SchedulerState;
use types::BackupConfig;
use tauri::{Emitter, Listener, Manager};

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
        // IMPORTANT: single-instance MUST be the FIRST plugin (order matters!)
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // Detect CLI mode (--backup <config_id>)
            let is_cli_backup = argv.len() >= 3 && argv.get(1).map(|s| s.as_str()) == Some("--backup");

            if is_cli_backup {
                // CLI mode: run backup from existing instance
                let config_id = argv.get(2).cloned().unwrap_or_default();
                log::info!("Single instance: running scheduled backup for {} from existing instance", config_id);

                // Create progress window and run backup in existing instance
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    // Create scheduled-progress window dynamically
                    match tauri::WebviewWindowBuilder::new(
                        &app_handle,
                        "scheduled-progress",
                        tauri::WebviewUrl::App("progress.html".into())
                    )
                    .title("Scheduled Backup - InLocker")
                    .inner_size(600.0, 450.0)
                    .center()
                    .resizable(false)
                    .visible(true)
                    .build() {
                        Ok(window) => {
                            log::info!("Created scheduled-progress window for single-instance backup");
                            let _ = window.set_focus();
                        }
                        Err(e) => {
                            log::error!("Failed to create scheduled-progress window: {}", e);
                        }
                    }

                    // Wait for window to be ready
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    // Run the backup
                    // NOTE: Auto-close is handled by progress.html (Single Source of Truth - InDRY)
                    match run_scheduled_backup_static(&app_handle, &config_id).await {
                        Ok(_) => {
                            log::info!("Single-instance scheduled backup completed successfully");
                            // Window close is handled by progress.html based on user preferences
                        }
                        Err(e) => {
                            log::error!("Single-instance scheduled backup failed: {}", e);
                            // Window stays open for user to see the error (handled by progress.html)
                        }
                    }
                });
            } else {
                // Normal mode: focus existing main window
                log::info!("Single instance detected, focusing existing main window");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_focus();
                    let _ = window.unminimize();
                }
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
            commands::verify_backup_exists,
            commands::list_available_backups,
            commands::restore_backup,
            commands::load_preferences,
            commands::save_preferences,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Determine which mode we're in
            let is_cli_mode = cli_backup_config_id.is_some();

            // Setup window-ready event listeners based on mode
            if is_cli_mode {
                // CLI mode: Create and show scheduled-progress window dynamically
                // Window is NOT in tauri.conf.json - must be created here
                log::info!("CLI mode detected: creating scheduled-progress window dynamically");

                // Hide main window (in case it was created)
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
                    log::info!("Main window hidden for CLI mode");
                }

                // Create scheduled-progress window dynamically with pure HTML (no React)
                let app_handle_for_window = app.handle().clone();
                match tauri::WebviewWindowBuilder::new(
                    &app_handle_for_window,
                    "scheduled-progress",
                    tauri::WebviewUrl::App("progress.html".into())
                )
                .title("Scheduled Backup - InLocker")
                .inner_size(600.0, 450.0)
                .center()
                .resizable(false)
                .visible(false)
                .build() {
                    Ok(window) => {
                        log::info!("Created scheduled-progress window dynamically");
                        let progress_window = window.clone();
                        window.listen("window-ready", move |_event| {
                            log::info!("Received window-ready event for scheduled-progress window");
                            if let Err(e) = progress_window.show() {
                                log::error!("Failed to show scheduled-progress window: {}", e);
                            } else {
                                log::info!("Successfully showed scheduled-progress window after ready event");
                                if let Err(e) = progress_window.set_focus() {
                                    log::warn!("Failed to focus scheduled-progress window: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to create scheduled-progress window: {}", e);
                    }
                }
            } else {
                // Normal mode: Show main window only
                // scheduled-progress window will be created on-demand by test_schedule_now
                log::info!("Normal mode detected: setting up main window only");

                // Setup listener for main window
                if let Some(window) = app.get_webview_window("main") {
                    let main_window = window.clone();
                    window.listen("window-ready", move |event| {
                        let payload_str = event.payload();
                        log::info!("Received window-ready event with payload: {}", payload_str);

                        if payload_str.contains("\"label\":\"main\"") {
                            log::info!("Processing window-ready for main window");
                            if let Err(e) = main_window.show() {
                                log::error!("Failed to show main window: {}", e);
                            } else {
                                log::info!("Successfully showed main window after ready event");
                            }
                        }
                    });
                } else {
                    log::error!("main window not found in normal mode!");
                }
            }

            // CLI mode: run backup with progress UI
            if let Some(config_id) = cli_backup_config_id {
                log::info!("CLI mode: starting backup for config: {}", config_id);

                tauri::async_runtime::spawn(async move {
                    // Wait a bit for the window to be ready
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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
                // Normal app startup
                log::info!("Normal mode: starting regular app");

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
    let _ = app.emit("backup:progress", serde_json::json!({
        "stage": "initializing",
        "message": format!("Initializing backup: {}", config.name),
        "percentage": 0
    }));

    // Send notification: Backup started
    send_notification(
        app,
        "InLocker - Backup Started",
        &format!("Starting backup: {}", config.name),
    );

    // Emit progress event: Scanning
    let _ = app.emit("backup:progress", serde_json::json!({
        "stage": "scanning",
        "message": "Scanning files...",
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
    let _ = app.emit("backup:progress", serde_json::json!({
        "stage": "compressing",
        "message": "Compressing files...",
        "percentage": 30
    }));

    // Perform backup
    // TODO: Add password parameter when CLI supports it
    match backup::compress_folder(
        &config_id,
        &config.name,
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
            let _ = app.emit("backup:progress", serde_json::json!({
                "stage": "finalizing",
                "message": "Finalizing backup...",
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
            let _ = app.emit("backup:progress", serde_json::json!({
                "stage": "completed",
                "message": format!("Backup completed! {} files ({:.1} MB)", files_count, size_mb),
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

/// Run a scheduled backup (static version for single-instance callback)
async fn run_scheduled_backup_static(app: &tauri::AppHandle, config_id: &str) -> Result<(), String> {
    run_scheduled_backup(app, config_id).await
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
