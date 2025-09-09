//! Integration Layer Error Types
//!
//! This module defines error types for the integration layer,
//! providing comprehensive error handling for JSON-RPC client operations.

use thiserror::Error;

use crate::correlation::CorrelationError;
use crate::transport::TransportError;

/// Result type for integration layer operations
pub type IntegrationResult<T> = Result<T, IntegrationError>;

/// Integration layer error types
#[derive(Error, Debug)]
pub enum IntegrationError {
    /// Transport layer error
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    /// Correlation layer error
    #[error("Correlation error: {0}")]
    Correlation(#[from] CorrelationError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Client is already shutdown
    #[error("Client has been shutdown")]
    Shutdown,

    /// Invalid method name
    #[error("Invalid method name: {method}")]
    InvalidMethod { method: String },

    /// Handler registration error
    #[error("Handler registration failed: {reason}")]
    HandlerRegistration { reason: String },

    /// Message routing error
    #[error("Message routing failed: {reason}")]
    Routing { reason: String },

    /// Response timeout
    #[error("Response timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Unexpected response format
    #[error("Unexpected response format: {details}")]
    UnexpectedResponse { details: String },

    /// Generic integration error
    #[error("Integration error: {details}")]
    Other { details: String },
}

impl IntegrationError {
    /// Create a new handler registration error
    pub fn handler_registration<S: Into<String>>(reason: S) -> Self {
        Self::HandlerRegistration {
            reason: reason.into(),
        }
    }

    /// Create a new routing error
    pub fn routing<S: Into<String>>(reason: S) -> Self {
        Self::Routing {
            reason: reason.into(),
        }
    }

    /// Create a new timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }

    /// Create a new unexpected response error
    pub fn unexpected_response<S: Into<String>>(details: S) -> Self {
        Self::UnexpectedResponse {
            details: details.into(),
        }
    }

    /// Create a new generic integration error
    pub fn other<S: Into<String>>(details: S) -> Self {
        Self::Other {
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_error_creation() {
        let error = IntegrationError::handler_registration("duplicate handler");
        assert!(error.to_string().contains("Handler registration failed"));

        let error = IntegrationError::routing("no handler found");
        assert!(error.to_string().contains("Message routing failed"));

        let error = IntegrationError::timeout(5000);
        assert!(error.to_string().contains("5000ms"));

        let error = IntegrationError::unexpected_response("invalid format");
        assert!(error.to_string().contains("Unexpected response format"));

        let error = IntegrationError::other("custom error");
        assert!(error.to_string().contains("Integration error"));
    }

    #[test]
    fn test_error_conversions() {
        // Test that transport errors convert properly
        let transport_error = TransportError::Closed;
        let integration_error: IntegrationError = transport_error.into();
        assert!(matches!(integration_error, IntegrationError::Transport(_)));

        // Test that JSON errors convert properly
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let integration_error: IntegrationError = json_error.into();
        assert!(matches!(integration_error, IntegrationError::Json(_)));
    }

    #[test]
    fn test_error_display() {
        let error = IntegrationError::Shutdown;
        assert_eq!(error.to_string(), "Client has been shutdown");

        let error = IntegrationError::InvalidMethod {
            method: "invalid-method".to_string(),
        };
        assert!(error.to_string().contains("invalid-method"));
    }
}
