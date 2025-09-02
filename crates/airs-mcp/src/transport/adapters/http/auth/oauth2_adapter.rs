//! OAuth2 Strategy HTTP Adapter
//!
//! HTTP transport adapter for OAuth2 authentication strategy, providing seamless integration
//! between the generic OAuth2Strategy and HTTP-specific authentication requirements.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::authentication::{
    context::AuthContext,
    error::{AuthError, AuthResult},
    method::AuthMethod,
    request::AuthRequest,
    strategies::oauth2::{OAuth2AuthRequest, OAuth2Request, OAuth2Strategy},
    strategy::AuthenticationStrategy,
};
use crate::oauth2::{
    context::AuthContext as OAuth2Context,
    validator::{JwtValidator, ScopeValidator},
};
use crate::transport::adapters::http::auth_request::HttpAuthRequest;
use crate::transport::adapters::http::engine::HttpEngineError;

/// HTTP-specific error type for OAuth2 authentication
#[derive(Debug, thiserror::Error)]
pub enum HttpAuthError {
    /// Authentication failed
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    /// Invalid HTTP request format
    #[error("Invalid HTTP request: {message}")]
    InvalidRequest { message: String },

    /// Missing required HTTP headers
    #[error("Missing required header: {header}")]
    MissingHeader { header: String },

    /// Malformed authorization header
    #[error("Malformed authorization header: {message}")]
    MalformedAuth { message: String },

    /// HTTP engine error
    #[error("HTTP engine error: {0}")]
    EngineError(#[from] HttpEngineError),

    /// Generic authentication error
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
}

/// OAuth2 strategy adapter for HTTP transport
///
/// Provides HTTP-specific integration for OAuth2 authentication strategy,
/// handling HTTP request extraction, header parsing, and error conversion.
///
/// # Type Parameters
/// * `J` - JWT validator implementation (must implement JwtValidator)
/// * `S` - Scope validator implementation (must implement ScopeValidator)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::auth::OAuth2StrategyAdapter;
/// use airs_mcp::authentication::strategies::oauth2::OAuth2Strategy;
/// use airs_mcp::oauth2::validator::{Validator, Jwt, Scope};
/// use airs_mcp::oauth2::config::OAuth2Config;
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OAuth2Config::default();
///
/// // Create OAuth2 strategy
/// let jwt = Jwt::new(config)?;
/// let scope = Scope::with_default_mappings();
/// let validator = Validator::new(jwt, scope);
/// let oauth2_strategy = OAuth2Strategy::new(validator);
///
/// // Create HTTP adapter
/// let adapter = OAuth2StrategyAdapter::new(oauth2_strategy);
///
/// // Use adapter with HTTP request
/// let mut headers = HashMap::new();
/// headers.insert("Authorization".to_string(), "Bearer eyJ...".to_string());
///
/// let http_request = HttpAuthRequest::new(
///     "POST".to_string(),
///     "/mcp/tools/call".to_string(),
///     headers,
///     HashMap::new(),
///     None,
///     Some("192.168.1.1".to_string()),
///     HashMap::new(),
/// );
///
/// // let auth_context = adapter.authenticate_http(&http_request).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct OAuth2StrategyAdapter<J, S>
where
    J: JwtValidator + Clone + Send + Sync + 'static,
    S: ScopeValidator + Clone + Send + Sync + 'static,
{
    /// Underlying OAuth2 strategy
    strategy: OAuth2Strategy<J, S>,
}

impl<J, S> OAuth2StrategyAdapter<J, S>
where
    J: JwtValidator + Clone + Send + Sync + 'static,
    S: ScopeValidator + Clone + Send + Sync + 'static,
{
    /// Create new OAuth2 strategy adapter
    ///
    /// # Arguments
    /// * `strategy` - OAuth2 strategy instance to wrap
    ///
    /// # Returns
    /// * New OAuth2StrategyAdapter ready for HTTP authentication
    pub fn new(strategy: OAuth2Strategy<J, S>) -> Self {
        Self { strategy }
    }

    /// Get reference to underlying OAuth2 strategy
    ///
    /// Provides access to the OAuth2 strategy for advanced scenarios.
    /// Use sparingly to maintain abstraction boundaries.
    pub fn strategy(&self) -> &OAuth2Strategy<J, S> {
        &self.strategy
    }

    /// Extract bearer token from HTTP Authorization header
    ///
    /// Supports standard "Bearer <token>" format with case-insensitive header matching.
    ///
    /// # Arguments
    /// * `headers` - HTTP headers from the request
    ///
    /// # Returns
    /// * Bearer token if found and valid format
    /// * HttpAuthError if missing, malformed, or invalid
    pub fn extract_bearer_token(
        &self,
        headers: &HashMap<String, String>,
    ) -> Result<String, HttpAuthError> {
        // Try different case variations of Authorization header
        let auth_header = headers
            .get("Authorization")
            .or_else(|| headers.get("authorization"))
            .or_else(|| headers.get("AUTHORIZATION"))
            .ok_or_else(|| HttpAuthError::MissingHeader {
                header: "Authorization".to_string(),
            })?;

        // Parse Bearer token format
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if token.trim().is_empty() {
                return Err(HttpAuthError::MalformedAuth {
                    message: "Empty bearer token".to_string(),
                });
            }
            Ok(token.trim().to_string())
        } else if let Some(token) = auth_header.strip_prefix("bearer ") {
            // Handle lowercase bearer prefix
            if token.trim().is_empty() {
                return Err(HttpAuthError::MalformedAuth {
                    message: "Empty bearer token".to_string(),
                });
            }
            Ok(token.trim().to_string())
        } else {
            Err(HttpAuthError::MalformedAuth {
                message: format!("Expected 'Bearer <token>' format, got: {auth_header}"),
            })
        }
    }

    /// Extract method name from HTTP request
    ///
    /// Uses path-based method extraction for MCP protocol compliance.
    /// Expects paths like "/mcp/tools/call" or "/api/v1/resources/list".
    ///
    /// # Arguments
    /// * `path` - HTTP request path
    ///
    /// # Returns
    /// * Method name extracted from path, or None if not extractable
    pub fn extract_method(&self, path: &str) -> Option<String> {
        // Handle MCP-style paths: /mcp/tools/call -> tools/call
        if let Some(mcp_path) = path.strip_prefix("/mcp/") {
            return Some(mcp_path.to_string());
        }

        // Handle API-style paths: /api/v1/tools/call -> tools/call
        if let Some(api_path) = path.strip_prefix("/api/v1/") {
            return Some(api_path.to_string());
        }

        // Handle root-level paths: /tools/call -> tools/call
        if let Some(root_path) = path.strip_prefix('/') {
            if !root_path.is_empty() {
                return Some(root_path.to_string());
            }
        }

        None
    }

    /// Convert HTTP request to OAuth2 request
    ///
    /// Extracts bearer token, method, and metadata from HTTP request,
    /// creating an OAuth2Request suitable for authentication.
    ///
    /// # Arguments
    /// * `http_request` - HTTP authentication request
    ///
    /// # Returns
    /// * OAuth2Request with extracted data
    /// * HttpAuthError if token extraction fails
    pub fn convert_http_request(
        &self,
        http_request: &HttpAuthRequest,
    ) -> Result<OAuth2Request, HttpAuthError> {
        // Extract bearer token from Authorization header
        let bearer_token = self.extract_bearer_token(&http_request.headers)?;

        // Extract method from request path
        let method = self.extract_method(&http_request.path);

        // Build OAuth2 request with metadata
        let mut oauth2_request = OAuth2Request::new(bearer_token);

        if let Some(method) = method {
            oauth2_request = oauth2_request.with_method(method);
        }

        // Add HTTP-specific metadata
        oauth2_request = oauth2_request
            .with_metadata("http_method", &http_request.method)
            .with_metadata("http_path", &http_request.path);

        if let Some(client_ip) = &http_request.client_ip {
            oauth2_request = oauth2_request.with_metadata("client_ip", client_ip);
        }

        // Add query parameters as metadata
        for (key, value) in &http_request.query_params {
            oauth2_request = oauth2_request.with_metadata(format!("query:{key}"), value);
        }

        // Add custom attributes as metadata
        for (key, value) in &http_request.custom_attributes {
            oauth2_request = oauth2_request.with_metadata(format!("attr:{key}"), value);
        }

        Ok(oauth2_request)
    }

    /// Authenticate HTTP request using OAuth2 strategy
    ///
    /// Primary method for HTTP OAuth2 authentication. Converts HTTP request
    /// to OAuth2 format, performs authentication, and returns auth context.
    ///
    /// # Arguments
    /// * `http_request` - HTTP authentication request to authenticate
    ///
    /// # Returns
    /// * AuthContext with OAuth2 authentication data
    /// * HttpAuthError if authentication fails
    pub async fn authenticate_http(
        &self,
        http_request: &HttpAuthRequest,
    ) -> Result<AuthContext<OAuth2Context>, HttpAuthError> {
        // Convert HTTP request to OAuth2 request
        let oauth2_request = self.convert_http_request(http_request)?;

        // Create OAuth2 auth request wrapper
        let auth_request = OAuth2AuthRequest::new(oauth2_request);

        // Authenticate using OAuth2 strategy
        let auth_context = self
            .strategy
            .authenticate(&auth_request)
            .await
            .map_err(|e| HttpAuthError::AuthenticationFailed {
                message: format!("OAuth2 authentication failed: {e}"),
            })?;

        Ok(auth_context)
    }

    /// Validate existing authentication context
    ///
    /// Validates that an existing auth context is still valid.
    /// Useful for session management and token refresh scenarios.
    ///
    /// # Arguments
    /// * `context` - Authentication context to validate
    ///
    /// # Returns
    /// * true if context is valid, false otherwise
    /// * HttpAuthError if validation fails
    pub async fn validate_context(
        &self,
        context: &AuthContext<OAuth2Context>,
    ) -> Result<bool, HttpAuthError> {
        self.strategy
            .validate(context)
            .await
            .map_err(HttpAuthError::AuthError)
    }
}

/// Implement AuthenticationStrategy for HTTP adapter
///
/// Provides standard authentication strategy interface while handling
/// HTTP-specific request conversion internally.
#[async_trait]
impl<J, S> AuthenticationStrategy<HttpAuthRequest, OAuth2Context> for OAuth2StrategyAdapter<J, S>
where
    J: JwtValidator + Send + Sync + Clone + 'static,
    S: ScopeValidator + Send + Sync + Clone + 'static,
{
    fn method(&self) -> AuthMethod {
        self.strategy.method()
    }

    async fn authenticate(
        &self,
        request: &impl AuthRequest<HttpAuthRequest>,
    ) -> AuthResult<AuthContext<OAuth2Context>> {
        let http_request = request.inner();

        self.authenticate_http(http_request)
            .await
            .map_err(|e| match e {
                HttpAuthError::AuthError(auth_error) => auth_error,
                HttpAuthError::AuthenticationFailed { message } => {
                    AuthError::InvalidCredentials(message)
                }
                HttpAuthError::InvalidRequest { message } => {
                    AuthError::InvalidCredentials(format!("Invalid HTTP request: {message}"))
                }
                HttpAuthError::MissingHeader { header } => {
                    AuthError::MissingCredentials(format!("Missing HTTP header: {header}"))
                }
                HttpAuthError::MalformedAuth { message } => {
                    AuthError::InvalidCredentials(format!("Malformed authorization: {message}"))
                }
                HttpAuthError::EngineError(engine_error) => {
                    AuthError::Internal(format!("HTTP engine error: {engine_error}"))
                }
            })
    }

    async fn validate(&self, context: &AuthContext<OAuth2Context>) -> AuthResult<bool> {
        self.validate_context(context).await.map_err(|e| match e {
            HttpAuthError::AuthError(auth_error) => auth_error,
            _ => AuthError::Internal(format!("HTTP validation error: {e}")),
        })
    }
}

// Conversion from HttpAuthError to AuthError for error interoperability
impl From<HttpAuthError> for AuthError {
    fn from(error: HttpAuthError) -> Self {
        match error {
            HttpAuthError::AuthenticationFailed { message } => {
                AuthError::InvalidCredentials(message)
            }
            HttpAuthError::InvalidRequest { message } => {
                AuthError::InvalidCredentials(format!("Invalid HTTP request: {message}"))
            }
            HttpAuthError::MissingHeader { header } => {
                AuthError::MissingCredentials(format!("Missing HTTP header: {header}"))
            }
            HttpAuthError::MalformedAuth { message } => {
                AuthError::InvalidCredentials(format!("Malformed authorization: {message}"))
            }
            HttpAuthError::EngineError(engine_error) => {
                AuthError::Internal(format!("HTTP engine error: {engine_error}"))
            }
            HttpAuthError::AuthError(auth_error) => auth_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::{
        error::OAuth2Error,
        types::JwtClaims,
        validator::{JwtValidator, ScopeValidator, Validator},
    };

    // Mock implementations for testing
    #[derive(Clone)]
    struct MockJwtValidator {
        should_fail: bool,
    }

    // Safety: MockJwtValidator is safe to send and sync as it only contains simple bool
    unsafe impl Send for MockJwtValidator {}
    unsafe impl Sync for MockJwtValidator {}

    #[async_trait]
    impl JwtValidator for MockJwtValidator {
        type Error = OAuth2Error;

        async fn validate(&self, _token: &str) -> Result<JwtClaims, Self::Error> {
            if self.should_fail {
                Err(OAuth2Error::InvalidToken(
                    "Mock JWT validation failure".to_string(),
                ))
            } else {
                Ok(JwtClaims {
                    sub: "test_user_123".to_string(),
                    scope: Some("mcp:tools:execute mcp:resources:read".to_string()),
                    scopes: None,
                    aud: Some("test_audience".to_string()),
                    iss: Some("https://auth.example.com".to_string()),
                    exp: Some(chrono::Utc::now().timestamp() + 3600),
                    nbf: None,
                    iat: Some(chrono::Utc::now().timestamp()),
                    jti: Some("test_jwt_id".to_string()),
                })
            }
        }

        fn extract_scopes(&self, claims: &JwtClaims) -> Vec<String> {
            claims
                .scope
                .as_deref()
                .unwrap_or("")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        }
    }

    #[derive(Clone)]
    struct MockScopeValidator {
        should_fail: bool,
    }

    // Safety: MockScopeValidator is safe to send and sync as it only contains simple bool
    unsafe impl Send for MockScopeValidator {}
    unsafe impl Sync for MockScopeValidator {}

    impl ScopeValidator for MockScopeValidator {
        type Error = OAuth2Error;

        fn validate_method_access(
            &self,
            method: &str,
            _scopes: &[String],
        ) -> Result<(), Self::Error> {
            if self.should_fail || method == "forbidden/method" {
                Err(OAuth2Error::InsufficientScope {
                    required: "admin:access".to_string(),
                    provided: "user:basic".to_string(),
                })
            } else {
                Ok(())
            }
        }

        fn is_method_configured(&self, _method: &str) -> bool {
            true
        }

        fn get_required_scope(&self, _method: &str) -> Option<&str> {
            Some("mcp:tools:execute")
        }
    }

    fn create_test_adapter(
        jwt_fail: bool,
        scope_fail: bool,
    ) -> OAuth2StrategyAdapter<MockJwtValidator, MockScopeValidator> {
        let jwt = MockJwtValidator {
            should_fail: jwt_fail,
        };
        let scope = MockScopeValidator {
            should_fail: scope_fail,
        };
        let validator = Validator::new(jwt, scope);
        let strategy = OAuth2Strategy::new(validator);
        OAuth2StrategyAdapter::new(strategy)
    }

    fn create_test_http_request() -> HttpAuthRequest {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer valid_token_123".to_string(),
        );

        HttpAuthRequest::new(
            "POST".to_string(),
            "/mcp/tools/call".to_string(),
            headers,
            HashMap::new(),
        )
        .with_client_ip("192.168.1.1".to_string())
    }

    #[test]
    fn test_extract_bearer_token_success() {
        let adapter = create_test_adapter(false, false);
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer test_token_123".to_string(),
        );

        let result = adapter.extract_bearer_token(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_extract_bearer_token_case_insensitive() {
        let adapter = create_test_adapter(false, false);
        let mut headers = HashMap::new();
        headers.insert(
            "authorization".to_string(),
            "bearer test_token_123".to_string(),
        );

        let result = adapter.extract_bearer_token(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let adapter = create_test_adapter(false, false);
        let headers = HashMap::new();

        let result = adapter.extract_bearer_token(&headers);
        assert!(result.is_err());
        match result.unwrap_err() {
            HttpAuthError::MissingHeader { header } => {
                assert_eq!(header, "Authorization");
            }
            _ => panic!("Expected MissingHeader error"),
        }
    }

    #[test]
    fn test_extract_bearer_token_malformed() {
        let adapter = create_test_adapter(false, false);
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Basic dXNlcjpwYXNz".to_string(),
        );

        let result = adapter.extract_bearer_token(&headers);
        assert!(result.is_err());
        match result.unwrap_err() {
            HttpAuthError::MalformedAuth { message } => {
                assert!(message.contains("Expected 'Bearer <token>' format"));
            }
            _ => panic!("Expected MalformedAuth error"),
        }
    }

    #[test]
    fn test_extract_method_mcp_path() {
        let adapter = create_test_adapter(false, false);

        assert_eq!(
            adapter.extract_method("/mcp/tools/call"),
            Some("tools/call".to_string())
        );
        assert_eq!(
            adapter.extract_method("/mcp/resources/list"),
            Some("resources/list".to_string())
        );
    }

    #[test]
    fn test_extract_method_api_path() {
        let adapter = create_test_adapter(false, false);

        assert_eq!(
            adapter.extract_method("/api/v1/tools/call"),
            Some("tools/call".to_string())
        );
        assert_eq!(
            adapter.extract_method("/api/v1/resources/list"),
            Some("resources/list".to_string())
        );
    }

    #[test]
    fn test_extract_method_root_path() {
        let adapter = create_test_adapter(false, false);

        assert_eq!(
            adapter.extract_method("/tools/call"),
            Some("tools/call".to_string())
        );
        assert_eq!(
            adapter.extract_method("/resources/list"),
            Some("resources/list".to_string())
        );
    }

    #[test]
    fn test_extract_method_empty_path() {
        let adapter = create_test_adapter(false, false);

        assert_eq!(adapter.extract_method(""), None);
        assert_eq!(adapter.extract_method("/"), None);
    }

    #[test]
    fn test_convert_http_request_success() {
        let adapter = create_test_adapter(false, false);
        let http_request = create_test_http_request();

        let result = adapter.convert_http_request(&http_request);
        assert!(result.is_ok());

        let oauth2_request = result.unwrap();
        assert_eq!(oauth2_request.bearer_token, "valid_token_123");
        assert_eq!(oauth2_request.method, Some("tools/call".to_string()));
        assert_eq!(
            oauth2_request.metadata.get("http_method"),
            Some(&"POST".to_string())
        );
        assert_eq!(
            oauth2_request.metadata.get("client_ip"),
            Some(&"192.168.1.1".to_string())
        );
    }

    #[tokio::test]
    async fn test_authenticate_http_success() {
        let adapter = create_test_adapter(false, false);
        let http_request = create_test_http_request();

        let result = adapter.authenticate_http(&http_request).await;
        assert!(result.is_ok());

        let auth_context = result.unwrap();
        assert_eq!(auth_context.method.as_str(), "oauth2");
        assert_eq!(auth_context.auth_data.user_id(), "test_user_123");
    }

    #[tokio::test]
    async fn test_authenticate_http_jwt_failure() {
        let adapter = create_test_adapter(true, false); // JWT validation fails
        let http_request = create_test_http_request();

        let result = adapter.authenticate_http(&http_request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            HttpAuthError::AuthenticationFailed { message } => {
                assert!(message.contains("OAuth2 authentication failed"));
            }
            _ => panic!("Expected AuthenticationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_authenticate_http_missing_auth_header() {
        let adapter = create_test_adapter(false, false);
        let headers = HashMap::new();
        // No Authorization header

        let http_request = HttpAuthRequest::new(
            "POST".to_string(),
            "/mcp/tools/call".to_string(),
            headers,
            HashMap::new(),
        );

        let result = adapter.authenticate_http(&http_request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            HttpAuthError::MissingHeader { header } => {
                assert_eq!(header, "Authorization");
            }
            _ => panic!("Expected MissingHeader error"),
        }
    }
}
