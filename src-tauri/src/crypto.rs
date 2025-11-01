/// Cryptography Module - AES-256-GCM with Argon2id Key Derivation
///
/// This module provides secure encryption/decryption for backup files using:
/// - AES-256-GCM (AEAD cipher with authentication)
/// - Argon2id for key derivation (RFC 9106 parameters)
/// - Secure random IV generation
/// - Memory zeroization for keys
///
/// Security Standards:
/// - NIST SP 800-38D (GCM mode)
/// - RFC 9106 (Argon2id parameters)
/// - OWASP ASVS L2 requirements

use argon2::{Argon2, ParamsBuilder, Version};
use rand::{rngs::OsRng, RngCore};
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM};
use zeroize::Zeroize;

/// Encryption parameters (stored with encrypted data)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptionMetadata {
    /// Argon2id salt (base64 encoded)
    pub salt: String,
    /// Nonce/IV for AES-GCM (base64 encoded)
    pub nonce: String,
    /// Argon2id version
    pub version: u32,
    /// Memory cost in KB
    pub memory_cost: u32,
    /// Iteration count
    pub iterations: u32,
    /// Parallelism factor
    pub parallelism: u32,
}

impl Default for EncryptionMetadata {
    fn default() -> Self {
        // RFC 9106 recommended parameters for interactive use
        Self {
            salt: String::new(),
            nonce: String::new(),
            version: 0x13, // Argon2 version 1.3
            memory_cost: 65536, // 64 MB
            iterations: 3,
            parallelism: 4,
        }
    }
}

/// Nonce sequence for AES-GCM (single-use)
struct OneNonceSequence {
    nonce: Option<Nonce>,
}

impl OneNonceSequence {
    fn new(nonce: Nonce) -> Self {
        Self { nonce: Some(nonce) }
    }
}

impl NonceSequence for OneNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        self.nonce.take().ok_or(ring::error::Unspecified)
    }
}

/// Derive 256-bit encryption key from password using Argon2id
///
/// Uses RFC 9106 recommended parameters:
/// - Memory: 64 MB
/// - Iterations: 3
/// - Parallelism: 4
/// - Output: 32 bytes (256 bits)
pub fn derive_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, String> {
    // Configure Argon2id parameters (RFC 9106)
    let params = ParamsBuilder::new()
        .m_cost(65536) // 64 MB
        .t_cost(3)     // 3 iterations
        .p_cost(4)     // 4 parallel threads
        .output_len(32) // 256 bits
        .build()
        .map_err(|e| format!("Failed to build Argon2 params: {}", e))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        params,
    );

    // Derive key
    let mut key_bytes = vec![0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    Ok(key_bytes)
}

/// Generate cryptographically secure random salt (16 bytes)
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Generate cryptographically secure random nonce/IV (12 bytes for GCM)
pub fn generate_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Encrypt data using AES-256-GCM
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `password` - User password
///
/// # Returns
/// Tuple of (ciphertext, encryption_metadata)
pub fn encrypt(plaintext: &[u8], password: &str) -> Result<(Vec<u8>, EncryptionMetadata), String> {
    // Generate salt and nonce
    let salt = generate_salt();
    let nonce_bytes = generate_nonce();

    // Derive key from password
    let mut key_bytes = derive_key(password, &salt)?;

    // Create AES-256-GCM key
    let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| "Failed to create encryption key".to_string())?;

    // Zeroize key material after use
    key_bytes.zeroize();

    // Create nonce
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|_| "Invalid nonce".to_string())?;

    // Create sealing key with nonce sequence
    let mut sealing_key = SealingKey::new(unbound_key, OneNonceSequence::new(nonce));

    // Encrypt in-place
    let mut ciphertext = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(Aad::empty(), &mut ciphertext)
        .map_err(|_| "Encryption failed".to_string())?;

    // Create metadata
    let metadata = EncryptionMetadata {
        salt: base64::encode(&salt),
        nonce: base64::encode(&nonce_bytes),
        version: 0x13,
        memory_cost: 65536,
        iterations: 3,
        parallelism: 4,
    };

    Ok((ciphertext, metadata))
}

/// Decrypt data using AES-256-GCM
///
/// # Arguments
/// * `ciphertext` - Encrypted data (includes authentication tag)
/// * `password` - User password
/// * `metadata` - Encryption metadata (salt, nonce, params)
///
/// # Returns
/// Decrypted plaintext or error
pub fn decrypt(
    ciphertext: &[u8],
    password: &str,
    metadata: &EncryptionMetadata,
) -> Result<Vec<u8>, String> {
    // Decode salt and nonce
    let salt = base64::decode(&metadata.salt)
        .map_err(|e| format!("Invalid salt: {}", e))?;
    let nonce_bytes = base64::decode(&metadata.nonce)
        .map_err(|e| format!("Invalid nonce: {}", e))?;

    // Derive key from password
    let mut key_bytes = derive_key(password, &salt)?;

    // Create AES-256-GCM key
    let unbound_key = UnboundKey::new(&AES_256_GCM, &key_bytes)
        .map_err(|_| "Failed to create decryption key".to_string())?;

    // Zeroize key material after use
    key_bytes.zeroize();

    // Create nonce
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|_| "Invalid nonce".to_string())?;

    // Create opening key with nonce sequence
    let mut opening_key = OpeningKey::new(unbound_key, OneNonceSequence::new(nonce));

    // Decrypt in-place
    let mut plaintext = ciphertext.to_vec();
    let plaintext_len = opening_key
        .open_in_place(Aad::empty(), &mut plaintext)
        .map_err(|_| {
            "Decryption failed - wrong password or corrupted data".to_string()
        })?
        .len();

    plaintext.truncate(plaintext_len);

    Ok(plaintext)
}

/// Encrypt a file
///
/// Reads plaintext from `input_path`, encrypts it, and writes:
/// 1. Metadata JSON (salt, nonce, params) to `output_path.meta`
/// 2. Encrypted data to `output_path`
pub fn encrypt_file(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    password: &str,
) -> Result<EncryptionMetadata, String> {
    // Read plaintext
    let plaintext = std::fs::read(input_path)
        .map_err(|e| format!("Failed to read input file: {}", e))?;

    // Encrypt
    let (ciphertext, metadata) = encrypt(&plaintext, password)?;

    // Write metadata
    let metadata_path = output_path.with_extension("meta");
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    std::fs::write(&metadata_path, metadata_json)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    // Write ciphertext
    std::fs::write(output_path, ciphertext)
        .map_err(|e| format!("Failed to write encrypted file: {}", e))?;

    log::info!("✅ File encrypted: {} → {}", input_path.display(), output_path.display());
    log::info!("   Metadata: {}", metadata_path.display());

    Ok(metadata)
}

/// Decrypt a file
///
/// Reads metadata from `input_path.meta` and encrypted data from `input_path`,
/// decrypts, and writes plaintext to `output_path`.
pub fn decrypt_file(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    password: &str,
) -> Result<(), String> {
    // Read metadata
    let metadata_path = input_path.with_extension("meta");
    let metadata_json = std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    let metadata: EncryptionMetadata = serde_json::from_str(&metadata_json)
        .map_err(|e| format!("Failed to parse metadata: {}", e))?;

    // Read ciphertext
    let ciphertext = std::fs::read(input_path)
        .map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    // Decrypt
    let plaintext = decrypt(&ciphertext, password, &metadata)?;

    // Write plaintext
    std::fs::write(output_path, plaintext)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

    log::info!("✅ File decrypted: {} → {}", input_path.display(), output_path.display());

    Ok(())
}

/// Validate password strength
///
/// Returns error if password doesn't meet minimum requirements:
/// - At least 12 characters
/// - Contains uppercase, lowercase, digit, and special character
pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters long".to_string());
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character".to_string());
    }

    Ok(())
}

// Module for base64 encoding/decoding
mod base64 {
    pub fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        for chunk in data.chunks(3) {
            let b1 = chunk[0];
            let b2 = chunk.get(1).copied().unwrap_or(0);
            let b3 = chunk.get(2).copied().unwrap_or(0);

            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

            let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let chars: Vec<char> = alphabet.chars().collect();

            write!(&mut s, "{}", chars[((n >> 18) & 63) as usize]).unwrap();
            write!(&mut s, "{}", chars[((n >> 12) & 63) as usize]).unwrap();
            if chunk.len() > 1 {
                write!(&mut s, "{}", chars[((n >> 6) & 63) as usize]).unwrap();
            } else {
                s.push('=');
            }
            if chunk.len() > 2 {
                write!(&mut s, "{}", chars[(n & 63) as usize]).unwrap();
            } else {
                s.push('=');
            }
        }
        s
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = Vec::new();

        let s = s.trim_end_matches('=');
        let chunks: Vec<&str> = s.as_bytes().chunks(4).map(|c| std::str::from_utf8(c).unwrap()).collect();

        for chunk in chunks {
            let mut n = 0u32;
            for (i, c) in chunk.chars().enumerate() {
                if c == '=' {
                    break;
                }
                let val = alphabet.find(c).ok_or("Invalid base64 character")? as u32;
                n |= val << (18 - i * 6);
            }

            result.push((n >> 16) as u8);
            if chunk.len() > 2 {
                result.push((n >> 8) as u8);
            }
            if chunk.len() > 3 {
                result.push(n as u8);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "TestPassword123!";
        let salt = generate_salt();

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        // Same password + salt = same key
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32); // 256 bits
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let plaintext = b"Hello, World! This is a secret message.";
        let password = "SuperSecret123!";

        let (ciphertext, metadata) = encrypt(plaintext, password).unwrap();

        // Ciphertext should be different from plaintext
        assert_ne!(&ciphertext[..plaintext.len()], plaintext);

        // Decrypt
        let decrypted = decrypt(&ciphertext, password, &metadata).unwrap();

        // Should match original
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plaintext = b"Secret data";
        let password = "CorrectPassword123!";
        let wrong_password = "WrongPassword456!";

        let (ciphertext, metadata) = encrypt(plaintext, password).unwrap();

        // Wrong password should fail
        let result = decrypt(&ciphertext, wrong_password, &metadata);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Decryption failed"));
    }

    #[test]
    fn test_password_validation() {
        // Too short
        assert!(validate_password_strength("Short1!").is_err());

        // Missing uppercase
        assert!(validate_password_strength("lowercase123!").is_err());

        // Missing lowercase
        assert!(validate_password_strength("UPPERCASE123!").is_err());

        // Missing digit
        assert!(validate_password_strength("NoDigitsHere!").is_err());

        // Missing special char
        assert!(validate_password_strength("NoSpecial123").is_err());

        // Valid password
        assert!(validate_password_strength("ValidPassword123!").is_ok());
    }
}
