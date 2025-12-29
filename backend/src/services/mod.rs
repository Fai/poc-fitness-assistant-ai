//! Business logic services
//!
//! Services encapsulate business logic and coordinate between
//! repositories and external systems.

pub mod user;
pub mod weight;

pub use user::UserService;
pub use weight::WeightService;
