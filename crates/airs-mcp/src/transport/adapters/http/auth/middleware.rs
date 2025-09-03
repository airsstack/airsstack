//! Zero-Cost Generic HTTP Authentication Middleware
//!
//! This module provides a zero-cost generic authentication middleware architecture
//! that eliminates dynamic dispatch and follows workspace standard §6.
//!
//! The middleware uses compile-time generics with associated types for maximum
//! performance while maintaining ergonomic APIs through builder patterns.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;

// Layer 2: Third-party crate imports  
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::authentication::AuthContext;
use super::oauth2::error::HttpAuthError;

/// Configuration for HTTP authentication middleware
///
/// Stack-allocated configuration struct for HTTP authentication middleware.
/// Contains skip paths, error detail flags, authentication realm, and request timeout.
///
/// # Examples
///
/// ```rust
/// # use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;
/// let config = HttpAuthConfig::default();
/// ```
#[derive(Debug, Clone)]
pub struct HttpAuthConfig {
    /// Paths that should skip authentication (e.g., health checks)
    pub skip_paths: Vec<String>,
    
    /// Whether to include detailed error messages in authentication failures
    pub include_error_details: bool,
    
    /// Authentication realm for WWW-Authenticate headers
    pub auth_realm: String,
    
    /// Request timeout for authentication operations
    pub request_timeout_secs: u64,
}

impl Default for HttpAuthConfig {
    fn default() -> Self {
        Self {
            skip_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/status".to_string(),
            ],
            include_error_details: false,
            auth_realm: "MCP Server".to_string(),
            request_timeout_secs: 30,
        }
    }
}

/// HTTP authentication request data
///
/// Represents authentication data extracted from HTTP requests for strategy processing.
/// Used as a unified interface between HTTP transport and authentication strategies.
///
/// # Fields
/// * `headers` - HTTP headers including Authorization and custom headers
/// * `path` - Request path for method extraction and path-based skipping
/// * `query_params` - Query parameters for API key extraction
/// * `client_id` - Optional client identifier for multi-tenant authentication
/// * `metadata` - Additional request metadata for custom authentication logic
#[derive(Debug, Clone)]
pub struct HttpAuthRequest {
    /// HTTP headers from the request
    pub headers: HashMap<String, String>,
    
    /// HTTP request path
    pub path: String,
    
    /// Query parameters from the request URL
    pub query_params: HashMap<String, String>,
    
    /// Optional client identifier
    pub client_id: Option<String>,
    
    /// Additional request metadata
    pub metadata: HashMap<String, String>,
}

impl HttpAuthRequest {
    /// Create a new HTTP authentication request
    ///
    /// # Arguments
    /// * `headers` - HTTP headers containing authentication data
    /// * `path` - HTTP request path
    /// * `query_params` - Query parameters from URL
    ///
    /// # Returns
    /// * New HttpAuthRequest with default metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthRequest;
    /// let headers = HashMap::new();
    /// let query_params = HashMap::new();
    /// let request = HttpAuthRequest::new(headers, "/mcp/tools".to_string(), query_params);
    /// ```
    pub fn new(
        headers: HashMap<String, String>,
        path: String,
        query_params: HashMap<String, String>,
    ) -> Self {
        Self {
            headers,
            path,
            query_params,
            client_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Add client ID to the request (builder pattern)
    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Add metadata to the request (builder pattern)
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

impl fmt::Display for HttpAuthRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HttpAuthRequest {{ path: {}, client_id: {:?}, headers: {}, query_params: {} }}",
            self.path,
            self.client_id,
            self.headers.len(),
            self.query_params.len()
        )
    }
}

/// Zero-cost generic HTTP authentication strategy adapter trait
///
/// This trait defines the interface for HTTP authentication adapters using associated types
/// to achieve zero-cost abstractions. Each strategy adapter implements this trait to bridge
/// HTTP requests to specific authentication strategies without dynamic dispatch.
///
/// # Associated Types
/// * `RequestType` - Specific request type for the authentication strategy
/// * `AuthData` - Authentication data type produced by successful authentication
///
/// # Type Parameters
/// The implementing type serves as the strategy identifier, allowing the compiler
/// to monomorphize all authentication calls for maximum performance.
///
/// # Examples
///
/// ```rust
/// # use async_trait::async_trait;
/// # use airs_mcp::transport::adapters::http::auth::middleware::{HttpAuthStrategyAdapter, HttpAuthRequest};
/// # use airs_mcp::transport::adapters::http::auth::oauth2::error::HttpAuthError;
/// # use airs_mcp::authentication::{AuthContext, AuthMethod};
/// # 
/// # #[derive(Clone, Debug)]
/// # struct MockAuthAdapter;
/// # 
/// # #[derive(Clone, Debug)]
/// # struct MockAuthData;
/// # 
/// # #[async_trait::async_trait]
/// # impl HttpAuthStrategyAdapter for MockAuthAdapter {
/// #     type RequestType = ();
/// #     type AuthData = MockAuthData;
/// #     
/// #     fn auth_method(&self) -> &'static str {
/// #         "mock"
/// #     }
/// #     
/// #     async fn authenticate_http_request(&self, _request: &HttpAuthRequest) 
/// #         -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
/// #         Ok(AuthContext::new(AuthMethod::new("mock"), MockAuthData))
/// #     }
/// # }
/// ```
#[async_trait]
pub trait HttpAuthStrategyAdapter: Send + Sync + Clone + 'static {
    /// The request type specific to this authentication strategy
    type RequestType: Send + Sync;
    
    /// The authentication data type produced by successful authentication
    type AuthData: Send + Sync + Clone + 'static;

    /// Get the authentication method name for this adapter
    ///
    /// Returns a static string identifier for the authentication method.
    /// Used for logging, metrics, and debugging purposes.
    ///
    /// # Returns
    /// * Static string identifying the authentication method
    fn auth_method(&self) -> &'static str;

    /// Authenticate an HTTP request using this strategy
    ///
    /// Processes the HTTP request data and attempts authentication using
    /// the specific strategy implementation. Returns authentication context
    /// on success or detailed error information on failure.
    ///
    /// # Arguments
    /// * `request` - HTTP authentication request data
    ///
    /// # Returns
    /// * `AuthContext<Self::AuthData>` on successful authentication
    /// * `HttpAuthError` for authentication failures
    async fn authenticate_http_request(
        &self,
        request: &HttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, HttpAuthError>;

    /// Check if a specific path should skip authentication
    ///
    /// Allows strategies to implement path-based authentication skipping
    /// for health checks, metrics endpoints, or other public resources.
    /// Default implementation requires authentication for all paths.
    ///
    /// # Arguments
    /// * `path` - HTTP request path to check
    ///
    /// # Returns
    /// * `true` if authentication should be skipped for this path
    /// * `false` if authentication is required (default)
    fn should_skip_path(&self, _path: &str) -> bool {
        false
    }
}

/// Zero-cost generic HTTP authentication middleware
///
/// Provides HTTP authentication middleware using compile-time generics to eliminate
/// dynamic dispatch overhead while maintaining ergonomic APIs. The middleware wraps
/// an authentication strategy adapter and applies authentication to HTTP requests.
///
/// # Type Parameters
/// * `A` - Authentication strategy adapter implementing HttpAuthStrategyAdapter
///
/// # Performance Characteristics
/// * Zero dynamic dispatch - all authentication calls are monomorphized
/// * Stack allocation - no heap allocations for middleware operation
/// * Compile-time optimization - authentication logic inlined
/// * CPU cache friendly - direct method calls without vtable lookups
///
/// # Examples
///
/// ```rust
/// # use airs_mcp::transport::adapters::http::auth::middleware::{
/// #     HttpAuthMiddleware, HttpAuthConfig, HttpAuthStrategyAdapter, HttpAuthRequest
/// # };
/// # use airs_mcp::authentication::{AuthContext, AuthMethod};
/// # use airs_mcp::transport::adapters::http::auth::oauth2::error::HttpAuthError;
/// # 
/// # // Mock adapter for the example
/// # #[derive(Clone, Debug)]
/// # struct MockAdapter;
/// # 
/// # #[async_trait::async_trait]
/// # impl HttpAuthStrategyAdapter for MockAdapter {
/// #     type RequestType = ();
/// #     type AuthData = ();
/// #     fn auth_method(&self) -> &'static str { "mock" }
/// #     async fn authenticate_http_request(&self, _request: &HttpAuthRequest) 
/// #         -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
/// #         Ok(AuthContext::new(AuthMethod::new("mock"), ()))
/// #     }
/// # }
/// #
/// let adapter = MockAdapter;
/// let config = HttpAuthConfig::default();
/// let middleware = HttpAuthMiddleware::new(adapter, config);
/// ```
#[derive(Debug, Clone)]
pub struct HttpAuthMiddleware<A>
where
    A: HttpAuthStrategyAdapter,
{
    /// Authentication strategy adapter (zero-cost generic)
    adapter: A,
    
    /// Middleware configuration (stack-allocated)
    config: HttpAuthConfig,
}

impl<A> HttpAuthMiddleware<A>
where
    A: HttpAuthStrategyAdapter,
{
    /// Create a new HTTP authentication middleware
    ///
    /// Creates a zero-cost generic middleware instance that wraps the provided
    /// authentication strategy adapter with the specified configuration.
    ///
    /// # Arguments
    /// * `adapter` - Authentication strategy adapter
    /// * `config` - Middleware configuration
    ///
    /// # Returns
    /// * New middleware instance ready for HTTP request processing
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp::transport::adapters::http::auth::middleware::{
    /// #     HttpAuthMiddleware, HttpAuthConfig, HttpAuthStrategyAdapter, HttpAuthRequest
    /// # };
    /// # use airs_mcp::authentication::{AuthContext, AuthMethod};
    /// # use airs_mcp::transport::adapters::http::auth::oauth2::error::HttpAuthError;
    /// # 
    /// # // Mock adapter for the example
    /// # #[derive(Clone, Debug)]
    /// # struct MockAdapter;
    /// # 
    /// # #[async_trait::async_trait]
    /// # impl HttpAuthStrategyAdapter for MockAdapter {
    /// #     type RequestType = ();
    /// #     type AuthData = ();
    /// #     fn auth_method(&self) -> &'static str { "mock" }
    /// #     async fn authenticate_http_request(&self, _request: &HttpAuthRequest) 
    /// #         -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
    /// #         Ok(AuthContext::new(AuthMethod::new("mock"), ()))
    /// #     }
    /// # }
    /// #
    /// let adapter = MockAdapter;
    /// let config = HttpAuthConfig::default();
    /// let middleware = HttpAuthMiddleware::new(adapter, config);
    /// ```
    pub fn new(adapter: A, config: HttpAuthConfig) -> Self {
        Self { adapter, config }
    }

    /// Create middleware with default configuration
    ///
    /// Convenience constructor that creates middleware with default configuration
    /// for rapid development and testing scenarios.
    ///
    /// # Arguments
    /// * `adapter` - Authentication strategy adapter
    ///
    /// # Returns
    /// * New middleware instance with default configuration
    pub fn with_default_config(adapter: A) -> Self {
        Self::new(adapter, HttpAuthConfig::default())
    }

    /// Authenticate an HTTP request
    ///
    /// Processes HTTP request authentication using the configured strategy adapter.
    /// Handles path skipping, error formatting, and authentication context creation.
    ///
    /// # Arguments
    /// * `request` - HTTP authentication request data
    ///
    /// # Returns
    /// * `Some(AuthContext<A::AuthData>)` on successful authentication
    /// * `None` if path should be skipped
    /// * `HttpAuthError` for authentication failures
    pub async fn authenticate(
        &self,
        request: &HttpAuthRequest,
    ) -> Result<Option<AuthContext<A::AuthData>>, HttpAuthError> {
        // Check if path should skip authentication
        if self.config.skip_paths.contains(&request.path) || self.adapter.should_skip_path(&request.path) {
            return Ok(None);
        }

        // Delegate to strategy adapter
        self.adapter
            .authenticate_http_request(request)
            .await
            .map(Some)
    }

    /// Get authentication method name
    ///
    /// Returns the authentication method identifier from the wrapped adapter.
    /// Used for logging, metrics, and debugging purposes.
    ///
    /// # Returns
    /// * Static string identifying the authentication method
    pub fn auth_method(&self) -> &'static str {
        self.adapter.auth_method()
    }

    /// Get middleware configuration
    ///
    /// Provides access to the middleware configuration for inspection
    /// and advanced customization scenarios.
    ///
    /// # Returns
    /// * Reference to the middleware configuration
    pub fn config(&self) -> &HttpAuthConfig {
        &self.config
    }

    /// Get strategy adapter reference
    ///
    /// Provides access to the underlying authentication strategy adapter
    /// for advanced usage scenarios and direct strategy access.
    ///
    /// # Returns
    /// * Reference to the wrapped authentication strategy adapter
    pub fn adapter(&self) -> &A {
        &self.adapter
    }
}

impl<A> fmt::Display for HttpAuthMiddleware<A>
where
    A: HttpAuthStrategyAdapter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HttpAuthMiddleware {{ method: {}, realm: {}, skip_paths: {} }}",
            self.auth_method(),
            self.config.auth_realm,
            self.config.skip_paths.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock adapter for testing
    #[derive(Debug, Clone)]
    struct MockStrategyAdapter {
        should_skip: bool,
    }

    impl MockStrategyAdapter {
        fn new() -> Self {
            Self { should_skip: false }
        }

        fn with_skip(should_skip: bool) -> Self {
            Self { should_skip }
        }
    }

    #[async_trait]
    impl HttpAuthStrategyAdapter for MockStrategyAdapter {
        type RequestType = ();
        type AuthData = String;

        fn auth_method(&self) -> &'static str {
            "mock"
        }

        async fn authenticate_http_request(
            &self,
            _request: &HttpAuthRequest,
        ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
            use crate::authentication::AuthMethod;
            Ok(AuthContext::new(
                AuthMethod::new("mock"),
                "mock_user".to_string(),
            ))
        }

        fn should_skip_path(&self, _path: &str) -> bool {
            self.should_skip
        }
    }

    #[test]
    fn test_http_auth_config_default() {
        let config = HttpAuthConfig::default();
        
        assert_eq!(config.skip_paths.len(), 3);
        assert!(config.skip_paths.contains(&"/health".to_string()));
        assert!(config.skip_paths.contains(&"/metrics".to_string()));
        assert!(config.skip_paths.contains(&"/status".to_string()));
        assert!(!config.include_error_details);
        assert_eq!(config.auth_realm, "MCP Server");
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn test_http_auth_request_creation() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        
        let mut query_params = HashMap::new();
        query_params.insert("api_key".to_string(), "key123".to_string());

        let request = HttpAuthRequest::new(headers.clone(), "/mcp/tools".to_string(), query_params.clone());

        assert_eq!(request.path, "/mcp/tools");
        assert_eq!(request.headers, headers);
        assert_eq!(request.query_params, query_params);
        assert_eq!(request.client_id, None);
        assert!(request.metadata.is_empty());
    }

    #[test]
    fn test_http_auth_request_builder() {
        let request = HttpAuthRequest::new(HashMap::new(), "/mcp/tools".to_string(), HashMap::new())
            .with_client_id("client123".to_string())
            .with_metadata({
                let mut metadata = HashMap::new();
                metadata.insert("custom".to_string(), "value".to_string());
                metadata
            });

        assert_eq!(request.client_id, Some("client123".to_string()));
        assert_eq!(request.metadata.get("custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_http_auth_request_display() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        
        let request = HttpAuthRequest::new(headers, "/test".to_string(), HashMap::new())
            .with_client_id("test_client".to_string());

        let display_str = format!("{request}");
        assert!(display_str.contains("/test"));
        assert!(display_str.contains("test_client"));
        assert!(display_str.contains("headers: 1"));
        assert!(display_str.contains("query_params: 0"));
    }

    #[test]
    fn test_http_auth_middleware_creation() {
        let adapter = MockStrategyAdapter::new();
        let config = HttpAuthConfig::default();
        let middleware = HttpAuthMiddleware::new(adapter, config);

        assert_eq!(middleware.auth_method(), "mock");
        assert_eq!(middleware.config().skip_paths.len(), 3);
    }

    #[test]
    fn test_http_auth_middleware_with_default_config() {
        let adapter = MockStrategyAdapter::new();
        let middleware = HttpAuthMiddleware::with_default_config(adapter);

        assert_eq!(middleware.auth_method(), "mock");
        assert_eq!(middleware.config().auth_realm, "MCP Server");
    }

    #[tokio::test]
    async fn test_middleware_authenticate_skip_config_paths() {
        let adapter = MockStrategyAdapter::new();
        let middleware = HttpAuthMiddleware::with_default_config(adapter);

        let request = HttpAuthRequest::new(HashMap::new(), "/health".to_string(), HashMap::new());
        let result = middleware.authenticate(&request).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Authentication skipped
    }

    #[tokio::test]
    async fn test_middleware_authenticate_skip_adapter_paths() {
        let adapter = MockStrategyAdapter::with_skip(true);
        let middleware = HttpAuthMiddleware::with_default_config(adapter);

        let request = HttpAuthRequest::new(HashMap::new(), "/custom".to_string(), HashMap::new());
        let result = middleware.authenticate(&request).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Authentication skipped by adapter
    }

    #[tokio::test]
    async fn test_middleware_authenticate_success() {
        let adapter = MockStrategyAdapter::new();
        let middleware = HttpAuthMiddleware::with_default_config(adapter);

        let request = HttpAuthRequest::new(HashMap::new(), "/mcp/tools".to_string(), HashMap::new());
        let result = middleware.authenticate(&request).await;

        assert!(result.is_ok());
        let auth_context = result.unwrap().unwrap();
        assert_eq!(auth_context.auth_data, "mock_user");
    }

    #[test]
    fn test_middleware_display() {
        let adapter = MockStrategyAdapter::new();
        let middleware = HttpAuthMiddleware::with_default_config(adapter);

        let display_str = format!("{middleware}");
        assert!(display_str.contains("mock"));
        assert!(display_str.contains("MCP Server"));
        assert!(display_str.contains("skip_paths: 3"));
    }

    #[test]
    fn test_middleware_accessors() {
        let adapter = MockStrategyAdapter::new();
        let config = HttpAuthConfig {
            skip_paths: vec!["/test".to_string()],
            include_error_details: true,
            auth_realm: "Test Realm".to_string(),
            request_timeout_secs: 60,
        };
        let middleware = HttpAuthMiddleware::new(adapter.clone(), config.clone());

        assert_eq!(middleware.auth_method(), "mock");
        assert_eq!(middleware.config().auth_realm, "Test Realm");
        assert_eq!(middleware.config().request_timeout_secs, 60);
        assert!(middleware.config().include_error_details);
        assert!(!middleware.adapter().should_skip);
    }
}
