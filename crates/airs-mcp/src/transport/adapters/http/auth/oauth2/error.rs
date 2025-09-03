//! HTTP Authentication Error Types
//!
//! Error types specific to HTTP OAuth2 authentication operations.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use crate::authentication::error::AuthError;
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

    /// Missing API key in request
    #[error("Missing API key in request")]
    MissingApiKey,

    /// HTTP engine error
    #[error("HTTP engine error: {0}")]
    EngineError(#[from] HttpEngineError),

    /// Generic authentication error
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
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
            HttpAuthError::MissingApiKey => {
                AuthError::MissingCredentials("Missing API key in request".to_string())
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
    use crate::authentication::error::AuthError;

    #[test]
    fn test_authentication_failed_error() {
        let error = HttpAuthError::AuthenticationFailed {
            message: "Invalid token".to_string(),
        };

        assert_eq!(format!("{error}"), "Authentication failed: Invalid token");
    }

    #[test]
    fn test_missing_header_error() {
        let error = HttpAuthError::MissingHeader {
            header: "Authorization".to_string(),
        };

        assert_eq!(format!("{error}"), "Missing required header: Authorization");
    }

    #[test]
    fn test_malformed_auth_error() {
        let error = HttpAuthError::MalformedAuth {
            message: "Invalid format".to_string(),
        };

        assert_eq!(
            format!("{error}"),
            "Malformed authorization header: Invalid format"
        );
    }

    #[test]
    fn test_conversion_to_auth_error() {
        let http_error = HttpAuthError::AuthenticationFailed {
            message: "OAuth2 failed".to_string(),
        };

        let auth_error: AuthError = http_error.into();
        match auth_error {
            AuthError::InvalidCredentials(msg) => {
                assert_eq!(msg, "OAuth2 failed");
            }
            _ => panic!("Expected InvalidCredentials variant"),
        }
    }

    #[test]
    fn test_missing_header_conversion() {
        let http_error = HttpAuthError::MissingHeader {
            header: "Authorization".to_string(),
        };

        let auth_error: AuthError = http_error.into();
        match auth_error {
            AuthError::MissingCredentials(msg) => {
                assert_eq!(msg, "Missing HTTP header: Authorization");
            }
            _ => panic!("Expected MissingCredentials variant"),
        }
    }

    #[test]
    fn test_missing_api_key_error() {
        let error = HttpAuthError::MissingApiKey;
        assert_eq!(format!("{error}"), "Missing API key in request");
    }

    #[test]
    fn test_missing_api_key_conversion() {
        let http_error = HttpAuthError::MissingApiKey;
        let auth_error: AuthError = http_error.into();
        match auth_error {
            AuthError::MissingCredentials(msg) => {
                assert_eq!(msg, "Missing API key in request");
            }
            _ => panic!("Expected MissingCredentials variant"),
        }
    }
}
