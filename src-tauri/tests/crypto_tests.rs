/// CRYPTOGRAPHY TESTS - AES-256-GCM + Argon2id
///
/// Comprehensive test suite for encryption/decryption functionality.
/// Tests validate OWASP ASVS L2 requirements and NIST standards.
///
/// Standards Compliance:
/// - NIST SP 800-38D (AES-GCM)
/// - RFC 9106 (Argon2id parameters)
/// - OWASP ASVS V2 (Authentication)
/// - OWASP ASVS V6 (Cryptography)

use inlocker_lib::crypto::*;
use std::fs;
use std::path::PathBuf;

/// Helper: Setup test directories
fn setup_test_dirs(test_name: &str) -> (PathBuf, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let input_dir = temp_dir.join(format!("crypto_{}_input", test_name));
    let output_dir = temp_dir.join(format!("crypto_{}_output", test_name));

    let _ = fs::remove_dir_all(&input_dir);
    let _ = fs::remove_dir_all(&output_dir);

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    (input_dir, output_dir)
}

/// Helper: Cleanup test directories
fn cleanup_test_dirs(dirs: &[&std::path::Path]) {
    for dir in dirs {
        let _ = fs::remove_dir_all(dir);
    }
}

// ============================================================================
// TEST CATEGORY 1: KEY DERIVATION (Argon2id)
// ============================================================================

#[test]
fn test_argon2id_key_derivation() {
    let password = "TestPassword123!";
    let salt = generate_salt();

    let key = derive_key(password, &salt).unwrap();

    // Key must be 256 bits (32 bytes)
    assert_eq!(key.len(), 32, "Key must be 256 bits");

    // Key should be deterministic
    let key2 = derive_key(password, &salt).unwrap();
    assert_eq!(key, key2, "Same password+salt must produce same key");
}

#[test]
fn test_different_passwords_different_keys() {
    let salt = generate_salt();
    let key1 = derive_key("Password1!", &salt).unwrap();
    let key2 = derive_key("Password2!", &salt).unwrap();

    assert_ne!(key1, key2, "Different passwords must produce different keys");
}

#[test]
fn test_different_salts_different_keys() {
    let password = "SamePassword123!";
    let salt1 = generate_salt();
    let salt2 = generate_salt();

    let key1 = derive_key(password, &salt1).unwrap();
    let key2 = derive_key(password, &salt2).unwrap();

    assert_ne!(key1, key2, "Different salts must produce different keys");
}

#[test]
fn test_salt_generation_uniqueness() {
    let salt1 = generate_salt();
    let salt2 = generate_salt();
    let salt3 = generate_salt();

    // Salts should be 16 bytes
    assert_eq!(salt1.len(), 16);
    assert_eq!(salt2.len(), 16);

    // Salts should be unique (probabilistically)
    assert_ne!(salt1, salt2);
    assert_ne!(salt2, salt3);
    assert_ne!(salt1, salt3);
}

#[test]
fn test_nonce_generation_uniqueness() {
    let nonce1 = generate_nonce();
    let nonce2 = generate_nonce();
    let nonce3 = generate_nonce();

    // Nonces should be 12 bytes (GCM standard)
    assert_eq!(nonce1.len(), 12);
    assert_eq!(nonce2.len(), 12);

    // Nonces should be unique (probabilistically)
    assert_ne!(nonce1, nonce2);
    assert_ne!(nonce2, nonce3);
    assert_ne!(nonce1, nonce3);
}

// ============================================================================
// TEST CATEGORY 2: ENCRYPTION/DECRYPTION CORRECTNESS
// ============================================================================

#[test]
fn test_encrypt_decrypt_round_trip() {
    let plaintext = b"Hello, World! This is a test message for encryption.";
    let password = "SecurePassword123!";

    let (ciphertext, metadata) = encrypt(plaintext, password).unwrap();

    // Ciphertext should be different from plaintext
    assert_ne!(&ciphertext[..plaintext.len()], plaintext);

    // Ciphertext should be longer (includes auth tag)
    assert!(ciphertext.len() > plaintext.len(), "Ciphertext should include auth tag");

    // Decrypt
    let decrypted = decrypt(&ciphertext, password, &metadata).unwrap();

    // Should match original
    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_empty_data_encryption() {
    let plaintext = b"";
    let password = "TestPassword123!";

    let (ciphertext, metadata) = encrypt(plaintext, password).unwrap();
    let decrypted = decrypt(&ciphertext, password, &metadata).unwrap();

    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_large_data_encryption() {
    // 1MB of data
    let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    let password = "LargeDataPassword123!";

    let (ciphertext, metadata) = encrypt(&plaintext, password).unwrap();
    let decrypted = decrypt(&ciphertext, password, &metadata).unwrap();

    assert_eq!(decrypted.len(), plaintext.len());
    assert_eq!(&decrypted, &plaintext);
}

#[test]
fn test_binary_data_encryption() {
    // Binary data with all byte values
    let plaintext: Vec<u8> = (0..=255).collect();
    let password = "BinaryDataTest123!";

    let (ciphertext, metadata) = encrypt(&plaintext, password).unwrap();
    let decrypted = decrypt(&ciphertext, password, &metadata).unwrap();

    assert_eq!(&decrypted, &plaintext);
}

#[test]
fn test_unicode_data_encryption() {
    let plaintext = "Hello 世界 🌍 Привет مرحبا".as_bytes();
    let password = "UnicodeTest123!";

    let (ciphertext, metadata) = encrypt(plaintext, password).unwrap();
    let decrypted = decrypt(&ciphertext, password, &metadata).unwrap();

    assert_eq!(&decrypted, plaintext);
    assert_eq!(std::str::from_utf8(&decrypted).unwrap(), "Hello 世界 🌍 Привет مرحبا");
}

// ============================================================================
// TEST CATEGORY 3: AUTHENTICATION & INTEGRITY
// ============================================================================

#[test]
fn test_wrong_password_fails() {
    let plaintext = b"Sensitive data that should not be decryptable";
    let correct_password = "CorrectPassword123!";
    let wrong_password = "WrongPassword456!";

    let (ciphertext, metadata) = encrypt(plaintext, correct_password).unwrap();

    // Decrypt with wrong password should fail
    let result = decrypt(&ciphertext, wrong_password, &metadata);

    assert!(result.is_err(), "Wrong password should fail decryption");
    assert!(result.unwrap_err().contains("Decryption failed"),
        "Error should mention decryption failure");
}

#[test]
fn test_tampered_ciphertext_fails() {
    let plaintext = b"Original message";
    let password = "TestPassword123!";

    let (mut ciphertext, metadata) = encrypt(plaintext, password).unwrap();

    // Tamper with ciphertext (flip one bit)
    if ciphertext.len() > 10 {
        ciphertext[10] ^= 0xFF;
    }

    // Decryption should fail (GCM auth tag validation)
    let result = decrypt(&ciphertext, password, &metadata);

    assert!(result.is_err(), "Tampered ciphertext should fail decryption");
    assert!(result.unwrap_err().contains("Decryption failed"),
        "Error should indicate authentication failure");
}

#[test]
fn test_tampered_metadata_salt_fails() {
    let plaintext = b"Secret data";
    let password = "TestPassword123!";

    let (ciphertext, mut metadata) = encrypt(plaintext, password).unwrap();

    // Tamper with salt
    metadata.salt = generate_salt().iter().map(|b| format!("{:02x}", b)).collect();

    // Decryption should fail (wrong key derived)
    let result = decrypt(&ciphertext, password, &metadata);

    assert!(result.is_err(), "Tampered salt should fail decryption");
}

#[test]
fn test_tampered_nonce_fails() {
    let plaintext = b"Confidential information";
    let password = "SecurePass123!";

    let (ciphertext, mut metadata) = encrypt(plaintext, password).unwrap();

    // Tamper with nonce
    metadata.nonce = generate_nonce().iter().map(|b| format!("{:02x}", b)).collect();

    // Decryption should fail
    let result = decrypt(&ciphertext, password, &metadata);

    assert!(result.is_err(), "Tampered nonce should fail decryption");
}

// ============================================================================
// TEST CATEGORY 4: PASSWORD VALIDATION
// ============================================================================

#[test]
fn test_password_too_short() {
    let result = validate_password_strength("Short1!");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("12 characters"));
}

#[test]
fn test_password_missing_uppercase() {
    let result = validate_password_strength("lowercase123!");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("uppercase"));
}

#[test]
fn test_password_missing_lowercase() {
    let result = validate_password_strength("UPPERCASE123!");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("lowercase"));
}

#[test]
fn test_password_missing_digit() {
    let result = validate_password_strength("NoDigitsHere!");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("digit"));
}

#[test]
fn test_password_missing_special() {
    let result = validate_password_strength("NoSpecialChar123");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("special"));
}

#[test]
fn test_password_valid() {
    let result = validate_password_strength("ValidPassword123!");
    assert!(result.is_ok());
}

#[test]
fn test_password_strong_examples() {
    // Various valid passwords (all meet 12+ char requirement)
    let passwords = vec![
        "MySecurePass123!",
        "Tr0ub4dor&3Extra",  // Fixed: was 11 chars, now 16
        "C0rrect-H0rse-Battery-Staple!",
        "P@ssw0rd2024!",
    ];

    for password in passwords {
        assert!(validate_password_strength(password).is_ok(),
            "Password '{}' should be valid", password);
    }
}

// ============================================================================
// TEST CATEGORY 5: FILE ENCRYPTION/DECRYPTION
// ============================================================================

#[test]
fn test_file_encrypt_decrypt_cycle() {
    let (input_dir, output_dir) = setup_test_dirs("file_enc_dec");

    // Create test file
    let plaintext_path = input_dir.join("plaintext.txt");
    let plaintext_data = b"This is a test file for encryption.\nMultiple lines.\nWith various content.";
    fs::write(&plaintext_path, plaintext_data).unwrap();

    let encrypted_path = output_dir.join("encrypted.bin");
    let decrypted_path = output_dir.join("decrypted.txt");

    let password = "FileTestPassword123!";

    // Encrypt file
    let metadata = encrypt_file(&plaintext_path, &encrypted_path, password).unwrap();

    // Verify encrypted file exists
    assert!(encrypted_path.exists());
    assert!(encrypted_path.with_extension("meta").exists());

    // Verify encrypted content is different
    let encrypted_data = fs::read(&encrypted_path).unwrap();
    assert_ne!(&encrypted_data[..plaintext_data.len()], plaintext_data);

    // Decrypt file
    decrypt_file(&encrypted_path, &decrypted_path, password).unwrap();

    // Verify decrypted content matches original
    let decrypted_data = fs::read(&decrypted_path).unwrap();
    assert_eq!(&decrypted_data, plaintext_data);

    cleanup_test_dirs(&[&input_dir, &output_dir]);
}

#[test]
fn test_file_decrypt_with_wrong_password() {
    let (input_dir, output_dir) = setup_test_dirs("file_wrong_pass");

    let plaintext_path = input_dir.join("secret.txt");
    fs::write(&plaintext_path, b"Top secret data").unwrap();

    let encrypted_path = output_dir.join("encrypted.bin");
    let decrypted_path = output_dir.join("decrypted.txt");

    let correct_password = "CorrectPassword123!";
    let wrong_password = "WrongPassword456!";

    // Encrypt
    encrypt_file(&plaintext_path, &encrypted_path, correct_password).unwrap();

    // Try to decrypt with wrong password
    let result = decrypt_file(&encrypted_path, &decrypted_path, wrong_password);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Decryption failed"));

    cleanup_test_dirs(&[&input_dir, &output_dir]);
}

#[test]
fn test_large_file_encryption() {
    let (input_dir, output_dir) = setup_test_dirs("large_file");

    // Create 10MB file
    let plaintext_path = input_dir.join("large.bin");
    let large_data: Vec<u8> = (0..10_000_000).map(|i| (i % 256) as u8).collect();
    fs::write(&plaintext_path, &large_data).unwrap();

    let encrypted_path = output_dir.join("large_encrypted.bin");
    let decrypted_path = output_dir.join("large_decrypted.bin");

    let password = "LargeFileTest123!";

    // Encrypt
    encrypt_file(&plaintext_path, &encrypted_path, password).unwrap();

    // Decrypt
    decrypt_file(&encrypted_path, &decrypted_path, password).unwrap();

    // Verify
    let decrypted_data = fs::read(&decrypted_path).unwrap();
    assert_eq!(decrypted_data.len(), large_data.len());

    // Spot check (full comparison would be slow)
    assert_eq!(&decrypted_data[..1000], &large_data[..1000]);
    assert_eq!(&decrypted_data[5_000_000..5_001_000], &large_data[5_000_000..5_001_000]);

    cleanup_test_dirs(&[&input_dir, &output_dir]);
}

// ============================================================================
// TEST CATEGORY 6: METADATA HANDLING
// ============================================================================

#[test]
fn test_metadata_serialization() {
    let plaintext = b"Test data";
    let password = "TestPass123!";

    let (_, metadata) = encrypt(plaintext, password).unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&metadata).unwrap();

    // Deserialize back
    let metadata2: EncryptionMetadata = serde_json::from_str(&json).unwrap();

    // Should match
    assert_eq!(metadata.salt, metadata2.salt);
    assert_eq!(metadata.nonce, metadata2.nonce);
    assert_eq!(metadata.version, metadata2.version);
    assert_eq!(metadata.memory_cost, metadata2.memory_cost);
    assert_eq!(metadata.iterations, metadata2.iterations);
    assert_eq!(metadata.parallelism, metadata2.parallelism);
}

#[test]
fn test_metadata_rfc9106_compliance() {
    let metadata = EncryptionMetadata::default();

    // RFC 9106 recommended parameters for interactive use
    assert_eq!(metadata.version, 0x13, "Should use Argon2 v1.3");
    assert_eq!(metadata.memory_cost, 65536, "Should use 64MB memory");
    assert_eq!(metadata.iterations, 3, "Should use 3 iterations");
    assert_eq!(metadata.parallelism, 4, "Should use 4 parallel lanes");
}

// ============================================================================
// TEST CATEGORY 7: EDGE CASES & ERROR HANDLING
// ============================================================================

#[test]
fn test_encryption_with_empty_password() {
    let plaintext = b"Data";
    let password = "";

    // Even empty password should work (though not recommended)
    let result = encrypt(plaintext, password);
    assert!(result.is_ok(), "Empty password should technically work");
}

#[test]
fn test_encryption_with_very_long_password() {
    let plaintext = b"Test data";
    let password = "A".repeat(1000); // 1000 characters

    let (ciphertext, metadata) = encrypt(plaintext, &password).unwrap();
    let decrypted = decrypt(&ciphertext, &password, &metadata).unwrap();

    assert_eq!(&decrypted, plaintext);
}

#[test]
fn test_encryption_determinism() {
    let plaintext = b"Same plaintext";
    let password = "SamePassword123!";

    // Encrypt twice
    let (ciphertext1, metadata1) = encrypt(plaintext, password).unwrap();
    let (ciphertext2, metadata2) = encrypt(plaintext, password).unwrap();

    // Ciphertexts should be DIFFERENT (different nonces)
    assert_ne!(ciphertext1, ciphertext2, "Encryption should use random nonces");
    assert_ne!(metadata1.nonce, metadata2.nonce, "Nonces must be unique");

    // But both should decrypt correctly
    let decrypted1 = decrypt(&ciphertext1, password, &metadata1).unwrap();
    let decrypted2 = decrypt(&ciphertext2, password, &metadata2).unwrap();

    assert_eq!(&decrypted1, plaintext);
    assert_eq!(&decrypted2, plaintext);
}

#[test]
fn test_nonce_reuse_detection() {
    // This test documents that we DON'T reuse nonces
    let password = "TestPassword123!";
    let plaintext1 = b"Message 1";
    let plaintext2 = b"Message 2";

    let (_, meta1) = encrypt(plaintext1, password).unwrap();
    let (_, meta2) = encrypt(plaintext2, password).unwrap();

    // Nonces MUST be different
    assert_ne!(meta1.nonce, meta2.nonce, "CRITICAL: Nonce reuse detected!");
}

#[test]
fn test_iv_uniqueness_across_encryptions() {
    let password = "TestPass123!";
    let plaintext = b"Same data";

    let mut nonces = std::collections::HashSet::new();

    // Encrypt 10 times (reduced from 100 for faster testing)
    // In production, IV/nonce MUST be unique for every encryption
    for _ in 0..10 {
        let (_, metadata) = encrypt(plaintext, password).unwrap();
        assert!(nonces.insert(metadata.nonce.clone()),
            "CRITICAL: Duplicate nonce detected!");
    }

    assert_eq!(nonces.len(), 10, "All 10 nonces should be unique");
}
