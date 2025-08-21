//! OAuth 2.1 Middleware Types
//!
//! This module defines common types, enums, and data structures used
//! across OAuth middleware implementations.

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::oauth2::OAuth2Error;

/// OAuth middleware operation result
pub type MiddlewareResult<T> = Result<T, MiddlewareError>;

/// OAuth middleware specific errors
///
/// These errors are specific to middleware operations and complement
/// the core OAuth2Error types with middleware-specific error conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum MiddlewareError {
    /// Core OAuth 2.1 error
    OAuth(OAuth2Error),

    /// HTTP framework specific error
    Http(HttpError),

    /// Request parsing error
    RequestParsing(String),

    /// Response building error
    ResponseBuilding(String),

    /// Configuration error
    Configuration(String),

    /// Internal middleware error
    Internal(String),
}

/// HTTP framework specific errors
#[derive(Debug, Clone, PartialEq)]
pub enum HttpError {
    /// Invalid header format
    InvalidHeader(String),

    /// Missing required header
    MissingHeader(String),

    /// Request body parsing error
    BodyParsing(String),

    /// Response serialization error
    ResponseSerialization(String),

    /// Framework specific error
    Framework(String),
}

/// OAuth middleware configuration
///
/// Common configuration options that apply across different framework implementations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// Authentication realm for WWW-Authenticate headers
    pub auth_realm: Option<String>,

    /// Paths that should skip OAuth validation
    pub skip_paths: Vec<String>,

    /// Whether to include detailed error information in responses
    pub include_error_details: bool,

    /// CORS configuration for OAuth endpoints
    pub cors_config: Option<CorsConfig>,

    /// Request timeout for OAuth operations
    pub request_timeout_seconds: Option<u64>,

    /// Whether to log authentication attempts
    pub log_auth_attempts: bool,
}

/// CORS configuration for OAuth endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins for CORS requests
    pub allowed_origins: Vec<String>,

    /// Allowed methods for CORS requests
    pub allowed_methods: Vec<String>,

    /// Allowed headers for CORS requests
    pub allowed_headers: Vec<String>,

    /// Whether credentials are allowed
    pub allow_credentials: bool,

    /// Max age for preflight cache
    pub max_age_seconds: Option<u64>,
}

/// OAuth middleware operation context
///
/// Context information passed through middleware processing pipeline.
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    /// Request ID for tracing
    pub request_id: String,

    /// Start time for duration tracking
    pub start_time: std::time::Instant,

    /// Whether OAuth validation was skipped
    pub oauth_skipped: bool,

    /// Extracted resource path
    pub resource_path: Option<String>,

    /// HTTP method
    pub http_method: Option<String>,

    /// Additional context data
    pub metadata: std::collections::HashMap<String, String>,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            auth_realm: Some("OAuth 2.1 Protected Resource".to_string()),
            skip_paths: vec![
                "/health".to_string(),
                "/.well-known/oauth-protected-resource".to_string(),
                "/docs".to_string(),
                "/openapi.json".to_string(),
            ],
            include_error_details: true,
            cors_config: Some(CorsConfig::default()),
            request_timeout_seconds: Some(30),
            log_auth_attempts: true,
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
            allowed_headers: vec![
                "Authorization".to_string(),
                "Content-Type".to_string(),
                "Accept".to_string(),
            ],
            allow_credentials: false,
            max_age_seconds: Some(3600), // 1 hour
        }
    }
}

impl MiddlewareContext {
    /// Create a new middleware context
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            start_time: std::time::Instant::now(),
            oauth_skipped: false,
            resource_path: None,
            http_method: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get elapsed time since context creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Add metadata to context
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata from context
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MiddlewareError::OAuth(err) => write!(f, "OAuth error: {err}"),
            MiddlewareError::Http(err) => write!(f, "HTTP error: {err}"),
            MiddlewareError::RequestParsing(msg) => write!(f, "Request parsing error: {msg}"),
            MiddlewareError::ResponseBuilding(msg) => write!(f, "Response building error: {msg}"),
            MiddlewareError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            MiddlewareError::Internal(msg) => write!(f, "Internal middleware error: {msg}"),
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::InvalidHeader(header) => write!(f, "Invalid header: {header}"),
            HttpError::MissingHeader(header) => write!(f, "Missing header: {header}"),
            HttpError::BodyParsing(msg) => write!(f, "Body parsing error: {msg}"),
            HttpError::ResponseSerialization(msg) => {
                write!(f, "Response serialization error: {msg}")
            }
            HttpError::Framework(msg) => write!(f, "Framework error: {msg}"),
        }
    }
}

impl std::error::Error for MiddlewareError {}
impl std::error::Error for HttpError {}

impl From<OAuth2Error> for MiddlewareError {
    fn from(err: OAuth2Error) -> Self {
        MiddlewareError::OAuth(err)
    }
}

impl From<HttpError> for MiddlewareError {
    fn from(err: HttpError) -> Self {
        MiddlewareError::Http(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_config_default() {
        let config = MiddlewareConfig::default();
        assert!(config.include_error_details);
        assert!(config.log_auth_attempts);
        assert_eq!(
            config.auth_realm,
            Some("OAuth 2.1 Protected Resource".to_string())
        );
        assert!(config.skip_paths.contains(&"/health".to_string()));
    }

    #[test]
    fn test_cors_config_default() {
        let cors = CorsConfig::default();
        assert_eq!(cors.allowed_origins, vec!["*"]);
        assert!(cors.allowed_methods.contains(&"POST".to_string()));
        assert!(!cors.allow_credentials);
    }

    #[test]
    fn test_middleware_context() {
        let mut context = MiddlewareContext::new("test-request-123".to_string());
        assert_eq!(context.request_id, "test-request-123");
        assert!(!context.oauth_skipped);

        context.add_metadata("user_agent".to_string(), "test-client/1.0".to_string());
        assert_eq!(
            context.get_metadata("user_agent"),
            Some(&"test-client/1.0".to_string())
        );
    }

    #[test]
    fn test_error_conversions() {
        let oauth_error = OAuth2Error::MissingToken;
        let middleware_error: MiddlewareError = oauth_error.into();

        match middleware_error {
            MiddlewareError::OAuth(OAuth2Error::MissingToken) => (),
            _ => panic!("Expected OAuth error conversion"),
        }
    }
}

/// Framework-agnostic middleware request abstraction
///
/// This struct provides a common interface for HTTP requests across different
/// web frameworks, allowing the OAuth middleware core to work with any framework.
#[derive(Debug, Clone)]
pub struct MiddlewareRequest {
    /// Request method (GET, POST, etc.)
    pub method: String,

    /// Request path
    pub path: String,

    /// Request headers
    pub headers: std::collections::HashMap<String, String>,

    /// Query parameters
    pub query_params: std::collections::HashMap<String, String>,

    /// Request body (if applicable)
    pub body: Option<Vec<u8>>,

    /// Remote IP address
    pub remote_addr: Option<String>,
}

/// Framework-agnostic middleware response abstraction
///
/// This struct provides a common interface for HTTP responses across different
/// web frameworks.
#[derive(Debug, Clone)]
pub struct MiddlewareResponse {
    /// HTTP status code
    pub status_code: u16,

    /// Response headers
    pub headers: std::collections::HashMap<String, String>,

    /// Response body
    pub body: Vec<u8>,
}

/// Authentication result from OAuth processing
///
/// Contains the result of OAuth token validation and user authentication.
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    /// Whether authentication was successful
    pub authenticated: bool,

    /// Validated access token claims (if successful)
    pub token_claims: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Extracted scopes from the token
    pub scopes: Vec<String>,

    /// User identifier (e.g., sub claim)
    pub user_id: Option<String>,

    /// Client identifier
    pub client_id: Option<String>,

    /// Token expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Additional authentication context
    pub context: std::collections::HashMap<String, String>,
}

/// Result of OAuth request processing
///
/// Contains the outcome of processing an authenticated request, including
/// authorization decisions and required scopes.
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Whether the request is authorized to proceed
    pub authorized: bool,

    /// Required scope for the requested resource (if authorization failed)
    pub required_scope: Option<String>,

    /// Additional processing context
    pub context: std::collections::HashMap<String, String>,

    /// Processing duration in milliseconds
    pub processing_time_ms: Option<u64>,
}

impl MiddlewareRequest {
    /// Create a new middleware request
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            headers: std::collections::HashMap::new(),
            query_params: std::collections::HashMap::new(),
            body: None,
            remote_addr: None,
        }
    }

    /// Get a header value by name (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v)
    }

    /// Get the Authorization header value
    pub fn authorization_header(&self) -> Option<&String> {
        self.get_header("authorization")
    }
}

impl MiddlewareResponse {
    /// Create a new middleware response
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Set a header value
    pub fn set_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    /// Set the response body from a string
    pub fn set_body(&mut self, body: String) {
        self.body = body.into_bytes();
    }

    /// Set JSON response body
    pub fn set_json_body<T: Serialize>(&mut self, data: &T) -> Result<(), serde_json::Error> {
        let json_string = serde_json::to_string(data)?;
        self.set_body(json_string);
        self.set_header("content-type".to_string(), "application/json".to_string());
        Ok(())
    }
}

impl AuthenticationResult {
    /// Create a successful authentication result
    pub fn success(user_id: String, scopes: Vec<String>) -> Self {
        Self {
            authenticated: true,
            token_claims: None,
            scopes,
            user_id: Some(user_id),
            client_id: None,
            expires_at: None,
            context: std::collections::HashMap::new(),
        }
    }

    /// Create a failed authentication result
    pub fn failure() -> Self {
        Self {
            authenticated: false,
            token_claims: None,
            scopes: Vec::new(),
            user_id: None,
            client_id: None,
            expires_at: None,
            context: std::collections::HashMap::new(),
        }
    }
}

impl ProcessingResult {
    /// Create an authorized processing result
    pub fn authorized() -> Self {
        Self {
            authorized: true,
            required_scope: None,
            context: std::collections::HashMap::new(),
            processing_time_ms: None,
        }
    }

    /// Create an unauthorized processing result with required scope
    pub fn unauthorized(required_scope: String) -> Self {
        Self {
            authorized: false,
            required_scope: Some(required_scope),
            context: std::collections::HashMap::new(),
            processing_time_ms: None,
        }
    }
}
