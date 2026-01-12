//! Route definitions for the Fitness Assistant API
//!
//! This module organizes all API routes and applies middleware.

use crate::config::AppConfig;
use crate::state::AppState;
use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

mod auth;
mod biometrics;
mod biomarkers;
mod exercise;
mod export;
mod goals;
mod health;
mod hydration;
mod nutrition;
mod profile;
mod sleep;
mod weight;

#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod weight_tests;

pub use auth::auth_routes;
pub use biometrics::biometrics_routes;
pub use biomarkers::biomarkers_routes;
pub use exercise::exercise_routes;
pub use export::export_routes;
pub use goals::goals_routes;
pub use hydration::hydration_routes;
pub use nutrition::nutrition_routes;
pub use profile::profile_routes;
pub use sleep::sleep_routes;
pub use weight::weight_routes;

/// Create the main application router with all middleware
pub fn create_router(state: AppState) -> Router {
    // Build CORS layer based on configuration
    let cors = build_cors_layer(&state.config);
    
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        .nest("/api/v1", api_routes())
        // Apply middleware layers
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Build CORS layer based on configuration
/// 
/// In development (empty allowed_origins), allows any origin.
/// In production, restricts to configured origins.
fn build_cors_layer(config: &AppConfig) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT]);
    
    if config.server.allowed_origins.is_empty() {
        // Development mode: allow any origin
        base.allow_origin(tower_http::cors::Any)
    } else {
        // Production mode: restrict to configured origins
        let origins: Vec<HeaderValue> = config
            .server
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        
        base.allow_origin(origins)
    }
}

/// API v1 routes
fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { "Fitness Assistant API v1" }))
        .nest("/auth", auth::auth_routes())
        .nest("/weight", weight::weight_routes())
        .nest("/profile", profile::profile_routes())
        .nest("/nutrition", nutrition::nutrition_routes())
        .nest("/exercise", exercise::exercise_routes())
        .nest("/hydration", hydration::hydration_routes())
        .nest("/sleep", sleep::sleep_routes())
        .nest("/biometrics", biometrics::biometrics_routes())
        .nest("/goals", goals::goals_routes())
        .nest("/biomarkers", biomarkers::biomarkers_routes())
        .nest("/export", export::export_routes())
}
