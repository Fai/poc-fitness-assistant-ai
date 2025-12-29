//! Property-based tests for authentication
//!
//! Feature: fitness-assistant-ai
//! Property 20: Authentication Enforcement
//! Test that unauthenticated requests return 401

#[cfg(test)]
mod tests {
    use crate::auth::JwtService;
    use crate::config::AppConfig;
    use crate::routes::create_router;
    use crate::state::AppState;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use proptest::prelude::*;
    use sqlx::PgPool;
    use tower::ServiceExt;

    /// Create a test app state with a mock database pool (sync version for proptest)
    fn create_test_state_sync() -> AppState {
        let config = AppConfig::default();
        let pool = PgPool::connect_lazy("postgres://test:test@localhost:5432/test").unwrap();
        AppState::new(pool, config)
    }

    /// Generate random invalid tokens
    fn invalid_token_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            // Empty token
            Just("".to_string()),
            // Random string (not a valid JWT)
            "[a-zA-Z0-9]{10,50}".prop_map(|s| s),
            // Malformed JWT (wrong number of parts)
            "[a-zA-Z0-9]{10}\\.[a-zA-Z0-9]{10}".prop_map(|s| s),
            // Valid format but invalid signature
            "[a-zA-Z0-9_-]{20}\\.[a-zA-Z0-9_-]{20}\\.[a-zA-Z0-9_-]{20}".prop_map(|s| s),
        ]
    }

    /// Generate random authorization header formats
    fn auth_header_strategy() -> impl Strategy<Value = Option<String>> {
        prop_oneof![
            // No header
            Just(None),
            // Missing Bearer prefix
            invalid_token_strategy().prop_map(|t| Some(t)),
            // Wrong prefix
            invalid_token_strategy().prop_map(|t| Some(format!("Basic {}", t))),
            // Bearer with invalid token
            invalid_token_strategy().prop_map(|t| Some(format!("Bearer {}", t))),
        ]
    }

    // Feature: fitness-assistant-ai, Property 20: Authentication Enforcement
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// Property: Unauthenticated requests to protected endpoints return 401
        #[test]
        fn prop_unauthenticated_requests_return_401(
            auth_header in auth_header_strategy()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let state = create_test_state_sync();
                let app = create_router(state);

                // Build request to protected endpoint
                let mut request_builder = Request::builder()
                    .uri("/api/v1/auth/me")
                    .method("GET");

                if let Some(header) = auth_header {
                    request_builder = request_builder.header("Authorization", header);
                }

                let request = request_builder.body(Body::empty()).unwrap();
                let response = app.oneshot(request).await.unwrap();

                // All invalid auth should return 401
                prop_assert_eq!(
                    response.status(),
                    StatusCode::UNAUTHORIZED,
                    "Expected 401 for unauthenticated request"
                );

                Ok(())
            })?;
        }
    }

    #[tokio::test]
    async fn test_missing_auth_header_returns_401() {
        let state = create_test_state_sync();
        let app = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/auth/me")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_bearer_token_returns_401() {
        let state = create_test_state_sync();
        let app = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/auth/me")
            .method("GET")
            .header("Authorization", "Bearer invalid.token.here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_wrong_auth_scheme_returns_401() {
        let state = create_test_state_sync();
        let app = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/auth/me")
            .method("GET")
            .header("Authorization", "Basic dXNlcjpwYXNz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_token_with_wrong_secret_returns_401() {
        let state = create_test_state_sync();
        
        // Create a JWT service with a DIFFERENT secret
        let jwt_service = JwtService::new(
            "wrong-secret-key",
            3600,
            86400,
        );

        let user_id = uuid::Uuid::new_v4();
        let token = jwt_service.generate_access_token(user_id).unwrap();

        let app = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/auth/me")
            .method("GET")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_valid_token_passes_auth() {
        let state = create_test_state_sync();
        
        // Create a valid token using the state's JWT service
        let user_id = uuid::Uuid::new_v4();
        let valid_token = state.jwt().generate_access_token(user_id).unwrap();

        let app = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/auth/me")
            .method("GET")
            .header("Authorization", format!("Bearer {}", valid_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        // With valid token, we should NOT get 401
        // We might get 404 (user not found in DB) or 500 (DB connection failed)
        // but NOT 401 - the auth middleware passed
        assert_ne!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Valid token should pass authentication"
        );
    }
}
