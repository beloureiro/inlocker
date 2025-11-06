use crate::crypto::{decrypt, encrypt, EncryptionMetadata};
use crate::types::{BackupJob, BackupManifest, BackupMode, BackupStatus, BackupType, FileMetadata};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tauri::Emitter;

/// Progress event payload
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupProgress {
    pub config_id: String,
    pub stage: String,
    pub message: String,
    pub details: Option<String>,
    pub current: Option<usize>,  // Files processed so far
    pub total: Option<usize>,     // Total files to process
}

/// Backup a folder with support for 3 modes: Copy, Compressed, or Encrypted
///
/// # Modes
/// - Copy: No compression (fastest, largest size)
/// - Compressed: zstd compression level 3 (balanced)
/// - Encrypted: zstd compression + AES-256-GCM encryption (most secure)
///
/// # Security
/// - If mode is Encrypted, backup is encrypted with AES-256-GCM
/// - Encryption metadata (salt, nonce) is embedded in the file
/// - Password is derived using Argon2id (RFC 9106)
pub fn compress_folder(
    config_id: &str,
    source_path: &Path,
    dest_path: &Path,
    backup_type: &BackupType,
    mode: &BackupMode,
    previous_manifest: Option<&BackupManifest>,
    app: Option<&tauri::AppHandle>,
    password: Option<&str>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> Result<BackupJob, String> {
    // Helper to emit progress
    let emit_progress = |stage: &str, message: &str, details: Option<String>, current: Option<usize>, total: Option<usize>| {
        if let Some(app_handle) = app {
            let _ = app_handle.emit("backup:progress", BackupProgress {
                config_id: config_id.to_string(),
                stage: stage.to_string(),
                message: message.to_string(),
                details,
                current,
                total,
            });
        }
    };

    // Helper to check if backup was cancelled
    let check_cancelled = || -> Result<(), String> {
        if let Some(ref flag) = cancel_flag {
            if flag.load(Ordering::SeqCst) {
                log::warn!("🚫 Backup cancelled by user");
                return Err("Backup cancelled by user".to_string());
            }
        }
        Ok(())
    };

    log::info!("🔵 Starting {} backup", match backup_type {
        BackupType::Full => "FULL",
        BackupType::Incremental => "INCREMENTAL",
    });
    log::info!("📂 Source: {}", source_path.display());
    log::info!("💾 Destination: {}", dest_path.display());

    emit_progress("starting", "Starting backup", None, None, None);
    check_cancelled()?;

    let started_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Generate timestamp for backup name
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

    log::info!("📋 Scanning files...");
    emit_progress("scanning", "Scanning files", None, None, None);

    // Scan ALL files first (for comparison)
    let (all_files, total_source_size) = scan_all_files(source_path)?;
    let total_files_count = all_files.len();

    // Determine which files to backup
    let (files_to_backup, total_size) = match backup_type {
        BackupType::Full => (all_files, total_source_size),
        BackupType::Incremental => {
            if previous_manifest.is_none() {
                // No previous backup - backup everything
                (all_files, total_source_size)
            } else {
                // Has previous backup - find changed files
                scan_changed_files(source_path, previous_manifest)?
            }
        }
    };

    let files_count = files_to_backup.len();
    log::info!("✅ Found {} files to backup ({:.2} MB)", files_count, total_size as f64 / 1_048_576.0);
    emit_progress(
        "scanned",
        &format!("Found {} files", files_count),
        Some(format!("{:.1} MB", total_size as f64 / 1_048_576.0)),
        Some(0),
        Some(files_count)
    );
    check_cancelled()?;

    // Determine actual backup type based on reality (not just config)
    // If backing up all files, it's FULL, otherwise it's INCREMENTAL
    let actual_backup_type = if files_count == total_files_count {
        "full"
    } else {
        "incr"
    };

    // Generate backup filename with InLocker branding
    let backup_filename = match mode {
        BackupMode::Copy => {
            // Copy mode: folder name (no extension)
            format!("Bkp_InLocker_{}_{}", actual_backup_type, timestamp)
        },
        BackupMode::Compressed => {
            // Compressed: .tar.zst file
            format!("Bkp_InLocker_{}_{}.tar.zst", actual_backup_type, timestamp)
        },
        BackupMode::Encrypted => {
            // Encrypted: .tar.zst.enc file
            format!("Bkp_InLocker_{}_{}.tar.zst.enc", actual_backup_type, timestamp)
        }
    };
    let backup_path = dest_path.join(&backup_filename);

    log::info!("📝 Backup will be saved as: {}", backup_filename);

    // Handle Copy mode separately (direct copy, no TAR, no compression)
    if mode == &BackupMode::Copy {
        log::info!("📋 Copy mode - copying files directly (no TAR, no compression)");
        emit_progress("copying", "Copying files directly", Some(format!("{} files", files_count)), Some(0), Some(files_count));

        // Create backup folder
        fs::create_dir_all(&backup_path)
            .map_err(|e| format!("Failed to create copy destination: {}", e))?;

        // Copy each file preserving structure with cleanup on error/cancellation
        let copy_result = (|| -> Result<usize, String> {
            let mut copied_count = 0;
            for file_path in &files_to_backup {
                // Check for cancellation
                check_cancelled()?;

                let relative_path = file_path.strip_prefix(source_path)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let dest_file = backup_path.join(relative_path);

                // Create parent directories
                if let Some(parent) = dest_file.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                // Copy file
                fs::copy(file_path, &dest_file)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
                copied_count += 1;

                // Emit progress every 10 files
                if copied_count % 10 == 0 {
                    emit_progress("copying", "Copying files directly", Some(format!("{} files", copied_count)), Some(copied_count), Some(files_count));
                }
            }
            Ok(copied_count)
        })();

        match copy_result {
            Ok(copied_count) => {
                log::info!("✅ Copied {} files directly to {}", copied_count, backup_path.display());
            }
            Err(e) => {
                // Cleanup partial copy on error/cancellation
                log::warn!("⚠️  Copy failed, cleaning up partial backup folder...");
                let _ = fs::remove_dir_all(&backup_path);
                return Err(e);
            }
        }

        // Return early - Copy mode doesn't create archive file
        let completed_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        return Ok(BackupJob {
            id: uuid::Uuid::new_v4().to_string(),
            config_id: config_id.to_string(),
            status: BackupStatus::Completed,
            backup_type: backup_type.clone(),
            started_at,
            completed_at: Some(completed_at),
            original_size: Some(total_size),
            compressed_size: Some(total_size), // Same size (no compression)
            files_count: Some(files_count),
            changed_files_count: if matches!(backup_type, BackupType::Incremental) { Some(files_count) } else { None },
            error_message: None,
            backup_path: Some(backup_path.to_string_lossy().to_string()),
            checksum: None, // No checksum for direct copy
        });
    }

    // For Compressed and Encrypted modes: Create TAR archive with streaming compression
    log::info!("📦 Creating TAR archive with streaming compression...");
    emit_progress("creating_tar", "Creating TAR archive", Some(format!("{} files", files_count)), Some(0), Some(files_count));

    // Ensure destination directory exists
    fs::create_dir_all(dest_path).map_err(|e| format!("Failed to create dest dir: {}", e))?;

    // For Compressed mode: Write TAR directly to streaming zstd encoder
    let compressed_size = if mode == &BackupMode::Compressed {
        log::info!("🗜️  Streaming TAR + zstd compression (level 3)...");
        emit_progress("compressing", "Compressing with zstd", Some(format!("{} files", files_count)), Some(0), Some(files_count));

        // Create streaming encoder that writes directly to file
        let output_file = fs::File::create(&backup_path)
            .map_err(|e| {
                let _ = fs::remove_file(&backup_path);
                format!("Failed to create backup file: {}", e)
            })?;

        let result = create_tar_with_streaming_compression(
            &files_to_backup,
            source_path,
            output_file,
            3, // zstd level
            cancel_flag.clone(),
            |current, total| {
                emit_progress(
                    "compressing",
                    "Streaming TAR + zstd",
                    Some(format!("{} files", current)),
                    Some(current),
                    Some(total)
                );
            }
        );

        match result {
            Ok(size) => {
                let compression_ratio = (1.0 - (size as f64 / total_size.max(1) as f64)) * 100.0;
                log::info!("✅ Compressed to {:.2} MB ({:.1}% compression)",
                    size as f64 / 1_048_576.0,
                    compression_ratio
                );
                size
            }
            Err(e) => {
                // CRITICAL: Clean up partial file on error
                let _ = fs::remove_file(&backup_path);
                return Err(e);
            }
        }
    } else {
        // Encrypted mode: Use streaming TAR + zstd + encryption
        log::info!("🗜️  Streaming TAR + zstd + encryption...");
        emit_progress("compressing", "Compressing and encrypting", Some(format!("{} files", files_count)), Some(0), Some(files_count));

        let pwd = password.ok_or("Encryption enabled but no password provided")?;

        // Create temporary file for compressed data (before encryption)
        let temp_compressed = backup_path.with_extension("tmp.zst");

        // Step 1: Stream TAR + zstd to temp file
        let temp_file = fs::File::create(&temp_compressed)
            .map_err(|e| {
                let _ = fs::remove_file(&temp_compressed);
                format!("Failed to create temp file: {}", e)
            })?;

        let compressed_size_result = create_tar_with_streaming_compression(
            &files_to_backup,
            source_path,
            temp_file,
            3, // zstd level
            cancel_flag.clone(),
            |current, total| {
                emit_progress(
                    "compressing",
                    "Streaming TAR + zstd",
                    Some(format!("{} files", current)),
                    Some(current),
                    Some(total)
                );
            }
        );

        let compressed_size = match compressed_size_result {
            Ok(size) => {
                log::info!("✅ Compressed to {:.2} MB", size as f64 / 1_048_576.0);
                size
            }
            Err(e) => {
                // Clean up temp file on error
                let _ = fs::remove_file(&temp_compressed);
                return Err(e);
            }
        };

        check_cancelled()?;

        // Step 2: Encrypt the compressed data
        log::info!("🔐 Encrypting with AES-256-GCM...");
        emit_progress("encrypting", "Encrypting backup", Some(format!("{:.1} MB", compressed_size as f64 / 1_048_576.0)), None, None);

        let encryption_result = (|| -> Result<u64, String> {
            // Read compressed data
            let compressed_data = fs::read(&temp_compressed)
                .map_err(|e| format!("Failed to read compressed data: {}", e))?;

            // Encrypt
            let (encrypted, metadata) = encrypt(&compressed_data, pwd)
                .map_err(|e| format!("Encryption failed: {}", e))?;

            // Embed metadata in file format: [4-byte length][metadata JSON][encrypted data]
            let metadata_json = serde_json::to_vec(&metadata)
                .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

            let metadata_len = metadata_json.len() as u32;
            let mut final_data = Vec::with_capacity(4 + metadata_json.len() + encrypted.len());
            final_data.extend_from_slice(&metadata_len.to_le_bytes());
            final_data.extend_from_slice(&metadata_json);
            final_data.extend_from_slice(&encrypted);

            log::info!("✅ Backup encrypted ({:.2} MB)", final_data.len() as f64 / 1_048_576.0);

            // Write encrypted data to final file
            fs::write(&backup_path, &final_data)
                .map_err(|e| format!("Failed to write encrypted backup: {}", e))?;

            Ok(final_data.len() as u64)
        })();

        // Clean up temp file regardless of success/failure
        let _ = fs::remove_file(&temp_compressed);

        match encryption_result {
            Ok(size) => size,
            Err(e) => {
                // Clean up partial encrypted file on error
                let _ = fs::remove_file(&backup_path);
                return Err(e);
            }
        }
    };

    log::info!("✅ Backup file saved");

    // Calculate checksum
    log::info!("🔒 Calculating SHA-256 checksum...");
    emit_progress("checksum", "Calculating checksum", None, None, None);
    let checksum = match calculate_checksum(&backup_path) {
        Ok(sum) => sum,
        Err(e) => {
            // CRITICAL: Clean up backup file if checksum fails
            let _ = fs::remove_file(&backup_path);
            return Err(e);
        }
    };
    log::info!("✅ Checksum: {}", &checksum[..16]);

    let completed_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let duration = completed_at - started_at;
    log::info!("🎉 Backup completed successfully in {}s", duration);

    Ok(BackupJob {
        id: format!("job-{}", timestamp),
        config_id: String::new(), // Will be set by caller
        status: BackupStatus::Completed,
        backup_type: backup_type.clone(),
        started_at,
        completed_at: Some(completed_at),
        original_size: Some(total_size),
        compressed_size: Some(compressed_size),
        files_count: Some(files_count),
        changed_files_count: if *backup_type == BackupType::Incremental {
            Some(files_count)
        } else {
            None
        },
        error_message: None,
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        checksum: Some(checksum),
    })
}

/// Verify that physical backup files actually exist on disk
/// This prevents using stale manifests when backup files were deleted
pub fn verify_physical_backup_exists(
    dest_path: &Path,
    mode: &BackupMode,
    manifest: &BackupManifest,
) -> Result<bool, String> {
    log::info!("🔍 Verifying physical backup existence...");

    match mode {
        BackupMode::Copy => {
            // For Copy mode: verify backup folder exists and contains ALL files from manifest
            // Find the most recent backup folder
            let backup_folders: Vec<_> = fs::read_dir(dest_path)
                .map_err(|e| format!("Failed to read destination: {}", e))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    entry.path().is_dir() &&
                    (name.starts_with("Bkp_InLocker_") || name.starts_with("backup_")) // Support old format too
                })
                .collect();

            if backup_folders.is_empty() {
                log::warn!("⚠️  No backup folders found in {}", dest_path.display());
                return Ok(false);
            }

            // Get most recent backup folder
            let most_recent = backup_folders
                .iter()
                .max_by_key(|entry| {
                    entry.metadata().ok()
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                });

            if let Some(backup_folder) = most_recent {
                let backup_path = backup_folder.path();
                log::info!("📂 Checking backup folder: {}", backup_path.display());

                // Verify ALL files from manifest exist in backup folder
                let mut missing_files = Vec::new();
                for (relative_path, file_meta) in &manifest.files {
                    let file_path = backup_path.join(relative_path);

                    if !file_path.exists() {
                        missing_files.push(relative_path.clone());
                        continue;
                    }

                    // Also verify file size matches
                    if let Ok(metadata) = fs::metadata(&file_path) {
                        if metadata.len() != file_meta.size {
                            log::warn!("⚠️  File {} size mismatch: expected {}, got {}",
                                relative_path, file_meta.size, metadata.len());
                            missing_files.push(relative_path.clone());
                        }
                    } else {
                        missing_files.push(relative_path.clone());
                    }
                }

                if !missing_files.is_empty() {
                    log::warn!("⚠️  {} files missing or corrupted: {:?}",
                        missing_files.len(), missing_files);
                    return Ok(false);
                }

                log::info!("✅ All {} files verified in backup folder", manifest.files.len());
                Ok(true)
            } else {
                Ok(false)
            }
        },
        BackupMode::Compressed | BackupMode::Encrypted => {
            // For archive modes: verify .tar.zst or .tar.zst.enc file exists
            let extension = match mode {
                BackupMode::Compressed => ".tar.zst",
                BackupMode::Encrypted => ".tar.zst.enc",
                _ => unreachable!(),
            };

            let backup_files: Vec<_> = fs::read_dir(dest_path)
                .map_err(|e| format!("Failed to read destination: {}", e))?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().is_file() &&
                    entry.file_name().to_string_lossy().ends_with(extension)
                })
                .collect();

            if backup_files.is_empty() {
                log::warn!("⚠️  No {} backup files found", extension);
                return Ok(false);
            }

            // Get most recent backup file
            let most_recent = backup_files
                .iter()
                .max_by_key(|entry| {
                    entry.metadata().ok()
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                });

            if let Some(backup_file) = most_recent {
                let backup_path = backup_file.path();

                // Verify file has non-zero size
                if let Ok(metadata) = fs::metadata(&backup_path) {
                    if metadata.len() == 0 {
                        log::warn!("⚠️  Backup file is empty: {}", backup_path.display());
                        return Ok(false);
                    }
                    log::info!("✅ Backup file verified: {} ({} bytes)",
                        backup_path.display(), metadata.len());
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
    }
}

/// Scan all files in a directory recursively
pub fn scan_all_files(source_path: &Path) -> Result<(Vec<PathBuf>, u64), String> {
    let mut files = Vec::new();
    let mut total_size = 0u64;

    fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>, total_size: &mut u64) -> Result<(), String> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                visit_dirs(&path, files, total_size)?;
            } else if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    *total_size += metadata.len();
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    visit_dirs(source_path, &mut files, &mut total_size)?;
    Ok((files, total_size))
}

/// Scan only changed files for incremental backup
fn scan_changed_files(
    source_path: &Path,
    previous_manifest: Option<&BackupManifest>,
) -> Result<(Vec<PathBuf>, u64), String> {
    let (all_files, _) = scan_all_files(source_path)?;

    let mut changed_files = Vec::new();
    let mut total_size = 0u64;

    for file_path in all_files {
        let should_backup = if let Some(manifest) = previous_manifest {
            // Check if file has changed
            let relative_path = file_path
                .strip_prefix(source_path)
                .unwrap()
                .to_string_lossy()
                .to_string();

            if let Some(prev_metadata) = manifest.files.get(&relative_path) {
                // File exists in previous backup - check if modified
                if let Ok(metadata) = fs::metadata(&file_path) {
                    let modified_at = metadata
                        .modified()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    // Changed if modified time is different or size changed
                    modified_at != prev_metadata.modified_at || metadata.len() != prev_metadata.size
                } else {
                    false
                }
            } else {
                // New file - include it
                true
            }
        } else {
            // No previous manifest - backup everything
            true
        };

        if should_backup {
            if let Ok(metadata) = fs::metadata(&file_path) {
                total_size += metadata.len();
                changed_files.push(file_path);
            }
        }
    }

    Ok((changed_files, total_size))
}

/// Create a tar archive from file list (in-memory, for encrypted mode)
fn create_tar_archive<F>(
    files: &[PathBuf],
    base_path: &Path,
    cancel_flag: Option<Arc<AtomicBool>>,
    mut progress_callback: F,
) -> Result<Vec<u8>, String>
where
    F: FnMut(usize, usize),
{
    let mut tar_data = Vec::new();
    let total_files = files.len();
    {
        let mut tar = tar::Builder::new(&mut tar_data);

        for (index, file_path) in files.iter().enumerate() {
            // Check for cancellation every 10 files
            if index % 10 == 0 {
                if let Some(ref flag) = cancel_flag {
                    if flag.load(Ordering::SeqCst) {
                        log::warn!("🚫 TAR creation cancelled by user");
                        return Err("Backup cancelled by user".to_string());
                    }
                }
            }

            let relative_path = file_path
                .strip_prefix(base_path)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;

            tar.append_path_with_name(file_path, relative_path)
                .map_err(|e| format!("Failed to add file to tar: {}", e))?;

            // Emit progress every 100 files or on last file
            if (index + 1) % 100 == 0 || index + 1 == total_files {
                progress_callback(index + 1, total_files);
            }
        }

        tar.finish()
            .map_err(|e| format!("Failed to finalize tar: {}", e))?;
    }
    Ok(tar_data)
}

/// Create TAR archive with streaming zstd compression directly to file
/// This avoids loading the entire archive into memory
fn create_tar_with_streaming_compression<F>(
    files: &[PathBuf],
    base_path: &Path,
    output_file: fs::File,
    compression_level: i32,
    cancel_flag: Option<Arc<AtomicBool>>,
    mut progress_callback: F,
) -> Result<u64, String>
where
    F: FnMut(usize, usize),
{
    let total_files = files.len();

    // Create zstd encoder that writes directly to file
    // This streams: TAR → zstd → file (no intermediate buffers)
    let mut encoder = zstd::stream::write::Encoder::new(output_file, compression_level)
        .map_err(|e| format!("Failed to create zstd encoder: {}", e))?;

    // Create TAR builder that writes to the encoder
    {
        let mut tar = tar::Builder::new(&mut encoder);

        for (index, file_path) in files.iter().enumerate() {
            // Check for cancellation every 10 files
            if index % 10 == 0 {
                if let Some(ref flag) = cancel_flag {
                    if flag.load(Ordering::SeqCst) {
                        log::warn!("🚫 Streaming compression cancelled by user");
                        return Err("Backup cancelled by user".to_string());
                    }
                }
            }

            let relative_path = file_path
                .strip_prefix(base_path)
                .map_err(|e| format!("Failed to get relative path: {}", e))?;

            tar.append_path_with_name(file_path, relative_path)
                .map_err(|e| format!("Failed to add file to streaming tar: {}", e))?;

            // Emit progress every 50 files or on last file
            // More frequent updates since we're streaming
            if (index + 1) % 50 == 0 || index + 1 == total_files {
                progress_callback(index + 1, total_files);
            }
        }

        // Finish TAR archive (flushes to encoder)
        tar.finish()
            .map_err(|e| format!("Failed to finalize streaming tar: {}", e))?;
    } // tar is dropped here, encoder now has all data

    // Finish compression and get final file handle
    let output_file = encoder.finish()
        .map_err(|e| format!("Failed to finish zstd compression: {}", e))?;

    // Sync to disk
    output_file.sync_all()
        .map_err(|e| format!("Failed to sync file to disk: {}", e))?;

    // Get final compressed size
    let metadata = output_file.metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;

    Ok(metadata.len())
}

/// Compress data with zstd (level 3 for balanced performance)
fn compress_with_zstd(data: &[u8]) -> Result<Vec<u8>, String> {
    zstd::encode_all(data, 3).map_err(|e| format!("Failed to compress: {}", e))
}

/// Calculate SHA-256 checksum
fn calculate_checksum(file_path: &Path) -> Result<String, String> {
    use ring::digest::{Context, SHA256};

    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file for checksum: {}", e))?;

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 8192];

    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    let digest = context.finish();
    Ok(hex::encode(digest.as_ref()))
}

/// Build manifest from file list
pub fn build_manifest(config_id: &str, files: &[PathBuf], base_path: &Path) -> Result<BackupManifest, String> {
    let mut file_map = HashMap::new();

    for file_path in files {
        let relative_path = file_path
            .strip_prefix(base_path)
            .unwrap()
            .to_string_lossy()
            .to_string();

        if let Ok(metadata) = fs::metadata(file_path) {
            let modified_at = metadata
                .modified()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // 🔒 SECURITY FIX: Use SHA-256 of file contents instead of size+mtime
            // Previous implementation: format!("{}:{}", metadata.len(), modified_at)
            // Vulnerability: Two files with same size and timestamp would have identical checksums
            // Fix: Calculate actual SHA-256 hash of file contents
            let checksum = calculate_file_checksum(file_path)
                .unwrap_or_else(|e| {
                    log::warn!("Failed to calculate checksum for {:?}: {}, using fallback", file_path, e);
                    // Fallback to size+mtime if file read fails (e.g., permission denied)
                    format!("fallback:{}:{}", metadata.len(), modified_at)
                });

            file_map.insert(
                relative_path.clone(),
                FileMetadata {
                    path: relative_path,
                    size: metadata.len(),
                    modified_at,
                    checksum,
                },
            );
        }
    }

    Ok(BackupManifest {
        config_id: config_id.to_string(),
        created_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        files: file_map,
    })
}

/// Calculate SHA-256 checksum of a file's contents
fn calculate_file_checksum(file_path: &Path) -> Result<String, String> {
    use ring::digest::{Context, SHA256};

    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file for checksum: {}", e))?;

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 8192];

    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    let digest = context.finish();
    Ok(hex::encode(digest.as_ref()))
}

/// Restore a backup from a compressed/encrypted file
///
/// # Security
/// - If backup is encrypted (.enc extension), password is required
/// - Verifies integrity via SHA-256 checksum before restore
/// - Decrypts with AES-256-GCM if needed
pub fn restore_backup(
    backup_file_path: &Path,
    restore_destination: &Path,
    expected_checksum: Option<String>,
    password: Option<&str>,
    app: Option<&tauri::AppHandle>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> Result<RestoreResult, String> {
    let started_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    log::info!("🔄 Starting restore from: {:?}", backup_file_path);

    // Emit initial progress event
    if let Some(app_handle) = app {
        let _ = app_handle.emit("restore:progress", serde_json::json!({
            "stage": "preparing",
            "message": "Preparing to restore...",
            "details": "Loading backup file"
        }));
    }

    // Check cancellation
    if let Some(ref flag) = cancel_flag {
        if flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Restore cancelled by user".to_string());
        }
    }

    // Verify integrity if checksum is provided
    if let Some(expected) = expected_checksum {
        log::info!("🔍 Verifying backup integrity...");
        if let Some(app_handle) = app {
            let _ = app_handle.emit("restore:progress", serde_json::json!({
                "stage": "verifying",
                "message": "Verifying backup integrity...",
                "details": "Calculating checksum"
            }));
        }
        let actual_checksum = calculate_checksum(backup_file_path)?;

        // 🔒 SECURITY NOTE: Constant-time comparison for checksums
        // While checksums are typically public data (not secrets), we use constant-time
        // comparison as a defense-in-depth measure to prevent potential timing attacks.
        // For true secret comparison (passwords, keys), use dedicated crypto libraries.

        fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
            if a.len() != b.len() {
                return false;
            }
            let mut result = 0u8;
            for (x, y) in a.iter().zip(b.iter()) {
                result |= x ^ y;
            }
            result == 0
        }

        let checksum_match = constant_time_eq(
            actual_checksum.as_bytes(),
            expected.as_bytes()
        );

        if !checksum_match {
            log::error!("❌ Checksum mismatch!");
            log::error!("   Expected: {}", &expected[..16]);
            log::error!("   Actual:   {}", &actual_checksum[..16]);
            return Err(format!(
                "Backup file integrity check failed! The file may be corrupted. Expected checksum: {}..., Got: {}...",
                &expected[..16],
                &actual_checksum[..16]
            ));
        }
        log::info!("✅ Integrity verified - checksum matches");
    } else {
        log::warn!("⚠️  No checksum provided - skipping integrity verification");
    }

    // Check cancellation
    if let Some(ref flag) = cancel_flag {
        if flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Restore cancelled by user".to_string());
        }
    }

    // Read backup file (possibly encrypted)
    if let Some(app_handle) = app {
        let _ = app_handle.emit("restore:progress", serde_json::json!({
            "stage": "reading",
            "message": "Reading backup file...",
            "details": "Loading data"
        }));
    }

    let file_data = fs::read(backup_file_path)
        .map_err(|e| format!("Failed to read backup file: {}", e))?;

    // Check if file is encrypted (based on extension)
    let is_encrypted = backup_file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "enc")
        .unwrap_or(false);

    // Decrypt if needed
    let compressed_data = if is_encrypted {
        log::info!("🔓 Decrypting backup...");
        if let Some(app_handle) = app {
            let _ = app_handle.emit("restore:progress", serde_json::json!({
                "stage": "decrypting",
                "message": "Decrypting backup...",
                "details": "Using AES-256-GCM (cannot be interrupted)"
            }));
        }

        // Check cancellation before starting expensive operation
        if let Some(ref flag) = cancel_flag {
            if flag.load(std::sync::atomic::Ordering::SeqCst) {
                return Err("Restore cancelled by user".to_string());
            }
        }

        let pwd = password.ok_or_else(|| {
            "Backup is encrypted but no password provided. Please provide the password used during backup.".to_string()
        })?;

        // Extract metadata from file: [4-byte length][metadata JSON][encrypted data]
        if file_data.len() < 4 {
            return Err("Invalid encrypted file: too short".to_string());
        }

        let metadata_len = u32::from_le_bytes([
            file_data[0],
            file_data[1],
            file_data[2],
            file_data[3],
        ]) as usize;

        if file_data.len() < 4 + metadata_len {
            return Err("Invalid encrypted file: metadata truncated".to_string());
        }

        let metadata_json = &file_data[4..4 + metadata_len];
        let metadata: EncryptionMetadata = serde_json::from_slice(metadata_json)
            .map_err(|e| format!("Failed to parse encryption metadata: {}", e))?;

        let encrypted_data = &file_data[4 + metadata_len..];

        // NOTE: Decryption is blocking and cannot be interrupted
        // We check for cancellation immediately after it completes
        let decrypted = decrypt(encrypted_data, pwd, &metadata)
            .map_err(|e| format!("Decryption failed: {}. Please verify your password is correct.", e))?;

        // Check cancellation immediately after decryption
        if let Some(ref flag) = cancel_flag {
            if flag.load(std::sync::atomic::Ordering::SeqCst) {
                log::warn!("⚠️  Restore cancelled after decryption completed");
                return Err("Restore cancelled by user".to_string());
            }
        }

        log::info!("✅ Backup decrypted successfully");
        decrypted
    } else {
        if password.is_some() {
            log::warn!("⚠️  Password provided but backup is not encrypted - ignoring password");
        }
        file_data
    };

    // Decompress with zstd (only if compressed)
    // Check file extension to determine if decompression is needed
    let file_name = backup_file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let is_compressed = file_name.contains(".zst");

    // Check cancellation
    if let Some(ref flag) = cancel_flag {
        if flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Restore cancelled by user".to_string());
        }
    }

    let tar_data = if is_compressed {
        log::info!("📦 Decompressing backup...");
        if let Some(app_handle) = app {
            let _ = app_handle.emit("restore:progress", serde_json::json!({
                "stage": "decompressing",
                "message": "Decompressing backup...",
                "details": "Using zstd (cannot be interrupted)"
            }));
        }

        // NOTE: zstd decompression is blocking and cannot be interrupted
        // We check for cancellation immediately after it completes
        let decompressed = decompress_with_zstd(&compressed_data)?;

        // Check cancellation immediately after decompression
        if let Some(ref flag) = cancel_flag {
            if flag.load(std::sync::atomic::Ordering::SeqCst) {
                log::warn!("⚠️  Restore cancelled after decompression completed");
                return Err("Restore cancelled by user".to_string());
            }
        }

        decompressed
    } else {
        log::info!("📋 Copy mode - no decompression needed");
        compressed_data
    };

    // Check cancellation
    if let Some(ref flag) = cancel_flag {
        if flag.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Restore cancelled by user".to_string());
        }
    }

    // Extract tar archive
    log::info!("📂 Extracting files...");
    if let Some(app_handle) = app {
        let _ = app_handle.emit("restore:progress", serde_json::json!({
            "stage": "extracting",
            "message": "Extracting files...",
            "details": "Unpacking archive"
        }));
    }
    let files_extracted = extract_tar_archive(&tar_data, restore_destination, app, cancel_flag.clone())?;

    let completed_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    log::info!("✅ Restore completed successfully - {} files extracted", files_extracted);

    Ok(RestoreResult {
        success: true,
        message: format!("Restored {} files successfully", files_extracted),
        files_count: files_extracted,
        started_at,
        completed_at,
    })
}

/// Decompress data with zstd
fn decompress_with_zstd(data: &[u8]) -> Result<Vec<u8>, String> {
    zstd::decode_all(data).map_err(|e| format!("Failed to decompress: {}", e))
}

/// Extract tar archive to destination
fn extract_tar_archive(
    tar_data: &[u8],
    destination: &Path,
    app: Option<&tauri::AppHandle>,
    cancel_flag: Option<Arc<AtomicBool>>,
) -> Result<usize, String> {
    use std::io::Cursor;

    let cursor = Cursor::new(tar_data);
    let mut archive = tar::Archive::new(cursor);

    // Ensure destination exists
    fs::create_dir_all(destination)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    // Extract all files
    let mut count = 0;
    for entry_result in archive.entries().map_err(|e| format!("Failed to read tar entries: {}", e))? {
        // Check cancellation
        if let Some(ref flag) = cancel_flag {
            if flag.load(std::sync::atomic::Ordering::SeqCst) {
                return Err("Restore cancelled by user".to_string());
            }
        }

        let mut entry = entry_result.map_err(|e| format!("Failed to read tar entry: {}", e))?;

        let path = destination.join(entry.path().map_err(|e| format!("Invalid path in tar: {}", e))?);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        entry
            .unpack(&path)
            .map_err(|e| format!("Failed to extract file: {}", e))?;

        count += 1;

        // Emit progress every 100 files
        if count % 100 == 0 {
            if let Some(app_handle) = app {
                let _ = app_handle.emit("restore:progress", serde_json::json!({
                    "stage": "extracting",
                    "message": "Extracting files...",
                    "details": format!("{} files extracted", count),
                    "current": count
                }));
            }
        }
    }

    Ok(count)
}

/// List available backups in a destination folder
pub fn list_backups(destination_path: &Path) -> Result<Vec<BackupInfo>, String> {
    let mut backups = Vec::new();

    if !destination_path.exists() {
        return Ok(backups);
    }

    for entry in fs::read_dir(destination_path)
        .map_err(|e| format!("Failed to read destination directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // Include .tar.zst and .tar.zst.enc files (compressed and encrypted backups)
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.ends_with(".tar.zst") || filename_str.ends_with(".tar.zst.enc") {
                if let Ok(metadata) = fs::metadata(&path) {
                    let created_at = metadata
                        .modified()
                        .or_else(|_| metadata.created())
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    backups.push(BackupInfo {
                        filename: filename_str.to_string(),
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                        created_at,
                    });
                }
            }
        }
    }

    // Sort by creation time (newest first)
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(backups)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
    pub files_count: usize,
    pub started_at: i64,
    pub completed_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_checksum_calculation() {
        // Create temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_checksum.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test data for checksum").unwrap();

        // Calculate checksum
        let checksum = calculate_checksum(&test_file).unwrap();

        // Verify checksum is hex string with correct length (SHA-256 = 64 chars)
        assert_eq!(checksum.len(), 64);
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));

        // Calculate again - should be same
        let checksum2 = calculate_checksum(&test_file).unwrap();
        assert_eq!(checksum, checksum2);

        // Cleanup
        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_compression_decompression() {
        // Create test data
        let test_data = b"Hello World! This is test data for compression.".repeat(100);

        // Compress
        let compressed = compress_with_zstd(&test_data).unwrap();

        // Verify compressed is smaller
        assert!(compressed.len() < test_data.len());

        // Decompress
        let decompressed = decompress_with_zstd(&compressed).unwrap();

        // Verify data matches
        assert_eq!(&test_data[..], &decompressed[..]);
    }

    #[test]
    fn test_manifest_operations() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("manifest_test.txt");
        fs::write(&test_file, b"test").unwrap();

        let files = vec![test_file.clone()];
        let manifest = build_manifest("test-id", &files, &temp_dir).unwrap();

        // Verify manifest contains file
        assert_eq!(manifest.config_id, "test-id");
        assert!(manifest.files.contains_key("manifest_test.txt"));

        // Cleanup
        fs::remove_file(&test_file).ok();
    }
}
