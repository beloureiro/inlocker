use inlocker_lib::backup::{compress_folder, restore_backup};
use inlocker_lib::types::BackupType;
use std::fs;

/// Integration test: Complete backup and restore cycle using REAL functions
#[test]
fn test_backup_restore_cycle() {
    // Setup: Create test directories
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join("integration_test_source");
    let dest_dir = temp_dir.join("integration_test_dest");
    let restore_dir = temp_dir.join("integration_test_restore");

    // Cleanup from previous runs
    let _ = fs::remove_dir_all(&source_dir);
    let _ = fs::remove_dir_all(&dest_dir);
    let _ = fs::remove_dir_all(&restore_dir);

    // Create test directories
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();
    fs::create_dir_all(&restore_dir).unwrap();

    // Create test files with known content
    let test_content_1 = b"Hello World from Integration Test";
    let test_content_2 = b"Test Data 12345";
    let test_content_3 = b"Nested File Content XYZ";

    fs::write(source_dir.join("file1.txt"), test_content_1).unwrap();
    fs::write(source_dir.join("file2.txt"), test_content_2).unwrap();

    let subdir = source_dir.join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("file3.txt"), test_content_3).unwrap();

    // PHASE 1: Backup using REAL compress_folder function
    let backup_result = compress_folder(
        "test-config-id",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        None,
        None, // No app handle for testing
        None, // No encryption
    );

    assert!(backup_result.is_ok(), "Backup should succeed: {:?}", backup_result.err());

    let backup_job = backup_result.unwrap();
    let backup_path = std::path::PathBuf::from(backup_job.backup_path.expect("Backup should have path"));

    // Verify backup file exists and has content
    assert!(backup_path.exists(), "Backup file should exist at: {:?}", backup_path);

    let backup_metadata = fs::metadata(&backup_path).unwrap();
    assert!(backup_metadata.len() > 0, "Backup file should not be empty");
    assert!(backup_metadata.len() < 10_000, "Backup should be compressed (small test files)");

    // PHASE 2: Restore using REAL restore_backup function
    let restore_result = restore_backup(
        &backup_path,
        &restore_dir,
        None, // No checksum validation in this test
        None, // No password
    );

    assert!(restore_result.is_ok(), "Restore should succeed: {:?}", restore_result.err());

    let restore_info = restore_result.unwrap();
    assert_eq!(restore_info.files_count, 3, "Should restore 3 files");
    assert!(restore_info.success, "Restore should report success");

    // PHASE 3: Verify restored files EXACTLY match originals
    let restored_file1 = restore_dir.join("file1.txt");
    let restored_file2 = restore_dir.join("file2.txt");
    let restored_file3 = restore_dir.join("subdir/file3.txt");

    assert!(restored_file1.exists(), "file1.txt should be restored");
    assert!(restored_file2.exists(), "file2.txt should be restored");
    assert!(restored_file3.exists(), "subdir/file3.txt should be restored");

    // Verify exact content matches
    let content1 = fs::read(&restored_file1).unwrap();
    assert_eq!(&content1[..], test_content_1, "file1.txt content should match exactly");

    let content2 = fs::read(&restored_file2).unwrap();
    assert_eq!(&content2[..], test_content_2, "file2.txt content should match exactly");

    let content3 = fs::read(&restored_file3).unwrap();
    assert_eq!(&content3[..], test_content_3, "file3.txt content should match exactly");

    // PHASE 4: Test with checksum validation
    // Calculate checksum of backup
    let backup_data = fs::read(&backup_path).unwrap();
    let checksum = calculate_sha256(&backup_data);

    // Remove restored files to test again with checksum
    let _ = fs::remove_dir_all(&restore_dir);
    fs::create_dir_all(&restore_dir).unwrap();

    // Restore with checksum validation
    let restore_with_checksum = restore_backup(
        &backup_path,
        &restore_dir,
        Some(checksum.clone()),
        None, // No password
    );

    assert!(restore_with_checksum.is_ok(), "Restore with valid checksum should succeed");

    // Test with WRONG checksum (should fail)
    let _ = fs::remove_dir_all(&restore_dir);
    fs::create_dir_all(&restore_dir).unwrap();

    let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    let restore_with_wrong_checksum = restore_backup(
        &backup_path,
        &restore_dir,
        Some(wrong_checksum),
        None, // No password
    );

    assert!(restore_with_wrong_checksum.is_err(), "Restore with wrong checksum should FAIL");
    let error_msg = restore_with_wrong_checksum.unwrap_err();
    assert!(error_msg.contains("integrity"), "Error should mention integrity check");

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
