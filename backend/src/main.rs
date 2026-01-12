//! Fitness Assistant AI Backend
//!
//! A privacy-focused health and longevity tracking platform.
//!
//! ## Architecture
//!
//! The backend follows a layered architecture:
//! - Routes: HTTP request handling and routing
//! - Services: Business logic (to be implemented)
//! - Repositories: Data access (to be implemented)
//! - Database: PostgreSQL with SQLx

use anyhow::Result;
use fitness_assistant_backend::{config, db, routes, state::AppState};
use redis::aio::ConnectionManager;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    init_tracing();

    // Load configuration
    let config = config::AppConfig::load()?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        env = if config::AppConfig::is_production() { "production" } else { "development" },
        "Starting Fitness Assistant AI Backend"
    );

    // Validate production configuration
    if config::AppConfig::is_production() {
        validate_production_config(&config)?;
    }

    // Create database pool
    info!("Connecting to database...");
    let db_pool = db::create_pool(&config.database.url, config.database.max_connections).await?;

    // Run migrations (skip in production if using separate migration job)
    if !config::AppConfig::is_production() {
        info!("Running database migrations...");
        db::run_migrations(&db_pool).await?;
    }

    // Connect to Redis (optional - gracefully handle connection failure)
    let redis_conn = connect_redis(&config.redis.url).await;

    // Create application state
    let state = AppState::new(db_pool, redis_conn, config.clone());

    // Build application
    let app = routes::create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!(address = %addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Connect to Redis with graceful fallback
/// 
/// Returns None if Redis is unavailable, allowing the app to run without caching
async fn connect_redis(url: &str) -> Option<ConnectionManager> {
    info!("Connecting to Redis...");
    
    match redis::Client::open(url) {
        Ok(client) => {
            match ConnectionManager::new(client).await {
                Ok(conn) => {
                    info!("Redis connection established");
                    Some(conn)
                }
                Err(e) => {
                    warn!("Failed to connect to Redis: {}. Caching will be disabled.", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Invalid Redis URL: {}. Caching will be disabled.", e);
            None
        }
    }
}

/// Initialize tracing/logging
fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if config::AppConfig::is_production() {
            "fitness_assistant_backend=info,tower_http=info".into()
        } else {
            "fitness_assistant_backend=debug,tower_http=debug,sqlx=warn".into()
        }
    });

    let subscriber = tracing_subscriber::registry().with(env_filter);

    if config::AppConfig::is_production() {
        // JSON logging for production (better for log aggregation)
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        // Pretty logging for development
        subscriber
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}

/// Validate configuration for production deployment
fn validate_production_config(config: &config::AppConfig) -> Result<()> {
    let mut errors = Vec::new();

    // Check JWT secret is not default
    if config.jwt.secret.contains("development") || config.jwt.secret.len() < 32 {
        errors.push("JWT secret must be at least 32 characters and not contain 'development'");
    }

    // Check database URL is not localhost in production
    if config.database.url.contains("localhost") || config.database.url.contains("127.0.0.1") {
        warn!("Database URL contains localhost - ensure this is intentional for production");
    }

    if !errors.is_empty() {
        for err in &errors {
            error!("Configuration error: {}", err);
        }
        anyhow::bail!("Invalid production configuration");
    }

    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, starting graceful shutdown");
        }
    }
}
