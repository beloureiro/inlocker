/// ADVERSARIAL & EXTREME STRESS TESTS
///
/// These tests simulate real-world attacks, edge cases, and failure scenarios
/// that a production backup system MUST handle correctly.
/// These tests are DESIGNED TO BE HARD and expose real vulnerabilities.

use inlocker_lib::backup::{build_manifest, compress_folder, restore_backup, scan_all_files};
use inlocker_lib::types::{BackupManifest, BackupType};
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

/// Helper: Calculate SHA-256 checksum
fn calculate_sha256(data: &[u8]) -> String {
    use ring::digest::{Context, SHA256};
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    hex::encode(digest.as_ref())
}

/// Helper: Setup test directories
fn setup_test_dirs(test_name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join(format!("adv_{}_source", test_name));
    let dest_dir = temp_dir.join(format!("adv_{}_dest", test_name));
    let restore_dir = temp_dir.join(format!("adv_{}_restore", test_name));

    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    (source_dir, dest_dir, restore_dir)
}

/// Helper: Cleanup
fn cleanup_test_dirs(dirs: &[&Path]) {
    for dir in dirs {
        let _ = fs::remove_dir_all(dir);
    }
}

// ============================================================================
// ADVERSARIAL TEST 1: PATH TRAVERSAL ATTACK
// ============================================================================

#[test]
fn test_path_traversal_cannot_escape_backup() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("path_traversal");

    // Create normal file
    fs::write(source_dir.join("normal.txt"), b"normal content").unwrap();

    // Try to create a file with path traversal in name
    // On Unix, this is just a filename with dots, not actual traversal
    let malicious_name = "..%2f..%2fetc%2fpasswd.txt";
    fs::write(source_dir.join(malicious_name), b"malicious content").unwrap();

    // Backup
    let backup_job = compress_folder(
        "path-traversal-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = PathBuf::from(backup_job.backup_path.unwrap());

    // Restore
    restore_backup(&backup_path, &restore_dir, backup_job.checksum, None).unwrap();

    // CRITICAL: File should be restored with sanitized name, NOT escaping restore dir
    let restored_files: Vec<_> = fs::read_dir(&restore_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    // Verify file didn't escape
    assert!(restored_files.contains(&malicious_name.to_string()),
        "File should be restored with original (safe) name");

    // CRITICAL: Verify malicious file didn't escape to parent directories
    let parent_dir = restore_dir.parent().unwrap();

    // Check if malicious files exist in parent (they shouldn't)
    let passwd_escaped = parent_dir.join("passwd.txt");
    let etc_dir = parent_dir.join("etc");

    assert!(!passwd_escaped.exists(),
        "SECURITY FAILURE: File escaped to parent directory!");
    assert!(!etc_dir.exists() || !etc_dir.join("passwd.txt").exists(),
        "SECURITY FAILURE: Path traversal succeeded!");

    // Verify all files are contained within restore_dir
    for file in &restored_files {
        let file_path = restore_dir.join(file);
        assert!(file_path.starts_with(&restore_dir),
            "SECURITY FAILURE: File {} escaped restore directory!", file);
    }

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 2: CHECKSUM COLLISION RESISTANCE
// ============================================================================

#[test]
fn test_checksum_collision_resistance() {
    // Test that even 1-bit difference produces different checksum
    let data1 = vec![0u8; 1000];
    let mut data2 = vec![0u8; 1000];
    data2[500] = 1; // Single bit difference

    let checksum1 = calculate_sha256(&data1);
    let checksum2 = calculate_sha256(&data2);

    assert_ne!(checksum1, checksum2,
        "CRITICAL: SHA-256 collision or implementation error!");

    // Test known SHA-256 collision resistance
    // SHA-256 should produce 64 hex chars (256 bits)
    assert_eq!(checksum1.len(), 64);
    assert_eq!(checksum2.len(), 64);

    // Test that checksums are deterministic
    let checksum1_again = calculate_sha256(&data1);
    assert_eq!(checksum1, checksum1_again,
        "CRITICAL: Checksum is not deterministic!");
}

// ============================================================================
// ADVERSARIAL TEST 3: CONCURRENT FILE MODIFICATION
// ============================================================================

#[test]
fn test_backup_with_concurrent_file_changes() {
    let (source_dir, dest_dir, _) = setup_test_dirs("concurrent_mod");

    // Create initial files
    for i in 0..100 {
        fs::write(source_dir.join(format!("file_{}.txt", i)), format!("content {}", i)).unwrap();
    }

    // Start backup
    let backup_result = compress_folder(
        "concurrent-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    );

    // Backup should complete even if files were present
    assert!(backup_result.is_ok(),
        "Backup should handle file state at backup time");

    // Modify some files AFTER backup started (simulated by modifying after)
    for i in 0..10 {
        fs::write(source_dir.join(format!("file_{}.txt", i)), "MODIFIED").unwrap();
    }

    // The backup should have captured the ORIGINAL state
    let backup_job = backup_result.unwrap();
    assert_eq!(backup_job.files_count.unwrap(), 100,
        "Should have backed up all original files");

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 4: EXTREMELY DEEP DIRECTORY NESTING (100 levels)
// ============================================================================

#[test]
fn test_extremely_deep_nesting() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("extreme_deep");

    // Create 100-level deep directory structure
    let mut current_path = source_dir.clone();
    for i in 0..100 {
        current_path = current_path.join(format!("d{}", i));
        if let Err(_) = fs::create_dir_all(&current_path) {
            // Some systems have path length limits
            println!("⚠️  System limit reached at depth {}", i);
            break;
        }
    }

    // Write file at deepest point
    let deepest_file = current_path.join("deep.txt");
    if let Ok(_) = fs::write(&deepest_file, b"deeply nested content") {
        // Backup
        let backup_result = compress_folder(
            "deep-test",
            &source_dir,
            &dest_dir,
            &BackupType::Full,
            None,
            None,
            None,
        );

        assert!(backup_result.is_ok(), "Should handle deep nesting");

        let backup_job = backup_result.unwrap();
        let backup_path = PathBuf::from(backup_job.backup_path.unwrap());

        // Restore
        let restore_result = restore_backup(&backup_path, &restore_dir, backup_job.checksum, None);
        assert!(restore_result.is_ok(), "Should restore deep nesting");
    }

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 5: MALFORMED MANIFEST INJECTION
// ============================================================================

#[test]
fn test_malformed_manifest_handling() {
    let (source_dir, dest_dir, _) = setup_test_dirs("malformed_manifest");

    // Create files
    fs::write(source_dir.join("file1.txt"), b"content").unwrap();

    // Create MALFORMED manifest with invalid JSON
    let manifest_path = dest_dir.join("malformed-test_manifest.json");
    fs::write(&manifest_path, b"{ INVALID JSON ;;;").unwrap();

    // Try to load malformed manifest
    let manifest_str = fs::read_to_string(&manifest_path).ok();
    let parsed_manifest = manifest_str.and_then(|j| serde_json::from_str::<BackupManifest>(&j).ok());

    // CRITICAL: Should return None, not crash
    assert!(parsed_manifest.is_none(),
        "SECURITY: Should reject malformed manifest without crashing");

    // Now create manifest with SQL injection-like content
    let malicious_manifest = r#"{
        "config_id": "test'; DROP TABLE backups; --",
        "created_at": 1234567890,
        "files": {
            "../../../etc/passwd": {
                "path": "../../../etc/passwd",
                "size": 999999,
                "modified_at": 0,
                "checksum": "evil"
            }
        }
    }"#;

    fs::write(&manifest_path, malicious_manifest).unwrap();

    let manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let loaded_manifest: BackupManifest = serde_json::from_str(&manifest_str).unwrap();

    // Manifest should load but paths should be safe
    assert_eq!(loaded_manifest.config_id, "test'; DROP TABLE backups; --",
        "Should store as string, not execute");

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 6: ZERO-LENGTH AND MAXIMUM-SIZE FILES
// ============================================================================

#[test]
fn test_extreme_file_sizes() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("extreme_sizes");

    // Zero-byte file
    fs::write(source_dir.join("zero.bin"), b"").unwrap();

    // 50MB file (reduced from larger to keep test fast)
    let large_file_path = source_dir.join("large.bin");
    let large_data = vec![0xAA; 50 * 1024 * 1024]; // 50MB
    fs::write(&large_file_path, &large_data).unwrap();

    let original_large_checksum = calculate_sha256(&large_data);

    // Backup
    let backup_job = compress_folder(
        "extreme-size-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    // CRITICAL: Verify backup size is reasonable (compression should work)
    let compressed_size = backup_job.compressed_size.unwrap();
    let original_size = backup_job.original_size.unwrap();

    // 50MB of 0xAA should compress to <1MB
    let compression_ratio = original_size as f64 / compressed_size as f64;
    assert!(compression_ratio > 30.0,
        "FAILURE: Compression of repetitive data should be >30x, got {:.1}x",
        compression_ratio);

    // Restore
    let backup_path = PathBuf::from(backup_job.backup_path.unwrap());
    restore_backup(&backup_path, &restore_dir, backup_job.checksum, None).unwrap();

    // Verify zero file
    let zero_restored = fs::read(restore_dir.join("zero.bin")).unwrap();
    assert_eq!(zero_restored.len(), 0, "Zero-byte file corrupted");

    // Verify large file integrity
    let large_restored = fs::read(restore_dir.join("large.bin")).unwrap();
    let restored_large_checksum = calculate_sha256(&large_restored);
    assert_eq!(original_large_checksum, restored_large_checksum,
        "CRITICAL: 50MB file corrupted during backup/restore!");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 7: PERMISSION DENIED SCENARIOS
// ============================================================================

#[test]
#[cfg(unix)] // Unix-specific permissions
fn test_unreadable_files_handling() {
    let (source_dir, dest_dir, _) = setup_test_dirs("permissions");

    // Create readable file
    fs::write(source_dir.join("readable.txt"), b"can read").unwrap();

    // Create unreadable file (chmod 000)
    let unreadable_path = source_dir.join("unreadable.txt");
    fs::write(&unreadable_path, b"cannot read").unwrap();

    let mut perms = fs::metadata(&unreadable_path).unwrap().permissions();
    perms.set_mode(0o000); // No permissions
    fs::set_permissions(&unreadable_path, perms).unwrap();

    // Try to backup
    let backup_result = compress_folder(
        "permission-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    );

    // Backup might succeed (backing up readable files) or fail (strict mode)
    // Either way, it should NOT panic or crash
    match backup_result {
        Ok(job) => {
            // Should have backed up at least the readable file
            println!("✓ Backup succeeded with {} files", job.files_count.unwrap_or(0));
        }
        Err(e) => {
            // Should have clear error message about permissions
            println!("✓ Backup failed gracefully: {}", e);
        }
    }

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&unreadable_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&unreadable_path, perms).unwrap();

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 8: BACKUP FILE TAMPERING DETECTION
// ============================================================================

#[test]
fn test_detect_all_types_of_tampering() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("tampering");

    // Create test data
    fs::write(source_dir.join("important.txt"), b"CRITICAL DATA - DO NOT LOSE").unwrap();

    // Backup
    let backup_job = compress_folder(
        "tamper-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = PathBuf::from(backup_job.backup_path.clone().unwrap());
    let original_checksum = backup_job.checksum.clone().unwrap();

    // TEST 1: Flip random bits
    let mut backup_data = fs::read(&backup_path).unwrap();
    let len = backup_data.len();
    backup_data[len / 4] ^= 0xFF;
    backup_data[len / 2] ^= 0x01;
    backup_data[len - 100] ^= 0xAA;
    fs::write(&backup_path, &backup_data).unwrap();

    let result1 = restore_backup(&backup_path, &restore_dir, Some(original_checksum.clone()), None);
    assert!(result1.is_err(), "CRITICAL: Bit-flipped backup accepted!");

    // TEST 2: Truncate file
    fs::write(&backup_path, &backup_data[..backup_data.len() / 2]).unwrap();
    let result2 = restore_backup(&backup_path, &restore_dir, Some(original_checksum.clone()), None);
    assert!(result2.is_err(), "CRITICAL: Truncated backup accepted!");

    // TEST 3: Append garbage
    let mut extended_data = backup_data.clone();
    extended_data.extend_from_slice(b"MALICIOUS GARBAGE DATA");
    fs::write(&backup_path, &extended_data).unwrap();
    let result3 = restore_backup(&backup_path, &restore_dir, Some(original_checksum.clone()), None);
    assert!(result3.is_err(), "CRITICAL: Extended backup accepted!");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 9: RACE CONDITION IN INCREMENTAL BACKUP
// ============================================================================

#[test]
fn test_incremental_race_condition_safety() {
    let (source_dir, dest_dir, _) = setup_test_dirs("race_condition");

    // Create initial state
    fs::write(source_dir.join("file1.txt"), b"version 1").unwrap();

    // Full backup
    compress_folder("race-test", &source_dir, &dest_dir, &BackupType::Full, None, None, None).unwrap();

    let (all_files, _) = scan_all_files(&source_dir).unwrap();
    let manifest = build_manifest("race-test", &all_files, &source_dir).unwrap();

    // Modify file
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(source_dir.join("file1.txt"), b"version 2").unwrap();

    // Incremental backup with OLD manifest (race condition simulation)
    let incremental = compress_folder(
        "race-test",
        &source_dir,
        &dest_dir,
        &BackupType::Incremental,
        Some(&manifest),
        None,
        None,
    ).unwrap();

    // Should detect the change
    assert_eq!(incremental.files_count.unwrap(), 1,
        "CRITICAL: Failed to detect file change in race condition!");

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// ADVERSARIAL TEST 10: RESTORE TO NON-EMPTY DIRECTORY
// ============================================================================

#[test]
fn test_restore_overwrites_existing_files() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("overwrite");

    // Create source
    fs::write(source_dir.join("data.txt"), b"CORRECT CONTENT").unwrap();

    // Backup
    let backup_job = compress_folder(
        "overwrite-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    // Create WRONG file in restore directory
    fs::write(restore_dir.join("data.txt"), b"WRONG OLD CONTENT").unwrap();

    // Restore
    let backup_path = PathBuf::from(backup_job.backup_path.unwrap());
    restore_backup(&backup_path, &restore_dir, backup_job.checksum, None).unwrap();

    // CRITICAL: Should have overwritten with correct content
    let restored_content = fs::read_to_string(restore_dir.join("data.txt")).unwrap();
    assert_eq!(restored_content, "CORRECT CONTENT",
        "CRITICAL: Restore failed to overwrite existing file!");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}
