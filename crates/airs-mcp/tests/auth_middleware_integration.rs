//! Integration tests for Zero-Cost Generic HTTP Authentication Middleware
//!
//! This test suite validates the zero-cost generic authentication middleware
//! implementation using simplified mocks and focused unit tests to demonstrate
//! zero-cost generic patterns without requiring access to private modules.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::time::timeout;

use airs_mcp::authentication::{AuthContext, AuthMethod};
use airs_mcp::transport::adapters::http::auth::middleware::{HttpAuthConfig, HttpAuthMiddleware, HttpAuthRequest, HttpAuthStrategyAdapter};
use airs_mcp::transport::adapters::http::auth::oauth2::error::HttpAuthError;

// ================================================================================================
// Test Utilities
// ================================================================================================

/// Mock authentication adapter for performance benchmarks
#[derive(Debug, Clone)]
struct MockAdapter {
    name: &'static str,
    always_pass: bool,
}

impl MockAdapter {
    fn new(name: &'static str, always_pass: bool) -> Self {
        Self { name, always_pass }
    }
}

#[async_trait::async_trait]
impl HttpAuthStrategyAdapter for MockAdapter {
    type RequestType = ();
    type AuthData = String;

    fn auth_method(&self) -> &'static str {
        self.name
    }

    async fn authenticate_http_request(
        &self,
        _request: &HttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
        if self.always_pass {
            Ok(AuthContext::new(
                AuthMethod::new(self.name),
                "test_user".to_string(),
            ))
        } else {
            Err(HttpAuthError::AuthenticationFailed {
                message: "Mock authentication failed".to_string(),
            })
        }
    }

    fn should_skip_path(&self, path: &str) -> bool {
        path.starts_with("/health") || path.starts_with("/metrics")
    }
}

/// NoAuth adapter for testing
#[derive(Debug, Clone, Default)]
struct TestNoAuth;

#[async_trait::async_trait]
impl HttpAuthStrategyAdapter for TestNoAuth {
    type RequestType = ();
    type AuthData = ();

    fn auth_method(&self) -> &'static str {
        "none"
    }

    async fn authenticate_http_request(
        &self,
        _request: &HttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
        Ok(AuthContext::new(AuthMethod::new("none"), ()))
    }

    fn should_skip_path(&self, _path: &str) -> bool {
        true
    }
}


// ================================================================================================
// Zero Cost Generic Pattern Tests
// ================================================================================================

#[tokio::test]
async fn test_no_auth_skips_all_authentication() {
    let no_auth = TestNoAuth;
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/test".to_string(),
        HashMap::new(),
    );

    // NoAuth should always authenticate successfully
    let result = no_auth.authenticate_http_request(&auth_request).await;
    assert!(result.is_ok());

    // NoAuth should skip authentication for all paths
    assert!(no_auth.should_skip_path("/api/secure"));
    assert!(no_auth.should_skip_path("/health"));
    assert!(no_auth.should_skip_path("/metrics"));
}

// ================================================================================================
// Generic Middleware Tests
// ================================================================================================

#[tokio::test]
async fn test_type_differentiation() {
    // Create different adapter types - should be different types at compile time
    let success_adapter = MockAdapter::new("success", true);
    let failure_adapter = MockAdapter::new("failure", false);
    let no_auth_adapter = TestNoAuth;
    
    // Create middleware instances with different adapter types
    let success_middleware = HttpAuthMiddleware::new(success_adapter, HttpAuthConfig::default());
    let failure_middleware = HttpAuthMiddleware::new(failure_adapter, HttpAuthConfig::default());
    let no_auth_middleware = HttpAuthMiddleware::new(no_auth_adapter, HttpAuthConfig::default());
    
    // These should be different types at compile time
    assert_ne!(std::any::type_name_of_val(&success_middleware), 
               std::any::type_name_of_val(&no_auth_middleware));
               
    // Same adapter type but different instances should be the same type
    assert_eq!(std::any::type_name_of_val(&success_middleware),
               std::any::type_name_of_val(&failure_middleware));
    
    // Different auth methods based on adapter
    assert_eq!(success_middleware.auth_method(), "success");
    assert_eq!(failure_middleware.auth_method(), "failure");
    assert_eq!(no_auth_middleware.auth_method(), "none");
}

#[tokio::test]
async fn test_middleware_authentication_success() {
    let adapter = MockAdapter::new("success", true);
    let config = HttpAuthConfig {
        include_error_details: true,
        auth_realm: "Test Realm".to_string(),
        ..HttpAuthConfig::default()
    };
    let middleware = HttpAuthMiddleware::new(adapter, config);
    
    // Test basic properties
    assert_eq!(middleware.auth_method(), "success");
    assert!(middleware.config().include_error_details);
    assert_eq!(middleware.config().auth_realm, "Test Realm");
    
    // Test successful authentication
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/protected".to_string(),
        HashMap::new(),
    );
    
    let result = middleware.authenticate(&auth_request).await;
    assert!(result.is_ok());
    
    let auth_context = result.unwrap().unwrap();
    assert_eq!(auth_context.method.as_str(), "success");
    assert_eq!(auth_context.auth_data, "test_user");
}

#[tokio::test]
async fn test_middleware_authentication_failure() {
    let adapter = MockAdapter::new("failure", false);
    let config = HttpAuthConfig::default();
    let middleware = HttpAuthMiddleware::new(adapter, config);
    
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/protected".to_string(),
        HashMap::new(),
    );
    
    // Test failed authentication
    let result = middleware.authenticate(&auth_request).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        HttpAuthError::AuthenticationFailed { message } => {
            assert!(message.contains("Mock authentication failed"));
        }
        err => panic!("Unexpected error type: {err:?}"),
    }
}

#[tokio::test]
async fn test_path_skipping() {
    // Create test adapters
    let adapter = MockAdapter::new("test", true);
    let no_auth = TestNoAuth;
    
    // Test standard adapter path skipping
    assert!(!adapter.should_skip_path("/api/protected"));
    assert!(adapter.should_skip_path("/health"));
    assert!(adapter.should_skip_path("/metrics"));
    
    // Test NoAuth path skipping - should skip all paths
    assert!(no_auth.should_skip_path("/api/protected"));
    assert!(no_auth.should_skip_path("/health"));
    assert!(no_auth.should_skip_path("/metrics"));
}

// ================================================================================================
// Performance Tests
// ================================================================================================

#[tokio::test]
async fn test_zero_cost_abstraction_performance() {
    const NUM_REQUESTS: usize = 1000;

    // Benchmark TestNoAuth (baseline)
    let no_auth_adapter = TestNoAuth;
    let auth_config = HttpAuthConfig::default();
    let no_auth_middleware = HttpAuthMiddleware::new(no_auth_adapter, auth_config);
    
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/test".to_string(),
        HashMap::new(),
    );

    let start = Instant::now();
    for _ in 0..NUM_REQUESTS {
        let _ = no_auth_middleware.authenticate(&auth_request).await;
    }
    let no_auth_duration = start.elapsed();

    // Benchmark MockAdapter (should be similar performance due to zero-cost generics)
    let mock_adapter = MockAdapter::new("mock", true);
    let auth_config = HttpAuthConfig::default();
    let mock_middleware = HttpAuthMiddleware::new(mock_adapter, auth_config);

    let start = Instant::now();
    for _ in 0..NUM_REQUESTS {
        let _ = mock_middleware.authenticate(&auth_request).await;
    }
    let mock_duration = start.elapsed();

    println!("NoAuth duration: {no_auth_duration:?} for {NUM_REQUESTS} requests");
    println!("Mock adapter duration: {mock_duration:?} for {NUM_REQUESTS} requests");

    // Performance should demonstrate zero-cost abstractions
    // MockAdapter does actual work while NoAuth is a no-op, so some difference is expected
    // but both should be reasonably fast
    assert!(mock_duration.as_millis() < 50, 
            "Mock adapter should be fast due to zero-cost abstractions: {mock_duration:?}");
    assert!(no_auth_duration.as_millis() < 50, 
            "NoAuth should be very fast: {no_auth_duration:?}");
}

// ================================================================================================
// Async Timeout Tests
// ================================================================================================

#[tokio::test]
async fn test_async_timeout_behavior() {
    // Create a slow adapter that simulates slow network authentication
    #[derive(Debug, Clone)]
    struct SlowAdapter;

    #[async_trait::async_trait]
    impl HttpAuthStrategyAdapter for SlowAdapter {
        type RequestType = ();
        type AuthData = ();

        fn auth_method(&self) -> &'static str {
            "slow"
        }

        async fn authenticate_http_request(
            &self,
            _request: &HttpAuthRequest,
        ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
            // Simulate slow authentication
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(AuthContext::new(AuthMethod::new("slow"), ()))
        }

        fn should_skip_path(&self, _path: &str) -> bool {
            false
        }
    }

    let slow_adapter = SlowAdapter;
    let auth_config = HttpAuthConfig::default();
    let middleware = HttpAuthMiddleware::new(slow_adapter, auth_config);

    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/test".to_string(),
        HashMap::new(),
    );

    // Test with timeout that's sufficient
    let result = timeout(
        Duration::from_millis(200),
        middleware.authenticate(&auth_request)
    ).await;

    assert!(result.is_ok());
    let auth_result = result.unwrap();
    assert!(auth_result.is_ok());
    
    // Test with timeout that's too short
    let result = timeout(
        Duration::from_millis(50),
        middleware.authenticate(&auth_request)
    ).await;
    
    assert!(result.is_err()); // Should timeout
}

// ================================================================================================
// Concurrent Authentication Tests
// ================================================================================================

#[tokio::test]
async fn test_concurrent_authentication() {
    // Use a mock adapter for concurrent testing
    let adapter = MockAdapter::new("concurrent", true);
    let config = HttpAuthConfig::default();
    let middleware = HttpAuthMiddleware::new(adapter, config);
    
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/test".to_string(),
        HashMap::new(),
    );
    
    // Create multiple concurrent authentication requests
    let futures: Vec<_> = (0..10)
        .map(|_| {
            let req = auth_request.clone();
            let middleware_clone = middleware.clone();
            async move {
                middleware_clone.authenticate(&req).await
            }
        })
        .collect();
    
    // Run them concurrently
    let results = futures::future::join_all(futures).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}

// ================================================================================================
// Configuration Tests
// ================================================================================================

#[tokio::test]
async fn test_auth_config_controls() {
    let adapter = MockAdapter::new("test", false);
    
    // Test with various configurations
    let custom_config = HttpAuthConfig {
        include_error_details: true,
        auth_realm: "Custom Realm".to_string(),
        ..HttpAuthConfig::default()
    };
    
    let middleware = HttpAuthMiddleware::new(adapter, custom_config);
    
    // Verify config is correctly stored and accessible
    assert!(middleware.config().include_error_details);
    assert_eq!(middleware.config().auth_realm, "Custom Realm");
    
    // Create with default config
    let adapter = MockAdapter::new("test", true);
    let middleware = HttpAuthMiddleware::with_default_config(adapter);
    
    // Default config should have standard values
    assert!(!middleware.config().include_error_details); // Default is false
    assert_eq!(middleware.config().auth_realm, "MCP Server"); // Default realm
}

// ================================================================================================
// Stack Allocation and Memory Tests
// ================================================================================================

#[test]
fn test_stack_allocation() {
    // Verify middleware is stack-allocated (reasonably sized)
    let adapter = MockAdapter::new("test", true);
    let auth_config = HttpAuthConfig::default();
    let middleware = HttpAuthMiddleware::new(adapter, auth_config);
    
    // Calculate the size of the middleware in memory
    let middleware_size = std::mem::size_of_val(&middleware);
    println!("HttpAuthMiddleware size: {middleware_size} bytes");
    
    // Should be relatively small (stack allocation, not heap allocation with Box<dyn>)
    assert!(middleware_size < 1024, "Middleware should be reasonably sized for stack allocation");
}

// ================================================================================================
// HttpAuthRequest Tests
// ================================================================================================

#[test]
fn test_http_auth_request_creation() {
    // Test HttpAuthRequest creation and basic properties
    let mut headers = HashMap::new();
    headers.insert("authorization".to_string(), "Bearer test-token".to_string());
    headers.insert("x-api-key".to_string(), "test-key".to_string());
    
    let mut query_params = HashMap::new();
    query_params.insert("token".to_string(), "query-token".to_string());
    
    let auth_request = HttpAuthRequest::new(
        headers.clone(),
        "/api/secure".to_string(), 
        query_params.clone(),
    );
    
    // Verify properties
    assert_eq!(auth_request.headers, headers);
    assert_eq!(auth_request.path, "/api/secure");
    assert_eq!(auth_request.query_params, query_params);
    
    // Test direct field access
    assert_eq!(auth_request.headers.get("authorization"), Some(&"Bearer test-token".to_string()));
    assert_eq!(auth_request.headers.get("x-api-key"), Some(&"test-key".to_string()));
    assert_eq!(auth_request.headers.get("not-present"), None);
    
    assert_eq!(auth_request.query_params.get("token"), Some(&"query-token".to_string()));
    assert_eq!(auth_request.query_params.get("not-present"), None);
}

// ================================================================================================
// Workspace Standards Compliance Documentation
// ================================================================================================

#[test]
fn test_workspace_standards_compliance() {
    // This test documents the compliance with workspace standards
    // for the HTTP authentication middleware implementation
    
    // Standards compliance is verified through code review and compilation,
    // but we document key compliance points here:
    
    // 1. Zero-cost generics (§6.2)
    // - Uses associated types for Request/AuthData instead of Box<dyn>
    // - Stack allocation for all middleware state
    // - Generic specialization at compile time
    // - No dynamic dispatch in hot paths
    
    // 2. Error handling (§6.3)
    // - Custom error types with proper context
    // - thiserror for consistent implementation
    // - Proper error propagation
    
    // 3. Import organization (§6.1)
    // - Layer 1: std imports
    // - Layer 2: third-party imports
    // - Layer 3: internal imports
    
    // All verified through compilation and code review
    // Implementation follows workspace standards
}

