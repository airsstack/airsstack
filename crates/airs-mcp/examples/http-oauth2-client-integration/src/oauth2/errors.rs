//! OAuth2 error definitions

// Layer 2: Third-party crate imports
use thiserror::Error;

/// Common errors for OAuth2 integration
#[derive(Error, Debug)]
pub enum OAuth2IntegrationError {
    #[error("OAuth2 flow error: {message}")]
    OAuth2Flow { message: String },

    #[error("Token validation error: {message}")]
    TokenValidation { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Authorization failed: {message}")]
    AuthorizationFailed { message: String },

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("MCP protocol error: {message}")]
    McpProtocol { message: String },

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid scope: required {required}, got {actual}")]
    InvalidScope { required: String, actual: String },
}