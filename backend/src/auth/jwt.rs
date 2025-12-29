//! JWT token generation and validation
//!
//! Provides access and refresh token management with pre-computed keys
//! for optimal performance.

use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Token type: "access" or "refresh"
    pub token_type: String,
    /// JWT ID for token revocation tracking (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

/// Pre-computed JWT keys for efficient token operations
/// These are expensive to create, so we cache them in AppState
#[derive(Clone)]
pub struct JwtKeys {
    encoding: Arc<EncodingKey>,
    decoding: Arc<DecodingKey>,
}

impl JwtKeys {
    /// Create new JWT keys from secret
    /// This should be called once at startup
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: Arc::new(EncodingKey::from_secret(secret.as_bytes())),
            decoding: Arc::new(DecodingKey::from_secret(secret.as_bytes())),
        }
    }

    pub fn encoding(&self) -> &EncodingKey {
        &self.encoding
    }

    pub fn decoding(&self) -> &DecodingKey {
        &self.decoding
    }
}

/// JWT service configuration
#[derive(Clone)]
pub struct JwtConfig {
    pub access_token_expiry_secs: i64,
    pub refresh_token_expiry_secs: i64,
}

/// JWT service for token operations
/// 
/// Design: Uses pre-computed keys to avoid expensive key derivation
/// on every request. Keys are wrapped in Arc for cheap cloning.
#[derive(Clone)]
pub struct JwtService {
    keys: JwtKeys,
    config: JwtConfig,
}

impl JwtService {
    /// Create a new JWT service with pre-computed keys
    /// 
    /// # Performance Note
    /// Call this once at application startup and store in AppState.
    /// Do NOT create per-request.
    pub fn new(secret: &str, access_token_expiry_secs: i64, refresh_token_expiry_secs: i64) -> Self {
        Self {
            keys: JwtKeys::new(secret),
            config: JwtConfig {
                access_token_expiry_secs,
                refresh_token_expiry_secs,
            },
        }
    }

    /// Create from pre-computed keys (for sharing across handlers)
    pub fn from_keys(keys: JwtKeys, config: JwtConfig) -> Self {
        Self { keys, config }
    }

    /// Generate an access token for a user
    #[inline]
    pub fn generate_access_token(&self, user_id: Uuid) -> Result<String> {
        self.generate_token(user_id, "access", self.config.access_token_expiry_secs)
    }

    /// Generate a refresh token for a user
    #[inline]
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String> {
        self.generate_token(user_id, "refresh", self.config.refresh_token_expiry_secs)
    }

    /// Generate a token with specified type and expiry
    fn generate_token(&self, user_id: Uuid, token_type: &str, expiry_secs: i64) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(expiry_secs);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: token_type.to_string(),
            jti: None, // Can be used for token revocation
        };

        encode(&Header::default(), &claims, self.keys.encoding())
            .map_err(|e| anyhow::anyhow!("Failed to generate {} token: {}", token_type, e))
    }

    /// Validate a token and return claims
    #[inline]
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, self.keys.decoding(), &Validation::default())
            .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

        Ok(token_data.claims)
    }

    /// Validate an access token specifically
    #[inline]
    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != "access" {
            return Err(anyhow::anyhow!("Not an access token"));
        }
        Ok(claims)
    }

    /// Validate a refresh token specifically
    #[inline]
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != "refresh" {
            return Err(anyhow::anyhow!("Not a refresh token"));
        }
        Ok(claims)
    }

    /// Get access token expiry in seconds
    #[inline]
    pub fn access_token_expiry_secs(&self) -> i64 {
        self.config.access_token_expiry_secs
    }

    /// Get the pre-computed keys (for sharing)
    pub fn keys(&self) -> &JwtKeys {
        &self.keys
    }

    /// Get the config (for sharing)
    pub fn jwt_config(&self) -> &JwtConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> JwtService {
        JwtService::new("test-secret", 3600, 604800)
    }

    #[test]
    fn test_generate_and_validate_access_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let token = service.generate_access_token(user_id).unwrap();
        let claims = service.validate_access_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let token = service.generate_refresh_token(user_id).unwrap();
        let claims = service.validate_refresh_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_access_token_rejected_as_refresh() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let token = service.generate_access_token(user_id).unwrap();
        let result = service.validate_refresh_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token_rejected() {
        let service = create_test_service();
        let result = service.validate_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_service_is_clone_cheap() {
        let service = create_test_service();
        let _cloned = service.clone(); // Should be cheap due to Arc
    }

    #[test]
    fn test_keys_can_be_shared() {
        let service = create_test_service();
        let keys = service.keys().clone();
        let config = service.jwt_config().clone();
        
        let service2 = JwtService::from_keys(keys, config);
        let user_id = Uuid::new_v4();
        
        // Both services should produce valid tokens
        let token = service.generate_access_token(user_id).unwrap();
        let claims = service2.validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
    }
}
