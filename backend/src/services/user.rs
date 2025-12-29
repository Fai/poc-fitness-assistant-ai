//! User service for authentication and user management
//!
//! # Performance Optimizations
//! 
//! - Password hashing/verification runs on blocking thread pool
//! - JWT service is passed by reference (pre-computed keys)
//! - Database queries use connection pooling

use crate::auth::{JwtService, PasswordService};
use crate::error::ApiError;
use crate::repositories::UserRepository;
use fitness_assistant_shared::types::{AuthTokens, UserProfile};
use sqlx::PgPool;
use uuid::Uuid;
use validator::ValidateEmail;

/// User service for authentication operations
pub struct UserService;

impl UserService {
    /// Register a new user
    /// 
    /// # Performance
    /// Password hashing is offloaded to blocking thread pool via `spawn_blocking`.
    pub async fn register(
        pool: &PgPool,
        jwt_service: &JwtService,
        email: &str,
        password: &str,
    ) -> Result<AuthTokens, ApiError> {
        // Validate email format
        if !email.validate_email() {
            return Err(ApiError::Validation("Invalid email format".to_string()));
        }

        // Validate password strength
        if password.len() < 8 {
            return Err(ApiError::Validation(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        // Check if email already exists
        if UserRepository::email_exists(pool, email)
            .await
            .map_err(ApiError::Internal)?
        {
            return Err(ApiError::Conflict("Email already registered".to_string()));
        }

        // Hash password on blocking thread pool (CPU-intensive)
        let password_owned = password.to_string();
        let password_hash = PasswordService::hash_async(password_owned)
            .await
            .map_err(ApiError::Internal)?;

        // Create user
        let user = UserRepository::create(pool, email, &password_hash)
            .await
            .map_err(ApiError::Internal)?;

        // Generate tokens (uses pre-computed keys - fast)
        let access_token = jwt_service
            .generate_access_token(user.id)
            .map_err(ApiError::Internal)?;
        let refresh_token = jwt_service
            .generate_refresh_token(user.id)
            .map_err(ApiError::Internal)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_service.access_token_expiry_secs(),
        })
    }

    /// Login with email and password
    /// 
    /// # Performance
    /// Password verification is offloaded to blocking thread pool.
    pub async fn login(
        pool: &PgPool,
        jwt_service: &JwtService,
        email: &str,
        password: &str,
    ) -> Result<AuthTokens, ApiError> {
        // Find user by email
        let user = UserRepository::find_by_email(pool, email)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password on blocking thread pool (CPU-intensive)
        let password_owned = password.to_string();
        let hash_owned = user.password_hash.clone();
        let valid = PasswordService::verify_async(password_owned, hash_owned)
            .await
            .map_err(ApiError::Internal)?;

        if !valid {
            return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
        }

        // Generate tokens (uses pre-computed keys - fast)
        let access_token = jwt_service
            .generate_access_token(user.id)
            .map_err(ApiError::Internal)?;
        let refresh_token = jwt_service
            .generate_refresh_token(user.id)
            .map_err(ApiError::Internal)?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_service.access_token_expiry_secs(),
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(
        pool: &PgPool,
        jwt_service: &JwtService,
        refresh_token: &str,
    ) -> Result<AuthTokens, ApiError> {
        // Validate refresh token
        let claims = jwt_service
            .validate_refresh_token(refresh_token)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid refresh token: {}", e)))?;

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))?;

        // Verify user still exists
        let _user = UserRepository::find_by_id(pool, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))?;

        // Generate new tokens
        let access_token = jwt_service
            .generate_access_token(user_id)
            .map_err(ApiError::Internal)?;
        let new_refresh_token = jwt_service
            .generate_refresh_token(user_id)
            .map_err(ApiError::Internal)?;

        Ok(AuthTokens {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_service.access_token_expiry_secs(),
        })
    }

    /// Get user profile
    pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> Result<UserProfile, ApiError> {
        let user = UserRepository::find_by_id(pool, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        Ok(UserProfile {
            id: user.id.to_string(),
            email: user.email,
            created_at: user.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require database - marked with #[ignore]
}
