//! Fitness Assistant Shared Library
//!
//! This crate contains shared types, models, and utilities used across
//! the backend, frontend, and WASM modules.

pub mod errors;
pub mod health_metrics;
pub mod models;
pub mod types;
pub mod units;
pub mod validation;

// Re-export commonly used items
pub use errors::*;
pub use health_metrics::*;
pub use types::*;

// Export units module items (canonical source for unit types)
pub use units::*;

// Export models (excluding unit types which are re-exported from units)
pub use models::{DataSource, Goal, GoalStatus, GoalType, User, UserSettings};
