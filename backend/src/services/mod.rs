//! Business logic services
//!
//! Services encapsulate business logic and coordinate between
//! repositories and external systems.

pub mod insights;
pub mod profile;
pub mod user;
pub mod weight;

pub use insights::HealthInsightsService;
pub use profile::ProfileService;
pub use user::UserService;
pub use weight::WeightService;
