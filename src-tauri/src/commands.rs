use crate::backup;
use crate::launchd;
use crate::scheduler::SchedulerState;
use crate::types::{AppPreferences, BackupConfig, BackupManifest, BackupResult, BackupType, ScheduleDiagnostics};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// Application state wrapper
pub struct AppState {
    pub configs: Mutex<Vec<BackupConfig>>,
    /// Cancellation flags for running backups: config_id -> cancel_flag
    pub cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            configs: Mutex::new(Vec::new()),
            cancel_flags: Mutex::new(HashMap::new()),
        }
    }
}

/// Get the path to the config file
fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // Ensure the directory exists
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join("configs.json"))
}

/// Get the path to the preferences file
fn get_preferences_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // Ensure the directory exists
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join("preferences.json"))
}

/// Open folder picker dialog and return selected path
#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let folder = app.dialog()
        .file()
        .blocking_pick_folder();

    Ok(folder.map(|path| path.to_string()))
}

/// Open file picker dialog and return selected file path
#[tauri::command]
pub async fn select_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file = app.dialog()
        .file()
        .add_filter("Backup Files", &["zst", "enc"])
        .blocking_pick_file();

    Ok(file.map(|path| path.to_string()))
}

/// Save a backup configuration
#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: BackupConfig,
) -> Result<BackupConfig, String> {
    let mut configs = state.configs.lock().map_err(|e| e.to_string())?;

    // Check if config already exists
    if let Some(existing) = configs.iter_mut().find(|c| c.id == config.id) {
        *existing = config.clone();
    } else {
        configs.push(config.clone());
    }

    // Persist to file
    let config_path = get_config_path(&app)?;
    let json = serde_json::to_string_pretty(&*configs)
        .map_err(|e| format!("Failed to serialize configs: {}", e))?;
    fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write configs: {}", e))?;

    Ok(config)
}

/// Load all backup configurations
#[tauri::command]
pub async fn load_configs(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<BackupConfig>, String> {
    let config_path = get_config_path(&app)?;

    // Load from file if it exists
    if config_path.exists() {
        let json = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read configs: {}", e))?;
        let loaded_configs: Vec<BackupConfig> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse configs: {}", e))?;

        // Update state with loaded configs
        let mut configs = state.configs.lock().map_err(|e| e.to_string())?;
        *configs = loaded_configs.clone();

        Ok(loaded_configs)
    } else {
        // Return empty list if no config file exists yet
        Ok(Vec::new())
    }
}

/// Delete a backup configuration
#[tauri::command]
pub async fn delete_config(
    app: AppHandle,
    state: State<'_, AppState>,
    scheduler_state: State<'_, SchedulerState>,
    config_id: String,
) -> Result<bool, String> {
    let (was_deleted, config_name) = {
        let mut configs = state.configs.lock().map_err(|e| e.to_string())?;

        // Save config name before deletion for wrapper script cleanup
        let config_name = configs.iter().find(|c| c.id == config_id).map(|c| c.name.clone());

        let initial_len = configs.len();
        configs.retain(|c| c.id != config_id);

        let deleted = configs.len() < initial_len;

        // Persist to file if something was deleted
        if deleted {
            let config_path = get_config_path(&app)?;
            let json = serde_json::to_string_pretty(&*configs)
                .map_err(|e| format!("Failed to serialize configs: {}", e))?;
            fs::write(&config_path, json)
                .map_err(|e| format!("Failed to write configs: {}", e))?;
        }

        (deleted, config_name)
    }; // Lock is released here

    if was_deleted {
        // Clean up scheduler and launchd
        // Unregister from in-app scheduler (ignore errors if not scheduled)
        let _ = scheduler_state.unregister_schedule(&config_id).await;

        // Uninstall launchd agent (ignore errors if not installed)
        let _ = launchd::uninstall_launch_agent(&config_id, config_name.as_deref());

        log::info!("Deleted config and cleaned up schedules: {}", config_id);
    }

    Ok(was_deleted)
}

/// Run a backup immediately
#[tauri::command]
pub async fn run_backup_now(
    app: AppHandle,
    state: State<'_, AppState>,
    config_id: String,
    password: Option<String>,
) -> Result<BackupResult, String> {
    // Create cancellation flag for this backup
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = state.cancel_flags.lock().map_err(|e| e.to_string())?;
        flags.insert(config_id.clone(), Arc::clone(&cancel_flag));
    }

    // Get the config
    let configs = state.configs.lock().map_err(|e| e.to_string())?;
    let config = configs
        .iter()
        .find(|c| c.id == config_id)
        .ok_or("Config not found")?
        .clone();
    drop(configs); // Release lock

    // Get paths first
    let source_path = Path::new(&config.source_path);
    let dest_path = Path::new(&config.destination_path);

    // Load previous manifest for incremental backup
    // BUT only if physical backup files actually exist on disk
    let manifest_path = get_manifest_path(&app, &config_id)?;
    let previous_manifest = if manifest_path.exists() && config.backup_type == BackupType::Incremental {
        // First load manifest
        let loaded_manifest = fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|j| serde_json::from_str::<BackupManifest>(&j).ok());

        if let Some(manifest) = loaded_manifest {
            // Verify physical backup exists and matches manifest
            match backup::verify_physical_backup_exists(dest_path, &config.mode, &manifest) {
                Ok(true) => {
                    log::info!("✅ Physical backup verified - using manifest for incremental");
                    Some(manifest)
                },
                Ok(false) => {
                    log::warn!("⚠️  Manifest exists but physical backup missing/corrupted - treating as first backup");
                    // Delete stale manifest
                    let _ = fs::remove_file(&manifest_path);
                    None
                },
                Err(e) => {
                    log::error!("❌ Failed to verify backup: {} - treating as first backup", e);
                    None
                }
            }
        } else {
            log::warn!("⚠️  Failed to parse manifest - treating as first backup");
            None
        }
    } else {
        None
    };

    // Use password from parameter (provided by UI prompt)
    // Password is NEVER saved in config for security reasons
    let password_ref = password.as_deref();

    // Perform backup with cancellation support
    let backup_result = backup::compress_folder(
        &config_id,
        &config.name,
        source_path,
        dest_path,
        &config.backup_type,
        &config.mode,
        previous_manifest.as_ref(),
        Some(&app),
        password_ref,
        Some(Arc::clone(&cancel_flag)),
    );

    // Clean up cancellation flag
    {
        let mut flags = state.cancel_flags.lock().map_err(|e| e.to_string())?;
        flags.remove(&config_id);
    }

    match backup_result {
        Ok(mut job) => {
            job.config_id = config_id.clone();

            // FIRST: Emit completed event for progress window auto-close
            // This must happen BEFORE the slow manifest scan to avoid UI appearing stuck
            let files_count = job.files_count.unwrap_or(0);
            let original_mb = job.original_size.unwrap_or(0) as f64 / 1_048_576.0;
            let compressed_mb = job.compressed_size.unwrap_or(0) as f64 / 1_048_576.0;
            let compression_pct = if job.original_size.unwrap_or(0) > 0 {
                (1.0 - (job.compressed_size.unwrap_or(1) as f64
                    / job.original_size.unwrap_or(1).max(1) as f64))
                    * 100.0
            } else {
                0.0
            };

            log::info!("📤 Emitting completed event with compressed_size: {} bytes", job.compressed_size.unwrap_or(0));
            let emit_result = app.emit("backup:progress", serde_json::json!({
                "config_id": config_id,
                "stage": "completed",
                "message": format!("Backup completed! {} files", files_count),
                "details": format!("{:.1} MB → {:.1} MB ({:.0}% compression)", original_mb, compressed_mb, compression_pct),
                "current": files_count,
                "total": files_count,
                "original_size": job.original_size.unwrap_or(0),
                "compressed_size": job.compressed_size.unwrap_or(0)
            }));
            log::info!("📤 Emit result: {:?}", emit_result);

            // Update config's last_backup_at and size info (fast - do immediately)
            {
                let mut configs = state.configs.lock().map_err(|e| e.to_string())?;
                if let Some(cfg) = configs.iter_mut().find(|c| c.id == config_id) {
                    cfg.last_backup_at = Some(job.completed_at.unwrap_or(0));
                    cfg.last_backup_original_size = job.original_size;
                    cfg.last_backup_compressed_size = job.compressed_size;
                    cfg.last_backup_files_count = job.files_count;
                    cfg.last_backup_checksum = job.checksum.clone();
                    cfg.updated_at = job.completed_at.unwrap_or(0);
                }
            }

            // Persist configs (fast - do immediately)
            {
                let config_path = get_config_path(&app)?;
                let configs = state.configs.lock().map_err(|e| e.to_string())?;
                let json = serde_json::to_string_pretty(&*configs)
                    .map_err(|e| format!("Failed to serialize configs: {}", e))?;
                fs::write(&config_path, json)
                    .map_err(|e| format!("Failed to write configs: {}", e))?;
            }

            // Build and save manifest in background (slow - don't block UI)
            let manifest_config_id = config_id.clone();
            let manifest_source = source_path.to_path_buf();
            let manifest_dest = manifest_path.clone();
            tokio::spawn(async move {
                log::info!("📋 Building manifest in background for {}", manifest_config_id);
                match backup::scan_all_files(&manifest_source) {
                    Ok((all_files, _)) => {
                        match backup::build_manifest(&manifest_config_id, &all_files, &manifest_source) {
                            Ok(new_manifest) => {
                                match serde_json::to_string_pretty(&new_manifest) {
                                    Ok(manifest_json) => {
                                        if let Err(e) = fs::write(&manifest_dest, manifest_json) {
                                            log::error!("Failed to save manifest: {}", e);
                                        } else {
                                            log::info!("✅ Manifest saved in background");
                                        }
                                    }
                                    Err(e) => log::error!("Failed to serialize manifest: {}", e),
                                }
                            }
                            Err(e) => log::error!("Failed to build manifest: {}", e),
                        }
                    }
                    Err(e) => log::error!("Failed to scan files for manifest: {}", e),
                }
            });

            Ok(BackupResult {
                success: true,
                message: format!(
                    "Backup completed: {} files, {:.2} MB → {:.2} MB ({:.1}% compression)",
                    files_count,
                    original_mb,
                    compressed_mb,
                    compression_pct
                ),
                job: Some(job),
            })
        }
        Err(e) => Ok(BackupResult {
            success: false,
            message: format!("Backup failed: {}", e),
            job: None,
        }),
    }
}

/// Get the path to the manifest file for a config
fn get_manifest_path(app: &AppHandle, config_id: &str) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join(format!("manifest_{}.json", config_id)))
}

/// Register a scheduled backup
#[tauri::command]
pub async fn register_schedule(
    _app: AppHandle,
    state: State<'_, AppState>,
    scheduler_state: State<'_, SchedulerState>,
    config_id: String,
) -> Result<bool, String> {
    // Get the config
    let config = {
        let configs = state.configs.lock().map_err(|e| e.to_string())?;
        configs
            .iter()
            .find(|c| c.id == config_id)
            .ok_or("Config not found")?
            .clone()
    }; // Lock is released here

    // Check if schedule is configured
    let schedule = config
        .schedule
        .as_ref()
        .ok_or("No schedule configured for this backup")?;

    // NOTE: In-app scheduler removed - using only launchd for system-level scheduling
    // This provides independent scheduling that works even when the app is closed
    let _ = scheduler_state; // Suppress unused warning

    // Install launchd agent for system-level scheduling
    let cron_expr = &schedule.cron_expression;

    // Get app executable path (handles both dev and production)
    let app_path = launchd::get_executable_path()?;

    launchd::install_launch_agent(&config_id, &config.name, cron_expr, &app_path)?;

    log::info!(
        "Registered schedule for config {} (launchd)",
        config_id
    );

    Ok(true)
}

/// Unregister a scheduled backup
#[tauri::command]
pub async fn unregister_schedule(
    scheduler_state: State<'_, SchedulerState>,
    state: State<'_, AppState>,
    config_id: String,
) -> Result<bool, String> {
    let _ = scheduler_state; // Suppress unused warning

    // Get config name for wrapper script deletion
    let config_name = {
        let configs = state.configs.lock().map_err(|e| e.to_string())?;
        configs.iter().find(|c| c.id == config_id).map(|c| c.name.clone())
    };

    // Uninstall launchd agent
    launchd::uninstall_launch_agent(&config_id, config_name.as_deref())?;

    log::info!(
        "Unregistered schedule for config {} (launchd)",
        config_id
    );

    Ok(true)
}

/// Check if a backup is scheduled
#[tauri::command]
pub async fn check_schedule_status(
    scheduler_state: State<'_, SchedulerState>,
    config_id: String,
) -> Result<bool, String> {
    let _ = scheduler_state; // Suppress unused warning

    // Use launchd to check if agent is loaded
    let is_loaded = launchd::is_agent_loaded(&config_id).unwrap_or(false);
    Ok(is_loaded)
}

/// List available backups for a configuration
#[tauri::command]
pub async fn list_available_backups(config_id: String, state: State<'_, AppState>) -> Result<Vec<backup::BackupInfo>, String> {
    // Get the config to find the destination path
    let destination_path = {
        let configs = state.configs.lock().map_err(|e| e.to_string())?;
        configs
            .iter()
            .find(|c| c.id == config_id)
            .map(|c| c.destination_path.clone())
            .ok_or("Config not found")?
    };

    let dest_path = Path::new(&destination_path);
    backup::list_backups(dest_path)
}

/// Restore a backup to a specified location
#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    backup_file_path: String,
    restore_destination: String,
    expected_checksum: Option<String>,
    password: Option<String>,
) -> Result<backup::RestoreResult, String> {
    let backup_path = Path::new(&backup_file_path);
    let restore_path = Path::new(&restore_destination);

    if !backup_path.exists() {
        return Err("Backup file not found".to_string());
    }

    // Create cancellation flag for this restore
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = state.cancel_flags.lock().map_err(|e| e.to_string())?;
        // Use a special key for restore operations
        flags.insert(format!("restore-{}", backup_file_path), Arc::clone(&cancel_flag));
    }

    let result = backup::restore_backup(
        backup_path,
        restore_path,
        expected_checksum,
        password.as_deref(),
        Some(&app),
        Some(Arc::clone(&cancel_flag)),
    );

    // Clean up cancellation flag
    {
        let mut flags = state.cancel_flags.lock().map_err(|e| e.to_string())?;
        flags.remove(&format!("restore-{}", backup_file_path));
    }

    result
}

/// Cancel a running backup
#[tauri::command]
pub async fn cancel_backup(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<bool, String> {
    let flags = state.cancel_flags.lock().map_err(|e| e.to_string())?;

    if let Some(flag) = flags.get(&config_id) {
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
        log::info!("Cancellation requested for backup: {}", config_id);
        Ok(true)
    } else {
        // No backup running with this config_id
        Ok(false)
    }
}

/// Cancel a running restore
#[tauri::command]
pub async fn cancel_restore(
    state: State<'_, AppState>,
    backup_file_path: String,
) -> Result<bool, String> {
    let flags = state.cancel_flags.lock().map_err(|e| e.to_string())?;
    let restore_key = format!("restore-{}", backup_file_path);

    if let Some(flag) = flags.get(&restore_key) {
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
        log::info!("Cancellation requested for restore: {}", backup_file_path);
        Ok(true)
    } else {
        // No restore running with this backup_file_path
        Ok(false)
    }
}

/// Test a schedule now (destroy and recreate progress window)
#[tauri::command]
pub async fn test_schedule_now(
    app: tauri::AppHandle,
    config_id: String,
) -> Result<String, String> {
    use tauri::Manager;
    use tauri::Listener;
    use std::sync::Arc;
    use tokio::sync::Notify;

    log::info!("Manual test requested for schedule: {}", config_id);

    // Get config name for display in progress window
    let config_name = {
        let state = app.state::<AppState>();
        let configs = state.configs.lock().map_err(|e| e.to_string())?;
        configs.iter()
            .find(|c| c.id == config_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown Backup".to_string())
    };

    // Destroy existing window if it exists
    if let Some(old_window) = app.get_webview_window("scheduled-progress") {
        log::info!("Destroying existing scheduled-progress window");
        old_window.destroy().map_err(|e| format!("Failed to destroy window: {}", e))?;
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    log::info!("Creating new scheduled-progress window");

    // Create notification channel for window-ready
    let ready_notify = Arc::new(Notify::new());
    let ready_clone = ready_notify.clone();

    // Create new window dynamically with pure HTML (no React)
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "scheduled-progress",
        tauri::WebviewUrl::App("progress.html".into())
    )
    .title("Scheduled Backup - InLocker")
    .inner_size(600.0, 450.0)
    .center()
    .resizable(false)
    .visible(false)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    // Setup listener for window-ready
    let unlisten_handle = window.listen("window-ready", move |event| {
        let payload = event.payload();
        log::info!("Received window-ready: {}", payload);
        if payload.contains("\"label\":\"scheduled-progress\"") {
            ready_clone.notify_one();
        }
    });

    log::info!("Waiting for window-ready event...");

    // Wait for window-ready (timeout 3s)
    let timeout = tokio::time::Duration::from_secs(3);
    let wait_result = tokio::time::timeout(timeout, ready_notify.notified()).await;

    app.unlisten(unlisten_handle);

    if wait_result.is_err() {
        log::warn!("Timeout waiting for window-ready");
    }

    // Show window
    window.show().map_err(|e| format!("Failed to show: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus: {}", e))?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Emit backup trigger with config info
    log::info!("Emitting test-backup-trigger for: {} ({})", config_name, config_id);
    app.emit_to("scheduled-progress", "test-backup-trigger", serde_json::json!({
        "config_id": config_id,
        "config_name": config_name
    }))
    .map_err(|e| format!("Failed to emit: {}", e))?;

    Ok(format!("Test backup started for {}", config_id))
}

/// Check if app is running in scheduled/CLI mode
#[tauri::command]
pub fn is_scheduled_mode() -> bool {
    // Check if app was started with --backup argument
    let args: Vec<String> = std::env::args().collect();
    args.len() >= 3 && args[1] == "--backup"
}

/// Open the schedule logs directory in Finder
#[tauri::command]
pub async fn open_schedule_logs(config_id: String) -> Result<(), String> {
    let log_path = launchd::get_log_path(&config_id)?;
    let log_dir = log_path
        .parent()
        .ok_or("Failed to get logs directory")?;

    // Open in Finder using 'open' command
    let output = std::process::Command::new("open")
        .arg(log_dir)
        .output()
        .map_err(|e| format!("Failed to open Finder: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to open logs: {}", stderr));
    }

    log::info!("Opened logs directory for config: {}", config_id);
    Ok(())
}

/// Verify if the last backup file still exists on disk
#[tauri::command]
pub async fn verify_backup_exists(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<bool, String> {
    let configs = state.configs.lock().map_err(|e| e.to_string())?;

    let config = configs
        .iter()
        .find(|c| c.id == config_id)
        .ok_or("Configuration not found")?;

    if config.last_backup_at.is_none() {
        return Ok(false);
    }

    // Find the most recent backup file in destination_path
    let dest_dir = std::path::Path::new(&config.destination_path);

    if !dest_dir.exists() || !dest_dir.is_dir() {
        return Ok(false);
    }

    // Read directory and find backup files
    let entries = std::fs::read_dir(dest_dir)
        .map_err(|e| format!("Failed to read destination directory: {}", e))?;

    let mut backup_files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Match InLocker backup files
        if file_name.starts_with("Bkp_InLocker_") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    backup_files.push((path, modified));
                }
            }
        }
    }

    // Sort by modification time (most recent first)
    backup_files.sort_by(|a, b| b.1.cmp(&a.1));

    // Check if most recent backup file exists
    Ok(!backup_files.is_empty())
}

/// Diagnose scheduling system for a configuration
#[tauri::command]
pub async fn diagnose_schedule(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<ScheduleDiagnostics, String> {
    log::info!("=== Diagnosing schedule for config: {} ===", config_id);

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Get config
    let config = {
        let configs = state.configs.lock().map_err(|e| e.to_string())?;
        configs
            .iter()
            .find(|c| c.id == config_id)
            .ok_or("Config not found")?
            .clone()
    };

    let has_schedule = config.schedule.is_some();

    // Check plist file
    let plist_path = match launchd::get_plist_path(&config_id) {
        Ok(path) => Some(path),
        Err(e) => {
            errors.push(format!("Failed to get plist path: {}", e));
            None
        }
    };

    let plist_exists = plist_path.as_ref().map(|p| p.exists()).unwrap_or(false);

    if !plist_exists && has_schedule {
        errors.push("Schedule configured but plist file not found".to_string());
    }

    // Check if agent is loaded
    let agent_loaded = match launchd::is_agent_loaded(&config_id) {
        Ok(loaded) => {
            if !loaded && has_schedule {
                errors.push("Agent not loaded in launchctl".to_string());
            }
            loaded
        }
        Err(e) => {
            errors.push(format!("Failed to check agent status: {}", e));
            false
        }
    };

    let agent_label = if has_schedule {
        Some(format!("com.inlocker.backup.{}", config_id))
    } else {
        None
    };

    // Check executable path
    let executable_path = match launchd::get_executable_path() {
        Ok(path) => Some(path),
        Err(e) => {
            errors.push(format!("Failed to get executable path: {}", e));
            None
        }
    };

    let executable_exists = executable_path
        .as_ref()
        .map(|p| PathBuf::from(p).exists())
        .unwrap_or(false);

    if !executable_exists {
        errors.push("Executable not found at expected path".to_string());
    }

    // Check logs
    let logs_path = match launchd::get_log_path(&config_id) {
        Ok(path) => Some(path.to_string_lossy().to_string()),
        Err(e) => {
            warnings.push(format!("Failed to get log path: {}", e));
            None
        }
    };

    let logs_exist = logs_path.as_ref().map(|p| PathBuf::from(p).exists()).unwrap_or(false);

    // Next execution (simplified for now)
    let next_execution = config
        .schedule
        .as_ref()
        .and_then(|s| s.next_run)
        .map(|ts| {
            let datetime = chrono::DateTime::from_timestamp(ts, 0)
                .unwrap_or_else(|| chrono::Utc::now().into());
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        });

    let diagnostics = ScheduleDiagnostics {
        config_id: config_id.clone(),
        has_schedule,
        plist_exists,
        plist_path: plist_path.map(|p| p.to_string_lossy().to_string()),
        agent_loaded,
        agent_label,
        executable_path,
        executable_exists,
        logs_path,
        logs_exist,
        next_execution,
        errors,
        warnings,
    };

    log::info!("Diagnostics results: {:?}", diagnostics);
    Ok(diagnostics)
}

/// Load application preferences
#[tauri::command]
pub async fn load_preferences(app: AppHandle) -> Result<AppPreferences, String> {
    log::info!("load_preferences called");
    let prefs_path = get_preferences_path(&app)?;
    log::info!("Preferences path: {:?}", prefs_path);

    if prefs_path.exists() {
        log::info!("Preferences file exists, reading...");
        let json = fs::read_to_string(&prefs_path)
            .map_err(|e| {
                log::error!("Failed to read preferences file: {}", e);
                format!("Failed to read preferences: {}", e)
            })?;
        let prefs: AppPreferences = serde_json::from_str(&json)
            .map_err(|e| {
                log::error!("Failed to parse preferences JSON: {}", e);
                format!("Failed to parse preferences: {}", e)
            })?;
        log::info!("Successfully loaded preferences: {:?}", prefs);
        Ok(prefs)
    } else {
        log::info!("Preferences file does not exist, returning defaults");
        let defaults = AppPreferences::default();
        log::info!("Default preferences: {:?}", defaults);
        Ok(defaults)
    }
}

/// Save application preferences
#[tauri::command]
pub async fn save_preferences(
    app: AppHandle,
    preferences: AppPreferences,
) -> Result<AppPreferences, String> {
    let prefs_path = get_preferences_path(&app)?;

    let json = serde_json::to_string_pretty(&preferences)
        .map_err(|e| format!("Failed to serialize preferences: {}", e))?;
    fs::write(&prefs_path, json)
        .map_err(|e| format!("Failed to write preferences: {}", e))?;

    log::info!("Saved preferences: {:?}", preferences);
    Ok(preferences)
}
