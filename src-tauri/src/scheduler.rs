/// Simplified scheduler module
///
/// This module has been simplified to remove the in-app tokio-cron-scheduler.
/// All scheduling is now handled by macOS launchd for independent, system-level scheduling.
///
/// The SchedulerState struct is kept as a placeholder to maintain compatibility
/// with existing code, but it no longer manages active jobs.

/// Placeholder scheduler state
///
/// DEPRECATED: This is kept only for compatibility. All scheduling is now done via launchd.
/// The in-app scheduler has been removed in favor of system-level scheduling.
pub struct SchedulerState {
    // Empty - launchd handles all scheduling
}

impl SchedulerState {
    /// Create a new (empty) scheduler state
    pub async fn new() -> Result<Self, String> {
        log::info!("SchedulerState initialized (using launchd for actual scheduling)");
        Ok(Self {})
    }

    /// Start the scheduler (no-op - launchd handles this)
    pub async fn start(&self) -> Result<(), String> {
        log::info!("Scheduler started (launchd-based)");
        Ok(())
    }

    /// Register a schedule (no-op - launchd handles this)
    ///
    /// This is a placeholder. Actual registration is done via launchd::install_launch_agent
    pub async fn register_schedule(
        &self,
        _app: tauri::AppHandle,
        config: crate::types::BackupConfig,
    ) -> Result<(), String> {
        log::info!(
            "Schedule registration requested for config: {} (handled by launchd)",
            config.id
        );
        Ok(())
    }

    /// Unregister a schedule (no-op - launchd handles this)
    ///
    /// This is a placeholder. Actual unregistration is done via launchd::uninstall_launch_agent
    pub async fn unregister_schedule(&self, config_id: &str) -> Result<(), String> {
        log::info!(
            "Schedule unregistration requested for config: {} (handled by launchd)",
            config_id
        );
        Ok(())
    }

    /// Check if a config has a registered schedule
    ///
    /// DEPRECATED: Use launchd::is_agent_loaded instead
    pub async fn is_scheduled(&self, _config_id: &str) -> bool {
        // Always return false - use launchd::is_agent_loaded for real status
        false
    }
}
