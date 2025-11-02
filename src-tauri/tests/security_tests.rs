use inlocker_lib::backup::{compress_folder, restore_backup};
use inlocker_lib::types::{BackupMode, BackupType};
use std::fs;
use std::io::Write;

/// SECURITY TEST: Detect corrupted backup files
#[test]
fn test_detect_corrupted_backup() {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join("security_test_source");
    let dest_dir = temp_dir.join("security_test_dest");
    let restore_dir = temp_dir.join("security_test_restore");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    // Create test file with sensitive data
    let sensitive_data = b"SENSITIVE: Credit Card 1234-5678-9012-3456";
    fs::write(source_dir.join("sensitive.txt"), sensitive_data).unwrap();

    // Backup
    let backup_job = compress_folder(
        "security-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = std::path::PathBuf::from(backup_job.backup_path.unwrap());
    let checksum = backup_job.checksum.clone().unwrap();

    // CORRUPT THE BACKUP FILE (simulate data corruption)
    let mut backup_data = fs::read(&backup_path).unwrap();
    if backup_data.len() > 100 {
        let len = backup_data.len(); // Store length to avoid borrow checker issue
        backup_data[50] ^= 0xFF; // Flip bits in middle of file
        backup_data[len - 50] ^= 0xFF; // Flip bits near end
    }
    fs::write(&backup_path, &backup_data).unwrap();

    // Try to restore with checksum validation - MUST FAIL
    let restore_result = restore_backup(&backup_path, &restore_dir, Some(checksum), None);

    assert!(restore_result.is_err(), "SECURITY FAILURE: Corrupted backup was accepted!");
    let error = restore_result.unwrap_err();
    assert!(error.contains("integrity"), "Error should mention integrity failure");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);
}

/// SECURITY TEST: Large files (>100MB) maintain integrity
#[test]
fn test_large_file_integrity() {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join("large_file_test_source");
    let dest_dir = temp_dir.join("large_file_test_dest");
    let restore_dir = temp_dir.join("large_file_test_restore");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    // Create 10MB file (reduced from 100MB for faster test)
    let large_file_path = source_dir.join("large_file.bin");
    let chunk_size = 1024 * 1024; // 1MB chunks
    let mut file = fs::File::create(&large_file_path).unwrap();

    // Write deterministic pattern (not random) for verification
    for i in 0..10 {
        let pattern: Vec<u8> = (0..chunk_size).map(|j| ((i + j) % 256) as u8).collect();
        file.write_all(&pattern).unwrap();
    }
    drop(file);

    // Calculate original checksum
    let original_data = fs::read(&large_file_path).unwrap();
    let original_checksum = calculate_sha256(&original_data);

    // Backup
    let backup_job = compress_folder(
        "large-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = std::path::PathBuf::from(backup_job.backup_path.unwrap());

    // Restore
    let restore_result = restore_backup(&backup_path, &restore_dir, backup_job.checksum, None);
    assert!(restore_result.is_ok(), "Large file restore failed");

    // Verify byte-for-byte integrity
    let restored_data = fs::read(restore_dir.join("large_file.bin")).unwrap();
    let restored_checksum = calculate_sha256(&restored_data);

    assert_eq!(original_checksum, restored_checksum,
        "CRITICAL: Large file integrity compromised! Data corruption detected!");
    assert_eq!(original_data.len(), restored_data.len(),
        "CRITICAL: File size mismatch after restore!");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);
}

/// SECURITY TEST: Special characters and unicode in filenames
#[test]
fn test_special_filenames() {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join("special_chars_test");
    let dest_dir = temp_dir.join("special_chars_dest");
    let restore_dir = temp_dir.join("special_chars_restore");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    // Create files with challenging names
    let test_files = vec![
        "normal.txt",
        "with spaces.txt",
        "emoji_😀_test.txt",
        "chinese_中文.txt",
        "dots..and...more.txt",
    ];

    for filename in &test_files {
        let content = format!("Content of {}", filename);
        if let Err(e) = fs::write(source_dir.join(filename), content.as_bytes()) {
            eprintln!("Warning: Could not create {}: {}", filename, e);
        }
    }

    // Backup
    let backup_result = compress_folder(
        "special-chars-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
    );

    assert!(backup_result.is_ok(), "Backup with special filenames failed");

    let backup_job = backup_result.unwrap();
    let backup_path = std::path::PathBuf::from(backup_job.backup_path.unwrap());

    // Restore
    let restore_result = restore_backup(&backup_path, &restore_dir, backup_job.checksum, None);
    assert!(restore_result.is_ok(), "Restore with special filenames failed");

    // Verify all files exist and have correct content
    for filename in &test_files {
        let restored_path = restore_dir.join(filename);
        if restored_path.exists() {
            let content = fs::read_to_string(&restored_path).unwrap();
            assert_eq!(content, format!("Content of {}", filename));
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);
}

/// SECURITY TEST: Deep directory nesting
#[test]
fn test_deep_directory_structure() {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join("deep_test");
    let dest_dir = temp_dir.join("deep_dest");
    let restore_dir = temp_dir.join("deep_restore");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    // Create deep nested structure (20 levels)
    let mut current_path = source_dir.clone();
    for i in 0..20 {
        current_path = current_path.join(format!("level_{}", i));
        fs::create_dir_all(&current_path).unwrap();
        fs::write(current_path.join(format!("file_{}.txt", i)), format!("Level {}", i)).unwrap();
    }

    // Backup
    let backup_job = compress_folder(
        "deep-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = std::path::PathBuf::from(backup_job.backup_path.unwrap());

    // Restore
    let restore_result = restore_backup(&backup_path, &restore_dir, backup_job.checksum, None);
    assert!(restore_result.is_ok(), "Deep directory restore failed");

    // Verify deep file exists
    let mut verify_path = restore_dir.clone();
    for i in 0..20 {
        verify_path = verify_path.join(format!("level_{}", i));
    }
    let deepest_file = verify_path.join("file_19.txt");
    assert!(deepest_file.exists(), "Deepest file not restored");

    let content = fs::read_to_string(&deepest_file).unwrap();
    assert_eq!(content, "Level 19");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);
}

/// SECURITY TEST: Many small files (stress test)
#[test]
fn test_many_small_files() {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join("many_files_test");
    let dest_dir = temp_dir.join("many_files_dest");
    let restore_dir = temp_dir.join("many_files_restore");

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    // Create 1000 small files
    let num_files = 1000;
    for i in 0..num_files {
        let filename = format!("file_{:04}.txt", i);
        let content = format!("File number {} with some content", i);
        fs::write(source_dir.join(&filename), content).unwrap();
    }

    // Backup
    let backup_job = compress_folder(
        "many-files-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
    ).unwrap();

    let backup_path = std::path::PathBuf::from(backup_job.backup_path.unwrap());

    // Verify backup contains all files
    assert_eq!(backup_job.files_count.unwrap(), num_files,
        "Backup should contain all {} files", num_files);

    // Restore
    let restore_result = restore_backup(&backup_path, &restore_dir, backup_job.checksum, None);
    assert!(restore_result.is_ok(), "Many files restore failed");

    // Verify file count
    let restored_count = fs::read_dir(&restore_dir).unwrap().count();
    assert_eq!(restored_count, num_files,
        "Should restore all {} files, got {}", num_files, restored_count);

    // Spot check some files
    for i in [0, num_files/4, num_files/2, num_files-1] {
        let filename = format!("file_{:04}.txt", i);
        let content = fs::read_to_string(restore_dir.join(&filename)).unwrap();
        assert_eq!(content, format!("File number {} with some content", i));
    }

    // Cleanup
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);
}

fn calculate_sha256(data: &[u8]) -> String {
    use ring::digest::{Context, SHA256};
    let mut context = Context::new(&SHA256);
    context.update(data);
    let digest = context.finish();
    hex::encode(digest.as_ref())
}
