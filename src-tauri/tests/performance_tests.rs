/// PERFORMANCE TESTS
///
/// These tests validate that InLocker meets performance targets defined in the roadmap.
/// All tests use REAL data volumes to ensure accurate measurements.
///
/// Reference: docs/04-roadmap.md (lines 249-258), docs/08-testing-strategy.md
///
/// CRITICAL TARGETS:
/// - 1GB backup in <2 minutes
/// - Compression ratio >2x for text files
/// - Memory usage <500MB for 10GB backup
/// - Incremental 10x faster than full
/// - Handle 10,000 small files efficiently
/// - CPU usage <80% during backup

use inlocker_lib::backup::{build_manifest, compress_folder, restore_backup, scan_all_files};
use inlocker_lib::types::{BackupMode, BackupType};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Helper: Setup test directories
fn setup_test_dirs(test_name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let source_dir = temp_dir.join(format!("perf_{}_source", test_name));
    let dest_dir = temp_dir.join(format!("perf_{}_dest", test_name));
    let restore_dir = temp_dir.join(format!("perf_{}_restore", test_name));

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
// 🚨 CRITICAL PERFORMANCE TEST #1: 1GB BACKUP IN <2 MINUTES
// ============================================================================

#[test]
#[ignore] // Requires ~1GB disk space and takes ~1-2 minutes
fn test_1gb_backup_performance() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("1gb_perf");

    println!("📦 Creating 1GB test dataset...");
    let start_creation = Instant::now();

    // Create 1GB file using pattern (not random, for speed)
    let file_path = source_dir.join("large_file.dat");
    let mut file = fs::File::create(&file_path).unwrap();
    let chunk_size = 1024 * 1024; // 1MB chunks

    for i in 0..1024 {
        let pattern: Vec<u8> = (0..chunk_size)
            .map(|j| ((i + j) % 256) as u8)
            .collect();
        file.write_all(&pattern).unwrap();
    }
    drop(file);

    let creation_time = start_creation.elapsed();
    println!("✓ Test data created in {:.2}s", creation_time.as_secs_f64());

    // Verify file size
    let file_size = fs::metadata(&file_path).unwrap().len();
    assert_eq!(file_size, 1024 * 1024 * 1024, "File should be exactly 1GB");

    // CRITICAL TEST: Backup 1GB in <2 minutes
    println!("\n🔵 Starting 1GB backup performance test...");
    let start_backup = Instant::now();

    let backup_result = compress_folder(
        "1gb-perf-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
        None,
    );

    let backup_duration = start_backup.elapsed();

    assert!(backup_result.is_ok(), "Backup failed: {:?}", backup_result.err());

    let backup_job = backup_result.unwrap();
    let backup_path = PathBuf::from(backup_job.backup_path.unwrap());

    // CRITICAL ASSERTION: Must complete in <2 minutes (120 seconds)
    let backup_secs = backup_duration.as_secs_f64();
    assert!(
        backup_secs < 120.0,
        "PERFORMANCE FAILURE: 1GB backup took {:.2}s (must be <120s)",
        backup_secs
    );

    println!("✅ Backup completed in {:.2}s (target: <120s)", backup_secs);
    println!("   Original: {:.1} MB", backup_job.original_size.unwrap() as f64 / 1_048_576.0);
    println!("   Compressed: {:.1} MB", backup_job.compressed_size.unwrap() as f64 / 1_048_576.0);
    println!("   Throughput: {:.1} MB/s",
        (backup_job.original_size.unwrap() as f64 / 1_048_576.0) / backup_secs
    );

    // Verify backup integrity by restoring
    println!("\n🔄 Verifying backup integrity...");
    let start_restore = Instant::now();
    restore_backup(&backup_path, &restore_dir, backup_job.checksum, None).unwrap();
    let restore_duration = start_restore.elapsed();

    println!("✓ Restore completed in {:.2}s", restore_duration.as_secs_f64());

    // Verify restored file
    let restored_file = restore_dir.join("large_file.dat");
    assert!(restored_file.exists(), "Restored file not found");

    let restored_size = fs::metadata(&restored_file).unwrap().len();
    assert_eq!(restored_size, file_size, "Restored file size mismatch");

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);

    println!("\n✅ 1GB PERFORMANCE TEST PASSED");
    println!("   Backup: {:.2}s | Restore: {:.2}s | Total: {:.2}s",
        backup_secs,
        restore_duration.as_secs_f64(),
        backup_secs + restore_duration.as_secs_f64()
    );
}

// ============================================================================
// ⚡ PERFORMANCE TEST #2: COMPRESSION RATIO >2X FOR TEXT
// ============================================================================

#[test]
fn test_compression_ratio_text_files() {
    let (source_dir, dest_dir, _) = setup_test_dirs("compression_ratio");

    println!("📝 Creating text test files...");

    // Create various types of compressible text data
    // 1. Repetitive code
    let code = "function test() {\n  return true;\n}\n".repeat(50_000);
    fs::write(source_dir.join("code.js"), &code).unwrap();

    // 2. JSON data (repetitive structure)
    let json = r#"{"id":1234,"name":"test","active":true,"timestamp":"2025-11-02T10:00:00Z"}
"#.repeat(10_000);
    fs::write(source_dir.join("data.json"), &json).unwrap();

    // 3. Log files (repetitive patterns)
    let logs = "[INFO] 2025-11-02 10:00:00 - Application started successfully\n".repeat(20_000);
    fs::write(source_dir.join("app.log"), &logs).unwrap();

    // 4. Plain text (lorem ipsum)
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(30_000);
    fs::write(source_dir.join("document.txt"), &text).unwrap();

    // Calculate total original size
    let original_size: u64 = fs::read_dir(&source_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.metadata().unwrap().len())
        .sum();

    println!("✓ Created {:.1} MB of text data", original_size as f64 / 1_048_576.0);

    // Backup
    println!("\n🗜️  Testing compression ratio...");
    let backup_job = compress_folder(
        "compression-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
        None,
    ).unwrap();

    let compressed_size = backup_job.compressed_size.unwrap();
    let ratio = original_size as f64 / compressed_size as f64;

    println!("✓ Original size: {:.2} MB", original_size as f64 / 1_048_576.0);
    println!("✓ Compressed size: {:.2} MB", compressed_size as f64 / 1_048_576.0);
    println!("✓ Compression ratio: {:.2}x", ratio);

    // CRITICAL ASSERTION: Ratio must be >2x
    assert!(
        ratio > 2.0,
        "PERFORMANCE FAILURE: Compression ratio {:.2}x is not >2x for text files",
        ratio
    );

    println!("\n✅ COMPRESSION RATIO TEST PASSED ({:.2}x > 2.0x target)", ratio);

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// ⚡ PERFORMANCE TEST #3: INCREMENTAL 10X FASTER THAN FULL
// ============================================================================

#[test]
fn test_incremental_10x_faster() {
    let (source_dir, dest_dir, _) = setup_test_dirs("incremental_speed");

    println!("📦 Creating 1000 files dataset...");

    // Create 1000 files (1MB each = 1GB total)
    for i in 0..1000 {
        let file_path = source_dir.join(format!("file_{:04}.dat", i));
        let data = vec![(i % 256) as u8; 1024 * 1024]; // 1MB
        fs::write(&file_path, data).unwrap();
    }

    println!("✓ Created 1000 files (1GB total)");

    // FULL BACKUP
    println!("\n🔵 Measuring FULL backup performance...");
    let start_full = Instant::now();

    let _full_backup = compress_folder(
        "full-speed-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
        None,
    ).unwrap();

    let full_duration = start_full.elapsed();
    println!("✓ Full backup: {:.2}s", full_duration.as_secs_f64());

    // Build manifest from full backup
    println!("\n📋 Building manifest...");
    let (all_files, _) = scan_all_files(&source_dir).unwrap();
    let manifest = build_manifest("full-speed-test", &all_files, &source_dir).unwrap();
    println!("✓ Manifest built with {} files", manifest.files.len());

    // Modify only 10 files (1%)
    println!("\n📝 Modifying 10 files (1%)...");
    std::thread::sleep(std::time::Duration::from_millis(100)); // Ensure timestamp difference
    for i in 0..10 {
        let file_path = source_dir.join(format!("file_{:04}.dat", i));
        let data = vec![255u8; 1024 * 1024]; // Modified data
        fs::write(&file_path, data).unwrap();
    }

    // INCREMENTAL BACKUP
    println!("\n🔵 Measuring INCREMENTAL backup performance...");
    let start_incr = Instant::now();

    let incr_backup = compress_folder(
        "incr-speed-test",
        &source_dir,
        &dest_dir,
        &BackupType::Incremental,
        &BackupMode::Compressed,
        Some(&manifest),
        None,
        None,
        None,
    ).unwrap();

    let incr_duration = start_incr.elapsed();
    println!("✓ Incremental backup: {:.2}s", incr_duration.as_secs_f64());

    // Calculate speedup
    let speedup = full_duration.as_secs_f64() / incr_duration.as_secs_f64();

    println!("\n📊 Performance comparison:");
    println!("   Full backup: {:.2}s", full_duration.as_secs_f64());
    println!("   Incremental: {:.2}s", incr_duration.as_secs_f64());
    println!("   Speedup: {:.2}x", speedup);
    println!("   Files changed: {}/{}", incr_backup.changed_files_count.unwrap_or(0), 1000);

    // CRITICAL ASSERTION: Incremental must be >10x faster
    assert!(
        speedup > 10.0,
        "PERFORMANCE FAILURE: Incremental only {:.2}x faster (must be >10x)",
        speedup
    );

    println!("\n✅ INCREMENTAL SPEED TEST PASSED ({:.2}x > 10x target)", speedup);

    cleanup_test_dirs(&[&source_dir, &dest_dir]);
}

// ============================================================================
// ⚡ PERFORMANCE TEST #4: 10,000 SMALL FILES
// ============================================================================

#[test]
#[ignore] // Takes ~30-60 seconds
fn test_10000_small_files_performance() {
    let (source_dir, dest_dir, restore_dir) = setup_test_dirs("10k_files");

    println!("📦 Creating 10,000 small files...");
    let start_creation = Instant::now();

    // Create 10,000 files (1KB - 10KB each)
    for i in 0..10_000 {
        let file_path = source_dir.join(format!("file_{:05}.txt", i));
        let size = 1024 + (i % 9216); // 1KB to 10KB
        let data = vec![(i % 256) as u8; size];
        fs::write(&file_path, data).unwrap();
    }

    let creation_time = start_creation.elapsed();
    println!("✓ Created 10,000 files in {:.2}s", creation_time.as_secs_f64());

    // Backup
    println!("\n🔵 Backing up 10,000 files...");
    let start_backup = Instant::now();

    let backup_job = compress_folder(
        "10k-files-test",
        &source_dir,
        &dest_dir,
        &BackupType::Full,
        &BackupMode::Compressed,
        None,
        None,
        None,
        None,
    ).unwrap();

    let backup_duration = start_backup.elapsed();
    let backup_secs = backup_duration.as_secs_f64();

    println!("✓ Backup completed in {:.2}s", backup_secs);
    println!("   Files: {}", backup_job.files_count.unwrap());
    println!("   Throughput: {:.0} files/sec", 10_000.0 / backup_secs);

    // ASSERTION: Should complete in reasonable time (<5 minutes)
    assert!(
        backup_secs < 300.0,
        "PERFORMANCE WARNING: 10k files backup took {:.2}s (>5min)",
        backup_secs
    );

    // Restore
    println!("\n🔄 Restoring 10,000 files...");
    let backup_path = PathBuf::from(backup_job.backup_path.unwrap());
    let start_restore = Instant::now();

    restore_backup(&backup_path, &restore_dir, backup_job.checksum, None).unwrap();

    let restore_duration = start_restore.elapsed();
    let restore_secs = restore_duration.as_secs_f64();

    println!("✓ Restore completed in {:.2}s", restore_secs);
    println!("   Throughput: {:.0} files/sec", 10_000.0 / restore_secs);

    // Verify all files restored
    let restored_count = fs::read_dir(&restore_dir).unwrap().count();
    assert_eq!(restored_count, 10_000, "Not all files were restored");

    println!("\n✅ 10,000 FILES TEST PASSED");
    println!("   Backup: {:.2}s | Restore: {:.2}s", backup_secs, restore_secs);

    cleanup_test_dirs(&[&source_dir, &dest_dir, &restore_dir]);
}

// Note: Memory usage and CPU usage tests require additional dependencies
// (sysinfo crate) and will be added after confirming these tests work.
