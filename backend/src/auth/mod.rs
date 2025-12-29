//! Authentication module
//!
//! Provides JWT-based authentication with argon2 password hashing.

mod jwt;
mod middleware;
mod password;

pub use jwt::{Claims, JwtService};
pub use middleware::{auth_middleware, AuthUser};
pub use password::PasswordService;
