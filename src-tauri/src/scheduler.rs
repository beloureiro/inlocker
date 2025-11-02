use crate::backup;
use crate::types::{BackupConfig, BackupManifest, BackupType};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use uuid::Uuid;

/// Scheduler state that manages all scheduled jobs
pub struct SchedulerState {
    pub scheduler: Arc<Mutex<JobScheduler>>,
    pub job_ids: Arc<Mutex<HashMap<String, Uuid>>>, // config_id -> job_uuid
}

impl SchedulerState {
    /// Create a new scheduler state
    pub async fn new() -> Result<Self, JobSchedulerError> {
        let scheduler = JobScheduler::new().await?;

        Ok(Self {
            scheduler: Arc::new(Mutex::new(scheduler)),
            job_ids: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), String> {
        let scheduler = self.scheduler.lock().await;
        scheduler.start().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Register a scheduled backup job
    pub async fn register_schedule(
        &self,
        app: AppHandle,
        config: BackupConfig,
    ) -> Result<(), String> {
        // Get cron expression from config
        let cron_expr = config
            .schedule
            .as_ref()
            .ok_or("No schedule configured")?
            .cron_expression
            .clone();

        if cron_expr.is_empty() {
            return Err("Empty cron expression".to_string());
        }

        // Clone data for the closure
        let config_id = config.id.clone();
        let source_path = config.source_path.clone();
        let dest_path = config.destination_path.clone();
        let backup_type = config.backup_type.clone();
        let backup_mode = config.mode.clone();
        let encryption_password = config.encryption_password.clone();
        let app_clone = app.clone();

        // Create a job
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
            let config_id = config_id.clone();
            let source_path = source_path.clone();
            let dest_path = dest_path.clone();
            let backup_type = backup_type.clone();
            let backup_mode = backup_mode.clone();
            let encryption_password = encryption_password.clone();
            let app = app_clone.clone();

            Box::pin(async move {
                log::info!("Executing scheduled backup for config: {}", config_id);

                // Load previous manifest for incremental backup
                let manifest_path = get_manifest_path(&app, &config_id);
                let previous_manifest = if let Ok(path) = manifest_path {
                    if path.exists() && backup_type == BackupType::Incremental {
                        let json = fs::read_to_string(&path).ok();
                        json.and_then(|j| serde_json::from_str::<BackupManifest>(&j).ok())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Perform backup
                let source = Path::new(&source_path);
                let dest = Path::new(&dest_path);

                match backup::compress_folder(&config_id, source, dest, &backup_type, &backup_mode, previous_manifest.as_ref(), Some(&app), encryption_password.as_deref(), None)
                {
                    Ok(job) => {
                        log::info!("Scheduled backup completed successfully: {:?}", job);

                        // Update manifest
                        if let Ok(manifest_path) = get_manifest_path(&app, &config_id) {
                            if let Ok((all_files, _)) = backup::scan_all_files(source) {
                                if let Ok(new_manifest) =
                                    backup::build_manifest(&config_id, &all_files, source)
                                {
                                    if let Ok(manifest_json) =
                                        serde_json::to_string_pretty(&new_manifest)
                                    {
                                        let _ = fs::write(&manifest_path, manifest_json);
                                    }
                                }
                            }
                        }

                        // Send notification
                        let _ = send_notification(
                            &app,
                            "Backup Completed",
                            &format!(
                                "Backup completed successfully: {} files",
                                job.files_count.unwrap_or(0)
                            ),
                        );
                    }
                    Err(e) => {
                        log::error!("Scheduled backup failed: {}", e);

                        // Send error notification
                        let _ = send_notification(
                            &app,
                            "Backup Failed",
                            &format!("Backup failed: {}", e),
                        );
                    }
                }
            })
        })
        .map_err(|e| format!("Failed to create job: {}", e))?;

        // Add job to scheduler
        let scheduler = self.scheduler.lock().await;
        let job_uuid = scheduler
            .add(job)
            .await
            .map_err(|e| format!("Failed to add job: {}", e))?;

        // Store job UUID
        let mut job_ids = self.job_ids.lock().await;

        // Remove old job if exists
        if let Some(old_uuid) = job_ids.insert(config.id.clone(), job_uuid) {
            let _ = scheduler.remove(&old_uuid).await;
        }

        log::info!(
            "Registered schedule for config: {} with cron: {}",
            config.id,
            cron_expr
        );

        Ok(())
    }

    /// Unregister a scheduled backup job
    pub async fn unregister_schedule(&self, config_id: &str) -> Result<(), String> {
        let mut job_ids = self.job_ids.lock().await;

        if let Some(job_uuid) = job_ids.remove(config_id) {
            drop(job_ids); // Release lock before await
            let scheduler = self.scheduler.lock().await;
            scheduler
                .remove(&job_uuid)
                .await
                .map_err(|e| format!("Failed to remove job: {}", e))?;

            log::info!("Unregistered schedule for config: {}", config_id);
            Ok(())
        } else {
            Err(format!("No schedule found for config: {}", config_id))
        }
    }

    /// Check if a config has a registered schedule
    pub async fn is_scheduled(&self, config_id: &str) -> bool {
        let job_ids = self.job_ids.lock().await;
        job_ids.contains_key(config_id)
    }
}

/// Helper to get manifest path
fn get_manifest_path(app: &AppHandle, config_id: &str) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join(format!("manifest_{}.json", config_id)))
}

/// Send a native notification
fn send_notification(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| format!("Failed to send notification: {}", e))
}
