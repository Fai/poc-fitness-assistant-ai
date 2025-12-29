//! Database connection and pool management
//!
//! This module provides database connection pooling with proper
//! configuration for production use including health checks,
//! connection timeouts, and retry logic.

use anyhow::Result;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;
use tracing::{info, warn};

/// Database configuration for pool creation
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,      // 10 minutes
            max_lifetime_secs: 1800,     // 30 minutes
        }
    }
}

/// Create a PostgreSQL connection pool with production-ready settings
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool> {
    let config = DbConfig {
        url: database_url.to_string(),
        max_connections,
        ..Default::default()
    };
    create_pool_with_config(&config).await
}

/// Create a PostgreSQL connection pool with custom configuration
pub async fn create_pool_with_config(config: &DbConfig) -> Result<PgPool> {
    let connect_options = PgConnectOptions::from_str(&config.url)?
        .application_name("fitness-assistant");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await?;

    info!(
        "Database pool created: max={}, min={}",
        config.max_connections, config.min_connections
    );

    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(pool).await?;
    info!("Database migrations completed successfully");
    Ok(())
}

/// Check database health
pub async fn health_check(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| {
            warn!("Database health check failed: {}", e);
            e.into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_db_config() {
        let config = DbConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.acquire_timeout_secs, 30);
    }
}
