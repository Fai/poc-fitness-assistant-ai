//! Password hashing using argon2
//!
//! Provides secure password hashing and verification.
//! 
//! # Performance Considerations
//! 
//! Argon2 is intentionally CPU-intensive. For high-throughput scenarios,
//! consider using `spawn_blocking` to avoid blocking the async runtime.

use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Password hashing service
/// 
/// Uses Argon2id which is the recommended variant for password hashing.
/// It provides resistance against both side-channel and GPU-based attacks.
pub struct PasswordService;

impl PasswordService {
    /// Hash a password using argon2 (blocking operation)
    /// 
    /// # Performance Note
    /// This is CPU-intensive. For async contexts, wrap in `spawn_blocking`:
    /// ```ignore
    /// let hash = tokio::task::spawn_blocking(move || {
    ///     PasswordService::hash(&password)
    /// }).await??;
    /// ```
    pub fn hash(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
        Ok(hash.to_string())
    }

    /// Hash a password asynchronously (non-blocking)
    /// 
    /// Spawns the CPU-intensive work on a blocking thread pool,
    /// preventing it from blocking the async runtime.
    pub async fn hash_async(password: String) -> Result<String> {
        tokio::task::spawn_blocking(move || Self::hash(&password))
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
    }

    /// Verify a password against a hash (blocking operation)
    /// 
    /// # Performance Note
    /// This is CPU-intensive. For async contexts, use `verify_async`.
    pub fn verify(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid hash format: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Verify a password asynchronously (non-blocking)
    /// 
    /// Spawns the CPU-intensive work on a blocking thread pool.
    pub async fn verify_async(password: String, hash: String) -> Result<bool> {
        tokio::task::spawn_blocking(move || Self::verify(&password, &hash))
            .await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "secure_password_123";
        let hash = PasswordService::hash(password).unwrap();

        assert!(PasswordService::verify(password, &hash).unwrap());
        assert!(!PasswordService::verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "test_password";
        let hash1 = PasswordService::hash(password).unwrap();
        let hash2 = PasswordService::hash(password).unwrap();

        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordService::verify(password, &hash1).unwrap());
        assert!(PasswordService::verify(password, &hash2).unwrap());
    }

    #[tokio::test]
    async fn test_async_hash_and_verify() {
        let password = "async_test_password".to_string();
        let hash = PasswordService::hash_async(password.clone()).await.unwrap();

        assert!(PasswordService::verify_async(password.clone(), hash.clone()).await.unwrap());
        assert!(!PasswordService::verify_async("wrong".to_string(), hash).await.unwrap());
    }
}
