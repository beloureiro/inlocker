use serde::{Deserialize, Serialize};

/// Configuration for a single backup job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub destination_path: String,
    pub schedule: Option<String>,
    pub enabled: bool,
    pub encrypt: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Represents a backup job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub id: String,
    pub config_id: String,
    pub status: BackupStatus,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub original_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub error_message: Option<String>,
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
