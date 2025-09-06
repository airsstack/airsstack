//! Authorization Error Types
//!
//! Error types for authorization failures with structured error information
//! for debugging and audit logging.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use thiserror::Error;

// Layer 3: Internal module imports

/// Authorization error type
///
/// Structured errors for authorization failures with context information
/// for debugging, logging, and user feedback.
#[derive(Debug, Error, Clone)]
pub enum AuthzError {
    /// Access denied due to insufficient scope/permissions
    #[error("Insufficient scope: required '{required}', available: {available:?}")]
    InsufficientScope {
        required: String,
        available: Vec<String>,
    },

    /// General access denied error
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },

    /// Authorization context has expired
    #[error("Authorization expired at {expired_at}")]
    Expired { expired_at: String },

    /// Invalid authorization context
    #[error("Invalid authorization context: {reason}")]
    InvalidContext { reason: String },

    /// Method not authorized for this context
    #[error("Method '{method}' not authorized for subject '{subject}'")]
    MethodNotAuthorized { method: String, subject: String },

    /// Internal authorization system error
    #[error("Authorization system error: {message}")]
    Internal { message: String },
}

/// Result type for authorization operations
pub type AuthzResult<T> = Result<T, AuthzError>;

impl AuthzError {
    /// Create an insufficient scope error
    pub fn insufficient_scope(required: impl Into<String>, available: Vec<String>) -> Self {
        Self::InsufficientScope {
            required: required.into(),
            available,
        }
    }

    /// Create a general access denied error
    pub fn access_denied(reason: impl Into<String>) -> Self {
        Self::AccessDenied {
            reason: reason.into(),
        }
    }

    /// Create an expired authorization error
    pub fn expired(expired_at: impl Into<String>) -> Self {
        Self::Expired {
            expired_at: expired_at.into(),
        }
    }

    /// Create an invalid context error
    pub fn invalid_context(reason: impl Into<String>) -> Self {
        Self::InvalidContext {
            reason: reason.into(),
        }
    }

    /// Create a method not authorized error
    pub fn method_not_authorized(method: impl Into<String>, subject: impl Into<String>) -> Self {
        Self::MethodNotAuthorized {
            method: method.into(),
            subject: subject.into(),
        }
    }

    /// Create an internal system error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Check if this error is related to insufficient permissions
    pub fn is_permission_error(&self) -> bool {
        matches!(
            self,
            AuthzError::InsufficientScope { .. }
                | AuthzError::AccessDenied { .. }
                | AuthzError::MethodNotAuthorized { .. }
        )
    }

    /// Check if this error is related to expired authorization
    pub fn is_expired_error(&self) -> bool {
        matches!(self, AuthzError::Expired { .. })
    }

    /// Check if this error is a system error
    pub fn is_system_error(&self) -> bool {
        matches!(
            self,
            AuthzError::InvalidContext { .. } | AuthzError::Internal { .. }
        )
    }

    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            AuthzError::InsufficientScope { .. } => "insufficient_scope",
            AuthzError::AccessDenied { .. } => "access_denied",
            AuthzError::Expired { .. } => "expired",
            AuthzError::InvalidContext { .. } => "invalid_context",
            AuthzError::MethodNotAuthorized { .. } => "method_not_authorized",
            AuthzError::Internal { .. } => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_scope_error() {
        let error = AuthzError::insufficient_scope(
            "mcp:write",
            vec!["mcp:read".to_string(), "api:read".to_string()],
        );

        assert_eq!(error.category(), "insufficient_scope");
        assert!(error.is_permission_error());
        assert!(!error.is_expired_error());
        assert!(!error.is_system_error());

        let error_string = error.to_string();
        assert!(error_string.contains("mcp:write"));
        assert!(error_string.contains("mcp:read"));
    }

    #[test]
    fn test_access_denied_error() {
        let error = AuthzError::access_denied("User is not admin");

        assert_eq!(error.category(), "access_denied");
        assert!(error.is_permission_error());

        let error_string = error.to_string();
        assert!(error_string.contains("User is not admin"));
    }

    #[test]
    fn test_expired_error() {
        let error = AuthzError::expired("2025-09-06T10:00:00Z");

        assert_eq!(error.category(), "expired");
        assert!(error.is_expired_error());
        assert!(!error.is_permission_error());

        let error_string = error.to_string();
        assert!(error_string.contains("2025-09-06T10:00:00Z"));
    }

    #[test]
    fn test_invalid_context_error() {
        let error = AuthzError::invalid_context("Missing required claims");

        assert_eq!(error.category(), "invalid_context");
        assert!(error.is_system_error());
        assert!(!error.is_permission_error());
    }

    #[test]
    fn test_method_not_authorized_error() {
        let error = AuthzError::method_not_authorized("admin_delete", "user123");

        assert_eq!(error.category(), "method_not_authorized");
        assert!(error.is_permission_error());

        let error_string = error.to_string();
        assert!(error_string.contains("admin_delete"));
        assert!(error_string.contains("user123"));
    }

    #[test]
    fn test_internal_error() {
        let error = AuthzError::internal("Database connection failed");

        assert_eq!(error.category(), "internal");
        assert!(error.is_system_error());
        assert!(!error.is_permission_error());
    }
}
