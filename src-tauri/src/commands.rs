use crate::types::{BackupConfig, BackupResult};
use std::fs;
use std::path::PathBuf;
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
    config_id: String,
) -> Result<bool, String> {
    let mut configs = state.configs.lock().map_err(|e| e.to_string())?;

    let initial_len = configs.len();
    configs.retain(|c| c.id != config_id);

    // Persist to file if something was deleted
    if configs.len() < initial_len {
        let config_path = get_config_path(&app)?;
        let json = serde_json::to_string_pretty(&*configs)
            .map_err(|e| format!("Failed to serialize configs: {}", e))?;
        fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write configs: {}", e))?;
    }

    Ok(configs.len() < initial_len)
}

/// Run a backup immediately
#[tauri::command]
pub async fn run_backup_now(config_id: String) -> Result<BackupResult, String> {
    // TODO: Implement backup logic

    Ok(BackupResult {
        success: true,
        message: format!("Backup {} will be implemented in Week 2", config_id),
        job: None,
    })
}
