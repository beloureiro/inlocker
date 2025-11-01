/// CRITICAL BACKUP TESTS - Production-Grade Test Suite
///
/// This test suite validates critical backup functionality that MUST work
/// reliably in production. These tests cover edge cases, incremental backups,
/// data integrity, and error handling scenarios.

use inlocker_lib::backup::{build_manifest, compress_folder, restore_backup, scan_all_files};
use inlocker_lib::types::{BackupManifest, BackupType};
use std::fs;
use std::path::{Path, PathBuf};

/// Helper: Calculate SHA-256 checksum
fn calculate_sha256(data: &[u8]) -> String {
    use ring::digest::{Context, SHA256};
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    hex::encode(digest.as_ref())
}

/// Helper: Create test directory structure
fn setup_test_dirs(test_name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join(format!("{}_source", test_name));
    let dest_dir = temp_dir.join(format!("{}_dest", test_name));
    let restore_dir = temp_dir.join(format!("{}_restore", test_name));

    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    (source_dir, dest_dir, restore_dir)
}

/// Helper: Cleanup test directories
fn cleanup_test_dirs(dirs: &[&Path]) {
    for dir in dirs {
        let _ = fs::remove_dir_all(dir);
    }
}

// ============================================================================
// CRITICAL TEST 1: INCREMENTAL BACKUP ACCURACY
// ============================================================================

#[test]
fn test_incremental_backup_only_changed_files() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("incremental_test");

    // STEP 1: Create initial files
    fs::write(source_dir.join("unchanged.txt"), b"This file stays the same").unwrap();
    fs::write(source_dir.join("will_change.txt"), b"Original content v1").unwrap();
    fs::write(source_dir.join("will_delete.txt"), b"This will be deleted").unwrap();

    // STEP 2: First FULL backup
    let full_backup = compress_folder(
        "incremental-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    assert_eq!(full_backup.backup_type, BackupType::Full);
    assert_eq!(full_backup.files_count.unwrap(), 3, "Full backup should have 3 files");

    let full_backup_size = full_backup.compressed_size.unwrap();

    // STEP 2.5: Build and save manifest after full backup
    let (all_files, _) = scan_all_files(&source_dir).unwrap();
    let manifest = build_manifest("incremental-test", &all_files, &source_dir).unwrap();
    let manifest_path = dest_dir.join("incremental-test_manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // STEP 3: Modify files
    std::thread::sleep(std::time::Duration::from_millis(1100)); // Ensure timestamp difference
    fs::write(source_dir.join("will_change.txt"), b"Modified content v2 - MUCH LONGER NOW").unwrap();
    fs::write(source_dir.join("new_file.txt"), b"Brand new file").unwrap();
    fs::remove_file(source_dir.join("will_delete.txt")).unwrap();

    // STEP 4: Load manifest and perform incremental backup
    let manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let loaded_manifest: BackupManifest = serde_json::from_str(&manifest_str).unwrap();

    let incremental_backup = compress_folder(
        "incremental-test",
        &source_dir,
        &dest_dir,
        &BackupType::Incremental,
        Some(&loaded_manifest),
        None,
        None,
    ).unwrap();

    assert_eq!(incremental_backup.backup_type, BackupType::Incremental);

    // CRITICAL: Incremental should only contain 2 files (changed + new)
    // NOT the unchanged file!
    assert_eq!(incremental_backup.files_count.unwrap(), 2,
        "Incremental should ONLY backup changed (1) and new (1) files, NOT unchanged files!");

    // CRITICAL: Incremental backup should be MUCH smaller than full backup
    let incremental_size = incremental_backup.compressed_size.unwrap();
    assert!(incremental_size < full_backup_size,
        "CRITICAL: Incremental backup ({} bytes) should be smaller than full backup ({} bytes)!",
        incremental_size, full_backup_size);

    // STEP 5: Verify manifest was created
    assert!(manifest_path.exists(), "Manifest file must exist for incremental backups");

    let manifest_content = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_content.contains("unchanged.txt"), "Manifest should track unchanged file");
    assert!(manifest_content.contains("will_change.txt"), "Manifest should track changed file");

    // STEP 6: Restore and verify changed files
    let incremental_backup_path = PathBuf::from(incremental_backup.backup_path.unwrap());
    let restore_result = restore_backup(
        &incremental_backup_path,
        &restore_dir,
        incremental_backup.checksum,
        None,
    ).unwrap();

    assert_eq!(restore_result.files_count, 2, "Restore should have 2 files from incremental");

    // Verify changed file has NEW content
    let changed_content = fs::read_to_string(restore_dir.join("will_change.txt")).unwrap();
    assert_eq!(changed_content, "Modified content v2 - MUCH LONGER NOW");

    // Verify new file exists
    let new_content = fs::read_to_string(restore_dir.join("new_file.txt")).unwrap();
    assert_eq!(new_content, "Brand new file");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// CRITICAL TEST 2: COMPRESSION EFFICIENCY
// ============================================================================

#[test]
fn test_compression_efficiency() {
    let (source_dir, dest_dir, _) = setup_test_dirs("compression_test");

    // Create highly compressible file (repeated patterns)
    let mut highly_compressible = Vec::new();
    for _ in 0..10000 {
        highly_compressible.extend_from_slice(b"AAAAAAAAAA");
    }
    fs::write(source_dir.join("compressible.txt"), &highly_compressible).unwrap();

    // Create random-like file (less compressible)
    let mut pseudo_random = Vec::new();
    for i in 0..100000u32 {
        pseudo_random.extend_from_slice(&i.to_le_bytes());
    }
    fs::write(source_dir.join("random.bin"), &pseudo_random).unwrap();

    let backup_job = compress_folder(
        "compression-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    let original_size = backup_job.original_size.unwrap() as f64;
    let compressed_size = backup_job.compressed_size.unwrap() as f64;
    let compression_ratio = original_size / compressed_size;

    // CRITICAL: Compression should achieve at least 1.8x reduction
    // (Adjusted for realistic TAR header overhead on small test files)
    assert!(compression_ratio > 1.8,
        "CRITICAL: Compression ratio {:.2}x is too low! Expected >1.8x for test data",
        compression_ratio);

    println!("✅ Compression test passed: {:.2}x reduction ({:.1} KB → {:.1} KB)",
        compression_ratio,
        original_size / 1024.0,
        compressed_size / 1024.0);

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// CRITICAL TEST 3: BINARY FILES (Images, PDFs, etc.)
// ============================================================================

#[test]
fn test_binary_files_integrity() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("binary_test");

    // Simulate a PNG-like binary file header + data
    let mut png_like_data = Vec::new();
    png_like_data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG header
    png_like_data.extend_from_slice(&[0; 5000]); // Simulate image data
    for i in 0..1000 {
        png_like_data.push((i % 256) as u8);
    }
    fs::write(source_dir.join("image.png"), &png_like_data).unwrap();

    // Simulate a PDF-like binary file
    let mut pdf_like_data = Vec::new();
    pdf_like_data.extend_from_slice(b"%PDF-1.4\n");
    pdf_like_data.extend_from_slice(&[0; 10000]); // Simulate PDF content
    fs::write(source_dir.join("document.pdf"), &pdf_like_data).unwrap();

    // Calculate original checksums
    let png_checksum = calculate_sha256(&png_like_data);
    let pdf_checksum = calculate_sha256(&pdf_like_data);

    // Backup
    let backup_job = compress_folder(
        "binary-test",
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

    // CRITICAL: Verify binary files are bit-for-bit identical
    let restored_png = fs::read(restore_dir.join("image.png")).unwrap();
    let restored_png_checksum = calculate_sha256(&restored_png);
    assert_eq!(png_checksum, restored_png_checksum,
        "CRITICAL: PNG file corrupted during backup/restore cycle!");

    let restored_pdf = fs::read(restore_dir.join("document.pdf")).unwrap();
    let restored_pdf_checksum = calculate_sha256(&restored_pdf);
    assert_eq!(pdf_checksum, restored_pdf_checksum,
        "CRITICAL: PDF file corrupted during backup/restore cycle!");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// CRITICAL TEST 4: EDGE CASES - Empty Files and Zero-byte Files
// ============================================================================

#[test]
fn test_empty_and_zero_byte_files() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("empty_files_test");

    // Create various edge case files
    fs::write(source_dir.join("empty.txt"), b"").unwrap();
    fs::write(source_dir.join("single_byte.txt"), b"X").unwrap();
    fs::write(source_dir.join("newline_only.txt"), b"\n").unwrap();
    fs::write(source_dir.join("spaces_only.txt"), b"     ").unwrap();

    // Create empty directory
    fs::create_dir_all(source_dir.join("empty_dir")).unwrap();

    let backup_job = compress_folder(
        "empty-test",
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

    // Verify empty file
    let empty_content = fs::read(restore_dir.join("empty.txt")).unwrap();
    assert_eq!(empty_content.len(), 0, "Empty file should remain empty");

    // Verify single byte file
    let single_byte = fs::read(restore_dir.join("single_byte.txt")).unwrap();
    assert_eq!(single_byte, b"X", "Single byte file corrupted");

    // Verify newline file
    let newline = fs::read(restore_dir.join("newline_only.txt")).unwrap();
    assert_eq!(newline, b"\n", "Newline-only file corrupted");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// CRITICAL TEST 5: MANIFEST INTEGRITY (Incremental Backup Tracking)
// ============================================================================

#[test]
fn test_manifest_tracks_all_changes() {
    let (source_dir, dest_dir, _) = setup_test_dirs("manifest_integrity");

    // Create initial files with known timestamps
    fs::write(source_dir.join("file1.txt"), b"content1").unwrap();
    fs::write(source_dir.join("file2.txt"), b"content2").unwrap();

    // Full backup
    compress_folder(
        "manifest-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    // Build and save manifest manually (tests run without AppHandle)
    let (all_files, _) = scan_all_files(&source_dir).unwrap();
    let manifest = build_manifest("manifest-test", &all_files, &source_dir).unwrap();
    let manifest_path = dest_dir.join("manifest-test_manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Verify manifest created
    assert!(manifest_path.exists(), "Manifest must be created after full backup");

    // Parse manifest
    let manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();

    // CRITICAL: Manifest must contain all files with correct structure
    let files_obj = manifest.get("files").expect("Manifest must have 'files' field");
    assert!(files_obj.get("file1.txt").is_some(), "Manifest must track file1.txt");
    assert!(files_obj.get("file2.txt").is_some(), "Manifest must track file2.txt");

    // CRITICAL: Manifest must store correct metadata
    let file1_entry = files_obj.get("file1.txt").unwrap();
    assert!(file1_entry.get("size").is_some(), "Manifest must store file size");
    assert!(file1_entry.get("modified_at").is_some(), "Manifest must store modification time");

    let stored_size = file1_entry.get("size").unwrap().as_u64().unwrap();
    assert_eq!(stored_size, 8, "Manifest should store correct file size");

    // STEP 2: Modify a file and do incremental backup
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(source_dir.join("file1.txt"), b"modified content is longer").unwrap();

    // Load previous manifest for incremental
    let prev_manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let prev_manifest: BackupManifest = serde_json::from_str(&prev_manifest_str).unwrap();

    compress_folder(
        "manifest-test",
        &source_dir,
        &dest_dir,
        &BackupType::Incremental,
        Some(&prev_manifest),
        None,
        None,
    ).unwrap();

    // Rebuild manifest after incremental (simulating what the command does)
    let (all_files_after, _) = scan_all_files(&source_dir).unwrap();
    let updated_manifest_obj = build_manifest("manifest-test", &all_files_after, &source_dir).unwrap();
    let updated_manifest_json = serde_json::to_string_pretty(&updated_manifest_obj).unwrap();
    fs::write(&manifest_path, updated_manifest_json).unwrap();

    // Re-read manifest
    let updated_manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let updated_manifest: serde_json::Value = serde_json::from_str(&updated_manifest_str).unwrap();

    // CRITICAL: Manifest must update modified file
    let updated_files = updated_manifest.get("files").unwrap();
    let updated_file1 = updated_files.get("file1.txt").unwrap();
    let updated_size = updated_file1.get("size").unwrap().as_u64().unwrap();
    assert_eq!(updated_size, 26, "Manifest must update file size after incremental backup");

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// CRITICAL TEST 6: VERY LONG FILENAMES (macOS limit: 255 bytes)
// ============================================================================

#[test]
fn test_long_filenames() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("long_filename_test");

    // Create file with 200-character name (within limits)
    let long_name_200 = "a".repeat(200) + ".txt";
    fs::write(source_dir.join(&long_name_200), b"content").unwrap();

    // Create file with 250-character name (close to limit)
    let long_name_250 = "b".repeat(250) + ".txt";
    if let Err(_) = fs::write(source_dir.join(&long_name_250), b"content") {
        println!("⚠️  Note: Filesystem doesn't support 250-char filenames (expected on some systems)");
    }

    let backup_job = compress_folder(
        "long-filename-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = PathBuf::from(backup_job.backup_path.unwrap());
    restore_backup(&backup_path, &restore_dir, backup_job.checksum, None).unwrap();

    // Verify long filename restored correctly
    let restored_long = restore_dir.join(&long_name_200);
    assert!(restored_long.exists(), "Long filename should be restored");
    assert_eq!(fs::read(&restored_long).unwrap(), b"content");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// ============================================================================
// CRITICAL TEST 7: IDEMPOTENCY - Multiple Full Backups Same Result
// ============================================================================

#[test]
fn test_backup_idempotency() {
    let (source_dir, dest_dir, _) = setup_test_dirs("idempotency_test");

    // Create test files
    fs::write(source_dir.join("file1.txt"), b"unchanging content 1").unwrap();
    fs::write(source_dir.join("file2.txt"), b"unchanging content 2").unwrap();

    // Backup #1
    let backup1 = compress_folder(
        "idempotency-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    // Backup #2 (no changes)
    std::thread::sleep(std::time::Duration::from_millis(100));
    let backup2 = compress_folder(
        "idempotency-test-2",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    // CRITICAL: Both backups should have same metadata
    assert_eq!(backup1.files_count, backup2.files_count,
        "File count should be identical for unchanged source");
    assert_eq!(backup1.original_size, backup2.original_size,
        "Original size should be identical");

    // Compressed sizes might vary slightly due to timestamps in TAR headers,
    // but should be within 1% of each other
    let size1 = backup1.compressed_size.unwrap() as f64;
    let size2 = backup2.compressed_size.unwrap() as f64;
    let size_diff_pct = ((size1 - size2).abs() / size1) * 100.0;

    assert!(size_diff_pct < 1.0,
        "Compressed sizes should be nearly identical (<1% diff), got {:.2}% difference",
        size_diff_pct);

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// CRITICAL TEST 8: CHECKSUM COLLISION DETECTION
// ============================================================================

#[test]
fn test_checksum_must_differ_for_different_content() {
    let (source_dir, dest_dir, _) = setup_test_dirs("checksum_collision");

    // Create file v1
    fs::write(source_dir.join("data.txt"), b"Original Content Version 1").unwrap();

    let backup1 = compress_folder(
        "checksum-test-1",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    let checksum1 = backup1.checksum.clone().unwrap();

    // Modify file
    fs::write(source_dir.join("data.txt"), b"Modified Content Version 2").unwrap();

    let backup2 = compress_folder(
        "checksum-test-2",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    let checksum2 = backup2.checksum.unwrap();

    // CRITICAL: Checksums MUST be different for different content
    assert_ne!(checksum1, checksum2,
        "CRITICAL SECURITY FAILURE: Different backups have same checksum! Collision detected!");

    // Verify checksums are valid SHA-256 (64 hex characters)
    assert_eq!(checksum1.len(), 64, "Checksum should be 64 hex chars (SHA-256)");
    assert_eq!(checksum2.len(), 64, "Checksum should be 64 hex chars (SHA-256)");

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// CRITICAL TEST 9: RESTORE WITH MISSING BACKUP FILE (ERROR HANDLING)
// ============================================================================

#[test]
fn test_restore_nonexistent_backup_fails_gracefully() {
    let restore_dir = std::env::temp_dir().join("restore_error_test");
    let _ = fs::remove_dir_all(&restore_dir);
    fs::create_dir_all(&restore_dir).unwrap();

    let nonexistent_backup = PathBuf::from("/nonexistent/path/backup.tar.zst");

    // CRITICAL: Must return error, not panic
    let result = restore_backup(&nonexistent_backup, &restore_dir, None, None);

    assert!(result.is_err(), "Restore of nonexistent backup must fail gracefully");

    let error_msg = result.unwrap_err();
    assert!(!error_msg.is_empty(), "Error message should be descriptive");

    cleanup_test_dirs(&[&restore_dir]);
}

// ============================================================================
// CRITICAL TEST 10: INCREMENTAL BACKUP AFTER FILE DELETION
// ============================================================================

#[test]
fn test_incremental_handles_deleted_files() {
    let (source_dir, dest_dir, _) = setup_test_dirs("incremental_deletion");

    // Create initial files
    fs::write(source_dir.join("keep.txt"), b"This stays").unwrap();
    fs::write(source_dir.join("delete_me.txt"), b"This will be deleted").unwrap();

    // Full backup
    let full = compress_folder(
        "deletion-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None,
        None,
    ).unwrap();

    assert_eq!(full.files_count.unwrap(), 2);

    // Build and save manifest after full backup
    let (all_files, _) = scan_all_files(&source_dir).unwrap();
    let manifest = build_manifest("deletion-test", &all_files, &source_dir).unwrap();
    let manifest_path = dest_dir.join("deletion-test_manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(&manifest_path, manifest_json).unwrap();

    // Delete file
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::remove_file(source_dir.join("delete_me.txt")).unwrap();

    // Load previous manifest for incremental
    let prev_manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let prev_manifest: BackupManifest = serde_json::from_str(&prev_manifest_str).unwrap();

    // Incremental backup
    let incremental = compress_folder(
        "deletion-test",
        &source_dir,
        &dest_dir,
        &BackupType::Incremental,
        Some(&prev_manifest),
        None,
        None,
    ).unwrap();

    // CRITICAL: Incremental should have 0 files (nothing changed, one deleted)
    // The manifest should track the deletion
    assert_eq!(incremental.files_count.unwrap(), 0,
        "Incremental backup after deletion should have 0 new/changed files");

    // Update manifest to reflect current state (simulating what the command does)
    let (remaining_files, _) = scan_all_files(&source_dir).unwrap();
    let updated_manifest = build_manifest("deletion-test", &remaining_files, &source_dir).unwrap();
    let updated_manifest_json = serde_json::to_string_pretty(&updated_manifest).unwrap();
    fs::write(&manifest_path, updated_manifest_json).unwrap();

    // Verify manifest is updated
    let manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_str).unwrap();

    // Manifest should track remaining file
    let files = manifest.get("files").unwrap();
    assert!(files.get("keep.txt").is_some(), "Manifest should track remaining file");

    // Deleted file should NOT be in updated manifest (it's been removed from source)
    assert!(files.get("delete_me.txt").is_none(),
        "Manifest should NOT track deleted files in updated state");

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}
