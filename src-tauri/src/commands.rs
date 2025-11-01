use crate::backup;
use crate::launchd;
use crate::scheduler::SchedulerState;
use crate::types::{BackupConfig, BackupManifest, BackupResult, BackupType};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

/// Application state wrapper
pub struct AppState {
    pub configs: Mutex<Vec<BackupConfig>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            configs: Mutex::new(Vec::new()),
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

/// Open folder picker dialog and return selected path
#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let folder = app.dialog()
        .file()
        .blocking_pick_folder();

    Ok(folder.map(|path| path.to_string()))
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
    let was_deleted = {
        let mut configs = state.configs.lock().map_err(|e| e.to_string())?;

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

        deleted
    }; // Lock is released here

    if was_deleted {
        // Clean up scheduler and launchd
        // Unregister from in-app scheduler (ignore errors if not scheduled)
        let _ = scheduler_state.unregister_schedule(&config_id).await;

        // Uninstall launchd agent (ignore errors if not installed)
        let _ = launchd::uninstall_launch_agent(&config_id);

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
) -> Result<BackupResult, String> {
    // Get the config
    let configs = state.configs.lock().map_err(|e| e.to_string())?;
    let config = configs
        .iter()
        .find(|c| c.id == config_id)
        .ok_or("Config not found")?
        .clone();
    drop(configs); // Release lock

    // Load previous manifest for incremental backup
    let manifest_path = get_manifest_path(&app, &config_id)?;
    let previous_manifest = if manifest_path.exists() && config.backup_type == BackupType::Incremental {
        let json = fs::read_to_string(&manifest_path).ok();
        json.and_then(|j| serde_json::from_str::<BackupManifest>(&j).ok())
    } else {
        None
    };

    // Perform backup
    let source_path = Path::new(&config.source_path);
    let dest_path = Path::new(&config.destination_path);

    // TODO: Get password from config when encryption UI is implemented
    let password = config.encryption_password.as_deref();

    match backup::compress_folder(
        &config_id,
        source_path,
        dest_path,
        &config.backup_type,
        previous_manifest.as_ref(),
        Some(&app),
        password,
    ) {
        Ok(mut job) => {
            job.config_id = config_id.clone();

            // Build and save new manifest
            let (all_files, _) = backup::scan_all_files(source_path)?;
            let new_manifest = backup::build_manifest(&config_id, &all_files, source_path)?;
            let manifest_json = serde_json::to_string_pretty(&new_manifest)
                .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
            fs::write(&manifest_path, manifest_json)
                .map_err(|e| format!("Failed to save manifest: {}", e))?;

            // Update config's last_backup_at and size info
            let mut configs = state.configs.lock().map_err(|e| e.to_string())?;
            if let Some(cfg) = configs.iter_mut().find(|c| c.id == config_id) {
                cfg.last_backup_at = Some(job.completed_at.unwrap_or(0));
                cfg.last_backup_original_size = job.original_size;
                cfg.last_backup_compressed_size = job.compressed_size;
                cfg.last_backup_files_count = job.files_count;
                cfg.last_backup_checksum = job.checksum.clone();
                cfg.updated_at = job.completed_at.unwrap_or(0);
            }
            drop(configs);

            // Persist configs
            let config_path = get_config_path(&app)?;
            let configs = state.configs.lock().map_err(|e| e.to_string())?;
            let json = serde_json::to_string_pretty(&*configs)
                .map_err(|e| format!("Failed to serialize configs: {}", e))?;
            fs::write(&config_path, json)
                .map_err(|e| format!("Failed to write configs: {}", e))?;

            Ok(BackupResult {
                success: true,
                message: format!(
                    "Backup completed: {} files, {:.2} MB → {:.2} MB ({:.1}% compression)",
                    job.files_count.unwrap_or(0),
                    job.original_size.unwrap_or(0) as f64 / 1_048_576.0,
                    job.compressed_size.unwrap_or(0) as f64 / 1_048_576.0,
                    (1.0 - (job.compressed_size.unwrap_or(1) as f64
                        / job.original_size.unwrap_or(1).max(1) as f64))
                        * 100.0
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
    app: AppHandle,
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

    // Register with in-app scheduler (tokio-cron-scheduler)
    scheduler_state
        .register_schedule(app.clone(), config.clone())
        .await?;

    // Install launchd agent for independent scheduling
    let cron_expr = &schedule.cron_expression;

    // Get app executable path
    let app_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get app path: {}", e))?
        .to_string_lossy()
        .to_string();

    launchd::install_launch_agent(&config_id, cron_expr, &app_path)?;

    log::info!(
        "Registered schedule for config {} (in-app + launchd)",
        config_id
    );

    Ok(true)
}

/// Unregister a scheduled backup
#[tauri::command]
pub async fn unregister_schedule(
    scheduler_state: State<'_, SchedulerState>,
    config_id: String,
) -> Result<bool, String> {
    // Unregister from in-app scheduler
    scheduler_state.unregister_schedule(&config_id).await?;

    // Uninstall launchd agent
    launchd::uninstall_launch_agent(&config_id)?;

    log::info!(
        "Unregistered schedule for config {} (in-app + launchd)",
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
    Ok(scheduler_state.is_scheduled(&config_id).await)
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

    backup::restore_backup(backup_path, restore_path, expected_checksum, password.as_deref())
}
