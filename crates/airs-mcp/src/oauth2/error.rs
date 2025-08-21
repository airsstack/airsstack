//! OAuth 2.1 Error Types and RFC 6750 Compliant Error Responses
//!
//! This module provides structured error handling for OAuth 2.1 authentication
//! with RFC 6750 compliant WWW-Authenticate header generation.

use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// OAuth 2.1 specific error types with detailed context
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OAuth2Error {
    /// Invalid token error - token is malformed, expired, or invalid
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// Insufficient scope error - token lacks required permissions
    #[error("Insufficient scope: required '{required}', provided '{provided}'")]
    InsufficientScope { required: String, provided: String },

    /// Missing authorization header or Bearer token
    #[error("Missing authorization token")]
    MissingToken,

    /// Invalid token format (not Bearer, empty, etc.)
    #[error("Invalid token format")]
    InvalidTokenFormat,

    /// JWKS retrieval or validation error
    #[error("JWKS error: {0}")]
    JwksError(String),

    /// Token expired error with expiration details
    #[error("Token expired at {expired_at}")]
    TokenExpired { expired_at: String },

    /// Invalid audience claim in token
    #[error("Invalid audience: expected '{expected}', got '{actual}'")]
    InvalidAudience { expected: String, actual: String },

    /// Invalid issuer claim in token
    #[error("Invalid issuer: expected '{expected}', got '{actual}'")]
    InvalidIssuer { expected: String, actual: String },

    /// Missing authorization header
    #[error("Missing authorization header")]
    MissingAuthorization,

    /// Malformed authorization header
    #[error("Malformed authorization header: {0}")]
    MalformedAuthorization(String),

    /// Token validation error (signature, format, etc.)
    #[error("Token validation failed: {0}")]
    TokenValidation(String),

    /// Configuration error in OAuth setup
    #[error("OAuth configuration error: {0}")]
    Configuration(String),
}

/// Type alias for OAuth 2.1 results
pub type OAuth2Result<T> = Result<T, OAuth2Error>;

/// RFC 6750 Bearer Token Error Response
#[derive(Debug, Serialize, Deserialize)]
pub struct BearerTokenError {
    /// The error code as defined in RFC 6750
    pub error: String,
    /// Human-readable error description
    pub error_description: Option<String>,
    /// URI for additional error information
    pub error_uri: Option<String>,
}

impl OAuth2Error {
    /// Convert OAuth2Error to RFC 6750 compliant error code
    pub fn to_error_code(&self) -> &'static str {
        match self {
            OAuth2Error::InvalidToken(_) => "invalid_token",
            OAuth2Error::InsufficientScope { .. } => "insufficient_scope",
            OAuth2Error::MissingToken => "invalid_request",
            OAuth2Error::InvalidTokenFormat => "invalid_request",
            OAuth2Error::TokenExpired { .. } => "invalid_token",
            OAuth2Error::InvalidAudience { .. } => "invalid_token",
            OAuth2Error::InvalidIssuer { .. } => "invalid_token",
            OAuth2Error::MissingAuthorization => "invalid_request",
            OAuth2Error::MalformedAuthorization(_) => "invalid_request",
            OAuth2Error::TokenValidation(_) => "invalid_token",
            OAuth2Error::JwksError(_) => "temporarily_unavailable",
            OAuth2Error::Configuration(_) => "server_error",
        }
    }

    /// Convert OAuth2Error to appropriate HTTP status code
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            OAuth2Error::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            OAuth2Error::InsufficientScope { .. } => StatusCode::FORBIDDEN,
            OAuth2Error::MissingToken => StatusCode::UNAUTHORIZED,
            OAuth2Error::InvalidTokenFormat => StatusCode::BAD_REQUEST,
            OAuth2Error::TokenExpired { .. } => StatusCode::UNAUTHORIZED,
            OAuth2Error::InvalidAudience { .. } => StatusCode::UNAUTHORIZED,
            OAuth2Error::InvalidIssuer { .. } => StatusCode::UNAUTHORIZED,
            OAuth2Error::MissingAuthorization => StatusCode::UNAUTHORIZED,
            OAuth2Error::MalformedAuthorization(_) => StatusCode::BAD_REQUEST,
            OAuth2Error::TokenValidation(_) => StatusCode::UNAUTHORIZED,
            OAuth2Error::JwksError(_) => StatusCode::SERVICE_UNAVAILABLE,
            OAuth2Error::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Generate RFC 6750 compliant WWW-Authenticate header value
    pub fn to_www_authenticate_header(&self) -> String {
        let error_code = self.to_error_code();
        let error_description = self.to_string();

        match self {
            OAuth2Error::InsufficientScope { required, .. } => {
                format!(
                    r#"Bearer error="{error_code}", error_description="{error_description}", scope="{required}""#
                )
            }
            _ => {
                format!(r#"Bearer error="{error_code}", error_description="{error_description}""#)
            }
        }
    }

    /// Create BearerTokenError for JSON error responses
    pub fn to_bearer_token_error(&self) -> BearerTokenError {
        BearerTokenError {
            error: self.to_error_code().to_string(),
            error_description: Some(self.to_string()),
            error_uri: None, // Could be configured to point to documentation
        }
    }
}

/// Implement IntoResponse for OAuth2Error to provide Axum integration
impl IntoResponse for OAuth2Error {
    fn into_response(self) -> Response {
        let status = self.to_status_code();
        let www_authenticate = self.to_www_authenticate_header();
        let error_response = self.to_bearer_token_error();

        let mut headers = HeaderMap::new();

        // Add RFC 6750 compliant WWW-Authenticate header
        if let Ok(header_value) = HeaderValue::from_str(&www_authenticate) {
            headers.insert("WWW-Authenticate", header_value);
        }

        // Return JSON error response with appropriate headers
        (status, headers, Json(error_response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            OAuth2Error::InvalidToken("test".to_string()).to_error_code(),
            "invalid_token"
        );
        assert_eq!(
            OAuth2Error::InsufficientScope {
                required: "read".to_string(),
                provided: "write".to_string()
            }
            .to_error_code(),
            "insufficient_scope"
        );
        assert_eq!(
            OAuth2Error::MissingAuthorization.to_error_code(),
            "invalid_request"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            OAuth2Error::InvalidToken("test".to_string()).to_status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            OAuth2Error::InsufficientScope {
                required: "read".to_string(),
                provided: "write".to_string()
            }
            .to_status_code(),
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn test_www_authenticate_header() {
        let error = OAuth2Error::InvalidToken("expired".to_string());
        let header = error.to_www_authenticate_header();
        assert!(header.contains("Bearer error=\"invalid_token\""));
        assert!(header.contains("error_description=\"Invalid token: expired\""));

        let scope_error = OAuth2Error::InsufficientScope {
            required: "mcp:tools:execute".to_string(),
            provided: "mcp:tools:read".to_string(),
        };
        let scope_header = scope_error.to_www_authenticate_header();
        assert!(scope_header.contains("scope=\"mcp:tools:execute\""));
    }

    #[test]
    fn test_bearer_token_error() {
        let error = OAuth2Error::InvalidToken("malformed".to_string());
        let bearer_error = error.to_bearer_token_error();
        assert_eq!(bearer_error.error, "invalid_token");
        assert!(bearer_error.error_description.is_some());
        assert!(bearer_error.error_uri.is_none());
    }
}
