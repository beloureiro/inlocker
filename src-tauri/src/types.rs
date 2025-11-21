use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single backup job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub destination_path: String,
    #[serde(default)]
    pub schedule: Option<ScheduleConfig>,
    pub enabled: bool,
    /// Backup mode: Copy (no compression), Compressed (default), or Encrypted (compressed + encrypted)
    #[serde(default = "default_backup_mode")]
    pub mode: BackupMode,
    /// Encryption password (NEVER persisted to disk for security)
    /// User must provide this each time for encrypted backups
    #[serde(skip_serializing, default)]
    pub encryption_password: Option<String>,
    #[serde(default = "default_backup_type")]
    pub backup_type: BackupType,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub last_backup_at: Option<i64>,
    #[serde(default)]
    pub last_backup_original_size: Option<u64>,
    #[serde(default)]
    pub last_backup_compressed_size: Option<u64>,
    #[serde(default)]
    pub last_backup_files_count: Option<usize>,
    #[serde(default)]
    pub last_backup_checksum: Option<String>,
}

fn default_backup_type() -> BackupType {
    BackupType::Incremental
}

fn default_backup_mode() -> BackupMode {
    BackupMode::Compressed
}

/// Scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub cron_expression: String,
    pub preset: Option<SchedulePreset>,
    pub next_run: Option<i64>,
    pub enabled: bool,
}

/// Schedule presets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchedulePreset {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Custom,
}

/// Type of backup (Full or Incremental)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupType {
    Full,
    Incremental,
}

/// Backup mode (determines compression and encryption)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupMode {
    /// Copy only - no compression, no encryption (fastest)
    Copy,
    /// Compressed - with zstd compression (default)
    Compressed,
    /// Encrypted - compressed + AES-256-GCM encryption (most secure)
    Encrypted,
}

/// Represents a backup job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub id: String,
    pub config_id: String,
    pub status: BackupStatus,
    pub backup_type: BackupType,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub original_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub files_count: Option<usize>,
    pub changed_files_count: Option<usize>,
    pub error_message: Option<String>,
    pub backup_path: Option<String>,
    pub checksum: Option<String>,
}

/// Status of a backup job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Result of a backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub success: bool,
    pub message: String,
    pub job: Option<BackupJob>,
}

/// Backup manifest for incremental backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub config_id: String,
    pub created_at: i64,
    pub files: HashMap<String, FileMetadata>,
}

/// Metadata for a single file in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub modified_at: i64,
    pub checksum: String,
}

/// Diagnostics for a scheduled backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDiagnostics {
    pub config_id: String,
    pub has_schedule: bool,
    pub plist_exists: bool,
    pub plist_path: Option<String>,
    pub agent_loaded: bool,
    pub agent_label: Option<String>,
    pub executable_path: Option<String>,
    pub executable_exists: bool,
    pub logs_path: Option<String>,
    pub logs_exist: bool,
    pub next_execution: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
