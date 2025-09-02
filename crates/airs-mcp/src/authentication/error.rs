//! Authentication Error Types
//!
//! Error types for authentication operations.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports
// (none for this module)

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication error types
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    /// Missing credentials in request
    #[error("Missing credentials: {0}")]
    MissingCredentials(String),

    /// Invalid credentials provided
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    /// Authentication method not supported
    #[error("Unsupported authentication method: {0}")]
    Unsupported(String),

    /// Authentication timeout
    #[error("Authentication timeout")]
    Timeout,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal authentication error
    #[error("Internal authentication error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let error = AuthError::MissingCredentials("Authorization header required".to_string());
        assert_eq!(
            format!("{}", error),
            "Missing credentials: Authorization header required"
        );
    }

    #[test]
    fn test_auth_result() {
        let success: AuthResult<String> = Ok("authenticated".to_string());
        assert!(success.is_ok());

        let failure: AuthResult<String> = Err(AuthError::Timeout);
        assert!(failure.is_err());
    }
}
