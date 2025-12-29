//! Route definitions for the Fitness Assistant API
//!
//! This module organizes all API routes and applies middleware.

use crate::state::AppState;
use axum::{
    http::{header, Method},
    routing::get,
    Router,
};
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

mod auth;
mod health;
mod profile;
mod weight;

#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod weight_tests;

pub use auth::auth_routes;
pub use profile::profile_routes;
pub use weight::weight_routes;

/// Create the main application router with all middleware
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        .nest("/api/v1", api_routes())
        // Apply middleware layers
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// API v1 routes
fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { "Fitness Assistant API v1" }))
        .nest("/auth", auth::auth_routes())
        .nest("/weight", weight::weight_routes())
        .nest("/profile", profile::profile_routes())
    // Future routes will be nested here:
    // .nest("/nutrition", nutrition_routes())
    // .nest("/exercise", exercise_routes())
}
