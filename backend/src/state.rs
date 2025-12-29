//! Application state management
//!
//! This module provides the shared application state that is passed
//! to all request handlers via Axum's state extraction.
//!
//! # Design Principles
//! 
//! 1. **Pre-compute expensive resources**: JWT keys, DB pools are created once
//! 2. **Cheap cloning**: All fields use Arc or are already Clone-cheap
//! 3. **Immutable after creation**: State is read-only during request handling

use crate::auth::JwtService;
use crate::config::AppConfig;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state
///
/// This struct holds all shared resources that handlers need access to.
/// All fields are designed for cheap cloning across async tasks.
///
/// # Performance
/// 
/// - `db`: PgPool is internally Arc'd, cloning is O(1)
/// - `config`: Wrapped in Arc, cloning is O(1)
/// - `jwt`: Pre-computed keys wrapped in Arc, cloning is O(1)
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db: PgPool,
    /// Application configuration
    pub config: Arc<AppConfig>,
    /// Pre-initialized JWT service with cached keys
    pub jwt: JwtService,
}

impl AppState {
    /// Create a new application state
    /// 
    /// # Note
    /// This pre-computes JWT keys from the config secret.
    /// The keys are expensive to derive, so this should only
    /// be called once at application startup.
    pub fn new(db: PgPool, config: AppConfig) -> Self {
        // Pre-compute JWT service with cached keys
        let jwt = JwtService::new(
            &config.jwt.secret,
            config.jwt.access_token_expiry_secs,
            config.jwt.refresh_token_expiry_secs,
        );

        Self {
            db,
            config: Arc::new(config),
            jwt,
        }
    }

    /// Get a reference to the database pool
    #[inline]
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    /// Get a reference to the configuration
    #[inline]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get a reference to the JWT service
    #[inline]
    pub fn jwt(&self) -> &JwtService {
        &self.jwt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_state_clone_is_cheap() {
        // This test ensures our state design allows cheap cloning
        let config = AppConfig::default();
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test").unwrap();
        let state = AppState::new(pool, config);
        
        // Clone should be O(1) - just Arc increments
        let _cloned = state.clone();
    }

    #[tokio::test]
    async fn test_jwt_service_is_precomputed() {
        let config = AppConfig::default();
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test").unwrap();
        let state = AppState::new(pool, config);
        
        // JWT service should be ready to use
        let user_id = uuid::Uuid::new_v4();
        let token = state.jwt().generate_access_token(user_id).unwrap();
        assert!(!token.is_empty());
    }
}
