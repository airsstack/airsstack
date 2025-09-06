//! OAuth2 HTTP Authentication Adapter
//!
//! This module provides the core OAuth2StrategyAdapter that bridges
//! HTTP requests to OAuth2 authentication strategies.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use super::error::HttpAuthError;
use super::extractor::HttpExtractor;
use super::super::middleware::{HttpAuthRequest as MiddlewareHttpAuthRequest, HttpAuthStrategyAdapter};
use crate::authentication::{
    strategies::oauth2::{OAuth2Request, OAuth2Strategy},
    AuthContext, AuthRequest, AuthenticationStrategy,
};
use crate::oauth2::validator::{JwtValidator, ScopeValidator};

/// Wrapper for OAuth2Request that implements AuthRequest trait
#[derive(Debug)]
struct OAuth2RequestWrapper {
    request: OAuth2Request,
    attributes: HashMap<String, String>,
}

impl OAuth2RequestWrapper {
    fn new(request: OAuth2Request) -> Self {
        // Convert metadata to custom attributes
        let attributes = request.metadata.clone();
        Self {
            request,
            attributes,
        }
    }
}

impl AuthRequest<OAuth2Request> for OAuth2RequestWrapper {
    fn custom_attribute(&self, key: &str) -> Option<String> {
        self.attributes.get(key).cloned()
    }

    fn custom_attributes(&self) -> HashMap<String, String> {
        self.attributes.clone()
    }

    fn inner(&self) -> &OAuth2Request {
        &self.request
    }
}

/// OAuth2 strategy adapter for HTTP transport
///
/// Bridges HTTP requests to OAuth2 authentication strategies by converting
/// HTTP-specific request data into OAuth2Request format and handling responses.
///
/// # Type Parameters
/// * `J` - JWT validation backend type
/// * `S` - Security policy type
///
/// # Authentication Flow
/// 1. Extract bearer token from HTTP Authorization header
/// 2. Create OAuth2Request with token only (method extraction handled by authorization layer)
/// 3. Add HTTP-specific metadata for debugging and logging
/// 4. Delegate to underlying OAuth2 strategy for token validation
/// 5. Convert results back to HTTP-appropriate format
///
/// # OAuth2 Bug Fix
/// This adapter no longer extracts method names from HTTP paths to fix the critical
/// JSON-RPC method extraction bug. Method extraction is now properly handled by the
/// authorization layer using JsonRpcMethodExtractor for JSON-RPC over HTTP requests.
#[derive(Debug, Clone)]
pub struct OAuth2StrategyAdapter<J, S>
where
    J: JwtValidator + Send + Sync + Clone + 'static,
    S: ScopeValidator + Send + Sync + Clone + 'static,
{
    /// Underlying OAuth2 authentication strategy
    strategy: OAuth2Strategy<J, S>,
}

impl<J, S> OAuth2StrategyAdapter<J, S>
where
    J: JwtValidator + Send + Sync + Clone + 'static,
    S: ScopeValidator + Send + Sync + Clone + 'static,
{
    /// Create a new OAuth2 strategy adapter
    ///
    /// # Arguments
    /// * `strategy` - Configured OAuth2 authentication strategy
    ///
    /// # Returns
    /// * New adapter instance ready for HTTP authentication
    pub fn new(strategy: OAuth2Strategy<J, S>) -> Self {
        Self { strategy }
    }

    /// Authenticate HTTP request using OAuth2 strategy
    ///
    /// Extracts bearer token from HTTP request and performs token-only authentication.
    /// Method extraction is now handled by the authorization layer to fix the JSON-RPC bug.
    ///
    /// # Arguments
    /// * `request` - HTTP authentication request data
    ///
    /// # Returns
    /// * AuthContext on successful authentication
    /// * HttpAuthError for HTTP-specific errors
    /// * AuthError for authentication failures
    pub async fn authenticate_http(
        &self,
        request: &HttpAuthRequest,
    ) -> Result<AuthContext<crate::oauth2::context::AuthContext>, HttpAuthError> {
        // Extract bearer token from headers
        let token = HttpExtractor::extract_bearer_token(&request.headers)?;

        // Create OAuth2Request with token only - no method extraction
        // Method extraction should happen in the authorization layer from JSON-RPC payload
        let mut oauth2_request = OAuth2Request::new(token);

        // Add HTTP-specific metadata for debugging/logging
        oauth2_request = oauth2_request.with_metadata("http_path", &request.path);
        oauth2_request = oauth2_request.with_metadata("transport", "http");

        if let Some(client_id) = &request.client_id {
            oauth2_request = oauth2_request.with_metadata("client_id", client_id);
        }

        // Wrap the request to implement AuthRequest trait
        let request_wrapper = OAuth2RequestWrapper::new(oauth2_request);

        // Delegate to OAuth2 strategy for token-only authentication
        self.strategy
            .authenticate(&request_wrapper)
            .await
            .map_err(|e| HttpAuthError::AuthenticationFailed {
                message: format!("OAuth2 authentication failed: {e}"),
            })
    }

    /// Get underlying OAuth2 strategy reference
    ///
    /// Provides access to the wrapped OAuth2 strategy for advanced usage.
    ///
    /// # Returns
    /// * Reference to the underlying OAuth2Strategy
    pub fn strategy(&self) -> &OAuth2Strategy<J, S> {
        &self.strategy
    }
}

/// Implementation of zero-cost generic HttpAuthStrategyAdapter for OAuth2
///
/// This implementation bridges the OAuth2StrategyAdapter to the new generic middleware
/// architecture while maintaining full backward compatibility with existing code.
/// Uses associated types to eliminate dynamic dispatch and achieve zero-cost abstractions.
#[async_trait::async_trait]
impl<J, S> HttpAuthStrategyAdapter for OAuth2StrategyAdapter<J, S>
where
    J: JwtValidator + Send + Sync + Clone + 'static,
    S: ScopeValidator + Send + Sync + Clone + 'static,
{
    /// OAuth2 requests use OAuth2Request from the authentication strategy
    type RequestType = OAuth2Request;
    
    /// OAuth2 authentication data uses the OAuth2 AuthContext
    type AuthData = crate::oauth2::context::AuthContext;

    /// Return the authentication method identifier
    fn auth_method(&self) -> &'static str {
        "oauth2"
    }

    /// Authenticate HTTP request using OAuth2 strategy
    ///
    /// Converts the generic HttpAuthRequest to OAuth2-specific format and
    /// delegates to the existing authenticate_http method. This maintains
    /// full compatibility while providing the new generic interface.
    async fn authenticate_http_request(
        &self,
        request: &MiddlewareHttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
        // Convert generic HttpAuthRequest to OAuth2-specific HttpAuthRequest
        let oauth2_http_request = HttpAuthRequest {
            headers: request.headers.clone(),
            path: request.path.clone(),
            client_id: request.client_id.clone(),
            metadata: request.metadata.clone(),
        };

        // Delegate to existing authenticate_http method
        // The existing method already returns the correct type: AuthContext<oauth2::context::AuthContext>
        self.authenticate_http(&oauth2_http_request).await
    }

    /// OAuth2 adapter does not skip any paths by default
    ///
    /// Path-based skipping is handled by the middleware configuration.
    /// OAuth2 requires authentication for all requests unless explicitly skipped.
    fn should_skip_path(&self, _path: &str) -> bool {
        false
    }
}

/// HTTP authentication request data
///
/// Represents authentication data extracted from HTTP requests for OAuth2 processing.
///
/// # Fields
/// * `headers` - HTTP headers including Authorization
/// * `path` - Request path for method extraction
/// * `client_id` - Optional client identifier
/// * `metadata` - Additional request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAuthRequest {
    /// HTTP headers from the request
    pub headers: HashMap<String, String>,
    /// HTTP request path
    pub path: String,
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
    ///
    /// # Returns
    /// * New HttpAuthRequest with default metadata
    pub fn new(headers: HashMap<String, String>, path: String) -> Self {
        Self {
            headers,
            path,
            client_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create HTTP auth request with client ID
    ///
    /// # Arguments
    /// * `headers` - HTTP headers containing authentication data
    /// * `path` - HTTP request path
    /// * `client_id` - Client identifier
    ///
    /// # Returns
    /// * New HttpAuthRequest with specified client ID
    pub fn with_client_id(
        headers: HashMap<String, String>,
        path: String,
        client_id: String,
    ) -> Self {
        Self {
            headers,
            path,
            client_id: Some(client_id),
            metadata: HashMap::new(),
        }
    }

    /// Create HTTP auth request with metadata
    ///
    /// # Arguments
    /// * `headers` - HTTP headers containing authentication data
    /// * `path` - HTTP request path
    /// * `metadata` - Request metadata
    ///
    /// # Returns
    /// * New HttpAuthRequest with specified metadata
    pub fn with_metadata(
        headers: HashMap<String, String>,
        path: String,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            headers,
            path,
            client_id: None,
            metadata,
        }
    }

    /// Add client ID to existing request
    ///
    /// # Arguments
    /// * `client_id` - Client identifier to add
    ///
    /// # Returns
    /// * Self for method chaining
    pub fn client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Add metadata to existing request
    ///
    /// # Arguments
    /// * `metadata` - Metadata to add
    ///
    /// # Returns
    /// * Self for method chaining
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

impl fmt::Display for HttpAuthRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HttpAuthRequest {{ path: {}, client_id: {:?}, headers: {} }}",
            self.path,
            self.client_id,
            self.headers.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::strategies::oauth2::OAuth2Strategy;
    use crate::oauth2::error::OAuth2Error;
    use crate::oauth2::types::JwtClaims;
    use crate::oauth2::validator::{jwt::JwtValidator, scope::ScopeValidator, Validator};
    use async_trait::async_trait;

    // Mock types for testing
    #[derive(Debug, Clone)]
    struct MockValidator;

    #[derive(Debug, Clone)]
    struct MockSecurityPolicy;

    // Mock error for testing
    #[derive(Debug, thiserror::Error)]
    #[error("Mock validation error")]
    struct MockError;

    impl From<MockError> for OAuth2Error {
        fn from(_val: MockError) -> Self {
            OAuth2Error::InvalidToken("Mock validation failed".to_string())
        }
    }

    #[async_trait]
    impl JwtValidator for MockValidator {
        type Error = MockError;

        async fn validate(&self, _token: &str) -> Result<JwtClaims, Self::Error> {
            Ok(JwtClaims {
                sub: "test_user".to_string(),
                aud: Some("test_audience".to_string()),
                iss: Some("test_issuer".to_string()),
                exp: Some(9999999999i64),
                nbf: None,
                iat: Some(1630000000i64),
                jti: Some("test_jti".to_string()),
                scope: Some("read write".to_string()),
                scopes: Some(vec!["read".to_string(), "write".to_string()]),
            })
        }
    }

    impl ScopeValidator for MockSecurityPolicy {
        type Error = MockError;

        fn validate_method_access(
            &self,
            _method: &str,
            _scopes: &[String],
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn test_http_auth_request_creation() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let request = HttpAuthRequest::new(headers.clone(), "/mcp/tools/call".to_string());

        assert_eq!(request.path, "/mcp/tools/call");
        assert_eq!(request.headers, headers);
        assert_eq!(request.client_id, None);
    }

    #[test]
    fn test_http_auth_request_with_client_id() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let request = HttpAuthRequest::with_client_id(
            headers.clone(),
            "/mcp/tools/call".to_string(),
            "test_client".to_string(),
        );

        assert_eq!(request.path, "/mcp/tools/call");
        assert_eq!(request.headers, headers);
        assert_eq!(request.client_id, Some("test_client".to_string()));
    }

    #[test]
    fn test_oauth2_strategy_adapter_creation() {
        let jwt = MockValidator;
        let scope = MockSecurityPolicy;
        let validator = Validator::new(jwt, scope);
        let strategy = OAuth2Strategy::new(validator);
        let _adapter = OAuth2StrategyAdapter::new(strategy);

        // Test passes if creation succeeds without panic
    }

    #[test]
    fn test_http_auth_request_display() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let request = HttpAuthRequest::new(headers, "/mcp/tools/call".to_string())
            .client_id("test_client".to_string());

        let display_str = format!("{request}");
        assert!(display_str.contains("/mcp/tools/call"));
        assert!(display_str.contains("test_client"));
        assert!(display_str.contains("headers: 1"));
    }
}
